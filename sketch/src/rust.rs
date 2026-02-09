//! Some of the code for this module comes from the [`cargo_toml`](https://docs.rs/cargo_toml/0.22.3/cargo_toml/index.html) crate.

macro_rules! prop_name {
	($name:ident) => {
		&stringify!($name).replace("_", "-")
	};
}

macro_rules! add_if_false {
	($target:ident, $table:ident => $($names:ident),*) => {
		$(
			if $target.$names.is_some_and(|v| !v) {
				$table[stringify!($names)] = false.into();
			}
		)*
	};
}

macro_rules! add_string {
	($target:ident, $table:ident => $($names:ident),*) => {
		$(
			if let Some(str) = &$target.$names {
				$table[prop_name!($names)] =  str.into();
			}
		)*
	};
}

macro_rules! add_bool {
	($target:ident, $table:ident => $($names:ident),*) => {
		$(
			if $target.$names {
				$table[prop_name!($names)] =  true.into();
			}
		)*
	};
}

macro_rules! add_optional_bool {
	($target:ident, $table:ident => $($names:ident),*) => {
		$(
			if let Some(bool) = $target.$names {
				$table[prop_name!($names)] =  bool.into();
			}
		)*
	};
}

macro_rules! add_value {
	($target:ident, $table:ident => $($names:ident),*) => {
		$(
			if let Some(val) = &$target.$names {
				$table[prop_name!($names)] =  val.as_toml_value().into();
			}
		)*
	};
}

macro_rules! add_map {
	($target:ident, $table:ident => $($names:ident),*) => {
		$(
			if !$target.$names.is_empty() {
				$table[prop_name!($names)] =  Table::from_iter(
					$target.$names.iter().map(
						|(k, v)| (toml_edit::Key::from(k), Item::from(v.as_toml_value()))
					)
				).into();
			}
		)*
	};
}

macro_rules! add_string_list {
	($target:ident, $table:ident => $($names:ident),*) => {
		$(
			if !$target.$names.is_empty() {
				$table[prop_name!($names)] = Array::from_iter(&$target.$names).into();
			}
		)*
	};
}

pub mod package;
pub mod profile_settings;
pub mod workspace;

use std::{
	collections::{BTreeMap, BTreeSet},
	fs::read_to_string,
	mem,
	path::PathBuf,
};

use clap::Args;
use indexmap::{IndexMap, IndexSet};
use merge::Merge;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use toml_edit::{Array, Decor, DocumentMut, InlineTable, Item, Table, Value as TomlValue};

use crate::{
	custom_templating::TemplatingPresetReference,
	fs::{create_all_dirs, serialize_toml, write_file},
	init_repo::gitignore::{GitIgnoreRef, GitignorePreset},
	licenses::License,
	rust::{
		package::Package,
		profile_settings::Profiles,
		workspace::{Lints, Workspace},
	},
	*,
};

pub fn toml_string_list<'a>(strings: impl IntoIterator<Item = &'a String>) -> Item {
	Array::from_iter(strings.into_iter().map(|s| {
		let mut val: TomlValue = s.into();
		*val.decor_mut() = Decor::new("\n  ", "");
		val
	}))
	.into()
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Merge, Default)]
#[serde(default)]
pub struct RustPresets {
	/// A map that contains presets for `Cargo.toml` files.
	#[merge(strategy = merge_index_maps)]
	pub manifest: IndexMap<String, CargoTomlPreset>,

	#[merge(strategy = merge_index_maps)]
	#[serde(rename = "crate")]
	pub crate_: IndexMap<String, Crate>,
}

#[derive(Args, Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Merge, Default)]
#[group(id = "crate_config")]
#[serde(default)]
pub struct Crate {
	#[arg(short, long, value_parser = CargoTomlPresetRef::from_cli, default_value_t = CargoTomlPresetRef::default())]
	#[merge(strategy = overwrite_always)]
	pub manifest: CargoTomlPresetRef,

	#[arg(long)]
	#[merge(strategy = overwrite_if_some)]
	/// Settings for the gitignore file.
	pub gitignore: Option<GitIgnoreRef>,

	#[arg(long)]
	#[merge(strategy = overwrite_if_some)]
	/// A license file to generate for the new repo.
	pub license: Option<License>,

	#[arg(short = 't', long = "template", value_name = "PRESET_ID")]
	#[merge(strategy = merge_vecs)]
	pub with_templates: Vec<TemplatingPresetReference>,
}

impl Crate {
	pub fn generate(
		self,
		dir: &PathBuf,
		name: Option<String>,
		config: &Config,
	) -> Result<(), GenError> {
		if dir.exists() {
			panic!("Dir exists");
		}

		create_all_dirs(dir)?;

		let name = name.unwrap_or_else(|| {
			dir.file_name()
				.expect("Empty path")
				.to_string_lossy()
				.to_string()
		});

		let CargoTomlPresetRef::Config(CargoTomlPreset {
			config: mut manifest,
			..
		}) = self.manifest
		else {
			panic!("Unresolved manifest");
		};

		manifest.package.get_or_insert_default().name = Some(name);

		let workspace_manifest_path = PathBuf::from("Cargo.toml");

		let workspace_manifest = if workspace_manifest_path.exists() {
			let workspace_manifest_raw = read_to_string(&workspace_manifest_path).map_err(|e| {
				GenError::DeserializationError {
					file: workspace_manifest_path.clone(),
					error: e.to_string(),
				}
			})?;

			let mut workspace_manifest_content = workspace_manifest_raw
				.parse::<DocumentMut>()
				.map_err(|e| GenError::DeserializationError {
					file: workspace_manifest_path.clone(),
					error: e.to_string(),
				})?;

			let workspace_entry = workspace_manifest_content
				.entry("workspace")
				.or_insert_with(|| Item::Table(Table::new()));

			let members = workspace_entry
				.as_table_mut()
				.unwrap()
				.entry("members")
				.or_insert_with(|| Item::Value(toml_edit::Value::Array(Array::new())))
				.as_array_mut()
				.unwrap();

			members.set_trailing_comma(true);

			let decor = members
				.get(0)
				.map(|i| i.decor().clone())
				.unwrap_or_else(|| Decor::new("\n  ", ""));

			let mut new_member: toml_edit::Value = dir.to_string_lossy().to_string().into();

			*new_member.decor_mut() = decor;

			members.push(new_member);

			write_file(
				&workspace_manifest_path,
				&workspace_manifest_content.to_string(),
				true,
			)?;

			let workspace_manifest_full: Manifest = toml::from_str(&workspace_manifest_raw)
				.map_err(|e| GenError::DeserializationError {
					file: workspace_manifest_path.clone(),
					error: e.to_string(),
				})?;

			workspace_manifest_full.workspace
		} else {
			None
		};

		if let Some(workspace_manifest) = workspace_manifest {
			if workspace_manifest.lints.is_some() && manifest.lints.is_none() {
				manifest.lints = Some(Inheritable::Workspace { workspace: true });
			}

			if let Some(_) = &workspace_manifest.package {
				let package_config = manifest.package.get_or_insert_default();

				macro_rules! inherit_opt {
					($($name:ident),*) => {
						$(
							package_config.$name = Some(Inheritable::Workspace {
								workspace: true,
							});
						)*
					};
				}

				inherit_opt!(edition, license, repository);

				package_config.keywords = Inheritable::Workspace { workspace: true };
			}
		}

		serialize_toml(&manifest, &dir.join("Cargo.toml"), true)?;

		if let Some(GitIgnoreRef::Config(gitignore)) = self.gitignore {
			write_file(
				&dir.join(".gitignore"),
				&gitignore.content.to_string(),
				true,
			)?;
		}

		if let Some(license) = self.license {
			write_file(&dir.join("LICENSE"), license.get_content(), true)?;
		}

		if !self.with_templates.is_empty() {
			config.generate_templates(dir, self.with_templates, &Default::default())?;
		}

		Ok(())
	}
}

impl Crate {
	pub fn process_data(
		mut self,
		manifests_store: &IndexMap<String, CargoTomlPreset>,
		gitignore_store: &IndexMap<String, GitignorePreset>,
	) -> Result<Self, GenError> {
		let mut manifest_id: Option<String> = None;

		if let CargoTomlPresetRef::Id(id) = self.manifest {
			manifest_id = Some(id.clone());

			let data = manifests_store
				.get(&id)
				.ok_or_else(|| GenError::PresetNotFound {
					kind: Preset::CargoToml,
					name: id,
				})?
				.clone();

			self.manifest = CargoTomlPresetRef::Config(data);
		}

		if let CargoTomlPresetRef::Config(data) = self.manifest {
			self.manifest = CargoTomlPresetRef::Config(data.process_data(
				manifest_id.as_deref().unwrap_or("__inlined"),
				manifests_store,
			)?);
		}

		let mut gitignore_id: Option<String> = None;

		if let Some(GitIgnoreRef::Id(id)) = self.gitignore {
			gitignore_id = Some(id.clone());

			let data = gitignore_store
				.get(&id)
				.ok_or_else(|| GenError::PresetNotFound {
					kind: Preset::Gitignore,
					name: id,
				})?
				.clone();

			self.gitignore = Some(GitIgnoreRef::Config(data));
		}

		if let Some(GitIgnoreRef::Config(data)) = self.gitignore {
			let resolved = data.process_data(
				gitignore_id.as_deref().unwrap_or("__inlined"),
				gitignore_store,
			)?;

			self.gitignore = Some(GitIgnoreRef::Config(resolved));
		}

		Ok(self)
	}
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
pub enum CargoTomlPresetRef {
	Id(String),
	#[serde(untagged)]
	Config(CargoTomlPreset),
}

impl std::fmt::Display for CargoTomlPresetRef {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Id(id) => write!(f, "{id}"),
			Self::Config(_) => write!(f, "default config"),
		}
	}
}

impl Default for CargoTomlPresetRef {
	fn default() -> Self {
		Self::Config(CargoTomlPreset::default())
	}
}

impl CargoTomlPresetRef {
	pub fn from_cli(str: &str) -> Result<Self, String> {
		Ok(Self::Id(str.to_string()))
	}
}

/// A preset for a `Cargo.toml` file.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Default, Merge)]
#[serde(default)]
pub struct CargoTomlPreset {
	/// The list of extended presets.
	#[merge(strategy = merge_index_sets)]
	pub extends_presets: IndexSet<String>,

	#[serde(flatten)]
	#[merge(strategy = merge_nested)]
	pub config: Manifest,
}

impl Extensible for CargoTomlPreset {
	fn get_extended(&self) -> &IndexSet<String> {
		&self.extends_presets
	}
}

impl CargoTomlPreset {
	pub fn process_data(self, id: &str, store: &IndexMap<String, Self>) -> Result<Self, GenError> {
		if self.extends_presets.is_empty() {
			return Ok(self);
		}

		let mut processed_ids: IndexSet<String> = IndexSet::new();

		let merged_preset = merge_presets(Preset::CargoToml, id, self, store, &mut processed_ids)?;

		Ok(merged_preset)
	}
}

/// The top-level `Cargo.toml` structure. **This is the main type in this library.**
///
/// The `Metadata` is a generic type for `[package.metadata]` table. You can replace it with
/// your own struct type if you use the metadata and don't want to use the catch-all `Value` type.
#[derive(Debug, Clone, PartialEq, JsonSchema, Serialize, Deserialize, Merge, Default)]
#[merge(strategy = overwrite_if_some)]
#[serde(rename_all = "kebab-case")]
pub struct Manifest {
	/// Workspace-wide settings
	#[merge(strategy = merge_optional_nested)]
	pub workspace: Option<Workspace>,

	/// Package definition (a cargo crate)
	#[merge(strategy = merge_optional_nested)]
	pub package: Option<Package>,

	/// Note that due to autolibs feature this is not the complete list
	/// unless you run [`Manifest::complete_from_path`]
	#[merge(strategy = merge_optional_nested)]
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub lib: Option<Product>,

	/// Note that due to autobins feature this is not the complete list
	/// unless you run [`Manifest::complete_from_path`]
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	#[merge(strategy = merge_btree_sets)]
	pub bin: BTreeSet<Product>,

	/// `[target.cfg.dependencies]`
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	#[merge(strategy = merge_btree_maps)]
	pub target: BTreeMap<String, Target>,

	/// `[patch.crates-io]` section
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	#[merge(strategy = merge_btree_maps)]
	pub patch: BTreeMap<String, BTreeMap<String, Dependency>>,

	/// Compilation/optimization settings
	#[serde(default, skip_serializing_if = "Option::is_none")]
	#[merge(strategy = merge_optional_nested)]
	pub profile: Option<Profiles>,

	/// Benchmarks
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	#[merge(strategy = merge_btree_sets)]
	pub bench: BTreeSet<Product>,

	/// Integration tests
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	#[merge(strategy = merge_btree_sets)]
	pub test: BTreeSet<Product>,

	/// Examples
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	#[merge(strategy = merge_btree_sets)]
	pub example: BTreeSet<Product>,

	/// Lints
	#[serde(default, skip_serializing_if = "Option::is_none")]
	#[merge(strategy = overwrite_if_some)]
	pub lints: Option<Inheritable<Lints>>,

	/// The `[features]` section. This set may be incomplete!
	///
	/// Optional dependencies may create implied Cargo features.
	/// This features section also supports microsyntax with `dep:`, `/`, and `?`
	/// for managing dependencies and their features.io
	///
	/// This crate has an optional [`features`] module for dealing with this
	/// complexity and getting the real list of features.
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	#[merge(strategy = merge_btree_maps)]
	pub features: BTreeMap<String, BTreeSet<String>>,

	/// Normal dependencies
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	#[merge(strategy = merge_dependencies)]
	pub dependencies: BTreeMap<String, Dependency>,

	/// Dev/test-only deps
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	#[merge(strategy = merge_dependencies)]
	pub dev_dependencies: BTreeMap<String, Dependency>,

	/// Build-time deps
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	#[merge(strategy = merge_dependencies)]
	pub build_dependencies: BTreeMap<String, Dependency>,
}

impl Manifest {
	pub fn as_document(&self) -> DocumentMut {
		let mut document = DocumentMut::new();

		if let Some(workspace) = &self.workspace {
			document.insert("workspace", workspace.as_toml_value());
		}

		add_value!(self, document => package, lib, profile);
		add_map!(self, document => target);

		if !self.bin.is_empty() {
			let array = Array::from_iter(self.bin.iter().map(|i| match i.as_toml_value() {
				Item::None => todo!(),
				Item::Value(value) => value,
				Item::Table(table) => table.into_inline_table().into(),
				Item::ArrayOfTables(_) => todo!(),
			}));

			document["bin"] = array.into();
		}

		if !self.bench.is_empty() {
			let array = Array::from_iter(
				self.bench
					.iter()
					.map(|i| match i.as_toml_value() {
						Item::None => todo!(),
						Item::Value(value) => value,
						Item::Table(table) => table.into_inline_table().into(),
						Item::ArrayOfTables(_) => todo!(),
					}),
			);

			document["bench"] = array.into();
		}

		if !self.test.is_empty() {
			let array = Array::from_iter(self.test.iter().map(|i| match i.as_toml_value() {
				Item::None => todo!(),
				Item::Value(value) => value,
				Item::Table(table) => table.into_inline_table().into(),
				Item::ArrayOfTables(_) => todo!(),
			}));

			document["test"] = array.into();
		}

		if !self.example.is_empty() {
			let array = Array::from_iter(
				self.example
					.iter()
					.map(|i| match i.as_toml_value() {
						Item::None => todo!(),
						Item::Value(value) => value,
						Item::Table(table) => table.into_inline_table().into(),
						Item::ArrayOfTables(_) => todo!(),
					}),
			);

			document["example"] = array.into();
		}

		if !self.patch.is_empty() {
			let patch_table = Table::from_iter(self.patch.iter().map(|(k, v)| {
				(
					k,
					Table::from_iter(v.iter().map(|(k, v)| (k, v.as_toml_value()))),
				)
			}));

			document["patch"] = patch_table.into();
		}

		if let Some(lints) = &self.lints {
			document["lints"] = match lints {
				Inheritable::Workspace { workspace } => {
					InlineTable::from_iter([("workspace", *workspace)]).into()
				}
				Inheritable::Set(lints) => lints.as_toml_value(),
			};
		}

		add_map!(self, document => dev_dependencies, build_dependencies, dependencies);

		if !self.features.is_empty() {
			document["features"] =
				Table::from_iter(self.features.iter().map(|(name, features)| {
					(
						toml_edit::Key::from(name.as_str()),
						Array::from_iter(features),
					)
				}))
				.into();
		}

		document
	}
}

/// Lint level.
#[derive(Debug, PartialEq, Eq, Copy, Clone, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LintLevel {
	Allow,
	Warn,
	ForceWarn,
	Deny,
	Forbid,
}

impl AsTomlValue for LintLevel {
	fn as_toml_value(&self) -> Item {
		let str = match self {
			Self::Allow => "allow",
			Self::Warn => "warn",
			Self::ForceWarn => "force-warn",
			Self::Deny => "deny",
			Self::Forbid => "forbid",
		};

		str.into()
	}
}

/// Lint definition.
#[derive(Debug, Clone, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
pub struct Lint {
	/// allow/warn/deny
	pub level: LintLevel,

	/// Controls which lints or lint groups override other lint groups.
	pub priority: Option<i8>,

	/// Unstable
	pub config: BTreeMap<String, Value>,
}

impl AsTomlValue for Lint {
	fn as_toml_value(&self) -> Item {
		let mut table = Table::new();

		table["level"] = self.level.as_toml_value().into();

		if let Some(priority) = self.priority {
			table["priority"] = i64::from(priority).into();
		}

		table.into()
	}
}

/// Dependencies that are platform-specific or enabled through custom `cfg()`.
#[derive(Debug, Clone, PartialEq, Eq, Default, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Target {
	/// platform-specific normal deps
	#[serde(default)]
	pub dependencies: BTreeMap<String, Dependency>,
	/// platform-specific dev-only/test-only deps
	#[serde(default)]
	pub dev_dependencies: BTreeMap<String, Dependency>,
	/// platform-specific build-time deps
	#[serde(default)]
	pub build_dependencies: BTreeMap<String, Dependency>,
}

impl AsTomlValue for Target {
	fn as_toml_value(&self) -> Item {
		let mut table = Table::new();

		add_map!(self, table => dependencies, dev_dependencies, build_dependencies);

		table.into()
	}
}

/// Dependency definition. Note that this struct doesn't carry it's key/name, which you need to read from its section.
///
/// It can be simple version number, or detailed settings, or inherited.
#[derive(Debug, Clone, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
	/// Version requirement (e.g. `^1.5`)
	Simple(String),
	/// Incomplete data
	Inherited(InheritedDependencyDetail), // order is important for serde
	/// `{ version = "^1.5", features = ["a", "b"] }` etc.
	Detailed(Box<DependencyDetail>),
}

impl AsTomlValue for Dependency {
	fn as_toml_value(&self) -> Item {
		match self {
			Self::Simple(ver) => ver.into(),
			Self::Inherited(dep) => dep.as_toml_value(),
			Self::Detailed(dep) => dep.as_toml_value(),
		}
	}
}

impl Dependency {
	pub fn optional(&self) -> bool {
		match self {
			Self::Simple(_) => false,
			Self::Inherited(dep) => dep.optional,
			Self::Detailed(dep) => dep.optional,
		}
	}

	pub fn features(&self) -> Option<&BTreeSet<String>> {
		match self {
			Self::Simple(_) => None,
			Self::Inherited(dep) => Some(&dep.features),
			Self::Detailed(dep) => Some(&dep.features),
		}
	}
}

pub(crate) fn merge_dependencies(
	left: &mut BTreeMap<String, Dependency>,
	right: BTreeMap<String, Dependency>,
) {
	for (name, dep) in right {
		if let Some(previous) = left.get_mut(&name) {
			previous.merge(dep);
		} else {
			left.insert(name, dep);
		}
	}
}

impl Merge for Dependency {
	fn merge(&mut self, other: Self) {
		match self {
			Self::Simple(_) => {
				*self = other;
			}
			Self::Inherited(left_options) => match other {
				Self::Simple(_) => *self = other,
				Self::Inherited(right) => left_options.merge(right),
				Self::Detailed(mut right) => {
					if left_options.optional {
						right.optional = true;
					}

					let left_features = mem::take(&mut left_options.features);

					right.features.extend(left_features);

					*self = Self::Detailed(right);
				}
			},
			Self::Detailed(left) => match other {
				Self::Simple(_) => *self = other,
				Self::Inherited(mut right) => {
					if left.optional {
						right.optional = true;
					}

					let left_features = mem::take(&mut left.features);

					right.features.extend(left_features);

					*self = Self::Inherited(right);
				}
				Self::Detailed(right) => {
					left.merge(*right);
				}
			},
		}
	}
}

/// When a dependency is defined as `{ workspace = true }`,
/// and workspace data hasn't been applied yet.
#[derive(Debug, Clone, PartialEq, Eq, Default, JsonSchema, Serialize, Deserialize, Merge)]
#[serde(default, rename_all = "kebab-case")]
pub struct InheritedDependencyDetail {
	#[merge(strategy = merge_btree_sets)]
	#[serde(skip_serializing_if = "BTreeSet::is_empty")]
	pub features: BTreeSet<String>,

	#[serde(skip_serializing_if = "crate::is_false")]
	#[merge(strategy = merge::bool::overwrite_true)]
	pub optional: bool,

	#[serde(skip_serializing_if = "crate::is_false")]
	#[merge(strategy = merge::bool::overwrite_true)]
	pub workspace: bool,
}

impl AsTomlValue for InheritedDependencyDetail {
	fn as_toml_value(&self) -> Item {
		let mut table = InlineTable::new();

		add_bool!(self, table => workspace, optional);
		add_string_list!(self, table => features);

		table.into()
	}
}

/// When definition of a dependency is more than just a version string.
#[derive(Debug, Clone, PartialEq, Eq, JsonSchema, Serialize, Deserialize, Merge)]
#[serde(rename_all = "kebab-case")]
#[merge(strategy = overwrite_if_some)]
pub struct DependencyDetail {
	/// Semver requirement. Note that a plain version number implies this version *or newer* compatible one.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub version: Option<String>,

	/// If `Some`, use this as the crate name instead of `[dependencies]`'s table key.
	///
	/// By using this, a crate can have multiple versions of the same dependency.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub package: Option<String>,

	/// Fetch this dependency from a custom 3rd party registry (alias defined in Cargo config), not crates-io.
	///
	/// This depends on local cargo configuration. It becomes `registry_index` after the crate is uploaded to a registry.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub registry: Option<String>,

	/// Directly define custom 3rd party registry URL (may be `sparse+https:`) instead of a config nickname.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub registry_index: Option<String>,

	/// This path is usually relative to the crate's manifest, but when using workspace inheritance, it may be relative to the workspace!
	///
	/// When calling [`Manifest::complete_from_path_and_workspace`] use absolute path for the workspace manifest, and then this will be corrected to be an absolute
	/// path when inherited from the workspace.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub path: Option<String>,

	/// Read dependency from git repo URL, not allowed on crates-io.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub git: Option<String>,
	/// Read dependency from git branch, not allowed on crates-io.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub branch: Option<String>,
	/// Read dependency from git tag, not allowed on crates-io.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub tag: Option<String>,
	/// Read dependency from git commit, not allowed on crates-io.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub rev: Option<String>,

	/// Enable these features of the dependency.
	///
	/// Note that Cargo interprets `default` in a special way.
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	#[merge(strategy = merge_btree_sets)]
	pub features: BTreeSet<String>,

	/// NB: Not allowed at workspace level
	///
	/// If not used with `dep:` or `?/` syntax in `[features]`, this also creates an implicit feature.
	/// See the [`features`] module for more info.
	#[serde(skip_serializing_if = "crate::is_false")]
	#[merge(strategy = merge::bool::overwrite_true)]
	pub optional: bool,

	/// Enable the `default` set of features of the dependency (enabled by default).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub default_features: Option<bool>,

	/// Contains the remaining unstable keys and values for the dependency.
	#[serde(flatten)]
	#[merge(strategy = merge_btree_maps)]
	pub unstable: BTreeMap<String, Value>,
}

impl AsTomlValue for DependencyDetail {
	fn as_toml_value(&self) -> Item {
		let mut table = InlineTable::new();

		add_string!(self, table => version, package, registry, registry_index, path, git, branch, tag, rev);

		add_string_list!(self, table => features);

		add_bool!(self, table => optional);

		add_if_false!(self, table => default_features);

		table.into()
	}
}

/// A value that can be set to `workspace`
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(untagged)]
pub enum Inheritable<T> {
	/// Inherit this setting from the `workspace`
	#[serde(rename = "workspace")]
	Workspace {
		workspace: bool,
	},
	Set(T),
}

impl<T> Inheritable<T> {
	pub fn is_workspace(&self) -> bool {
		matches!(self, Self::Workspace { workspace: true })
	}
}

impl<T: AsTomlValue> AsTomlValue for Inheritable<T> {
	fn as_toml_value(&self) -> Item {
		match self {
			Self::Workspace { workspace } => Table::from_iter([("workspace", *workspace)]).into(),
			Self::Set(set) => set.as_toml_value(),
		}
	}
}

pub trait AsTomlValue {
	fn as_toml_value(&self) -> Item;
}

impl<T: ?Sized + Into<Item> + Clone> AsTomlValue for T {
	fn as_toml_value(&self) -> Item {
		self.clone().into()
	}
}

impl<T: Default> Default for Inheritable<T> {
	fn default() -> Self {
		Self::Set(T::default())
	}
}

pub(crate) fn merge_inheritable_set<T: Ord>(
	left: &mut Inheritable<BTreeSet<T>>,
	right: Inheritable<BTreeSet<T>>,
) {
	match left {
		Inheritable::Workspace { .. } => {
			*left = right;
		}
		Inheritable::Set(left_list) => {
			match right {
				Inheritable::Workspace { workspace } => {
					*left = Inheritable::Workspace { workspace }
				}
				Inheritable::Set(right_list) => left_list.extend(right_list),
			};
		}
	}
}

pub(crate) fn merge_inheritable_map<T>(
	left: &mut Option<Inheritable<BTreeMap<String, T>>>,
	right: Option<Inheritable<BTreeMap<String, T>>>,
) {
	if let Some(right) = right {
		if let Some(left) = left {
			match left {
				Inheritable::Workspace { .. } => {
					*left = right;
				}
				Inheritable::Set(left_list) => {
					match right {
						Inheritable::Workspace { workspace } => {
							*left = Inheritable::Workspace { workspace }
						}
						Inheritable::Set(right_list) => {
							for (key, val) in right_list {
								left_list.insert(key, val);
							}
						}
					};
				}
			}
		} else {
			*left = Some(right);
		}
	}
}

impl<T: Default + PartialEq> Inheritable<T> {
	pub fn is_default(&self) -> bool {
		match self {
			Self::Workspace { .. } => false,
			Self::Set(v) => T::default() == *v,
		}
	}
}

/// Edition setting, which opts in to new Rust/Cargo behaviors.
#[derive(
	Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Default, Eq, PartialOrd, Ord,
)]
pub enum Edition {
	/// 2015
	#[default]
	#[serde(rename = "2015")]
	E2015 = 2015,
	/// 2018
	#[serde(rename = "2018")]
	E2018 = 2018,
	/// 2021
	#[serde(rename = "2021")]
	E2021 = 2021,
	/// 2024
	#[serde(rename = "2024")]
	E2024 = 2024,
}

impl AsTomlValue for Edition {
	fn as_toml_value(&self) -> Item {
		let str = match self {
			Self::E2015 => "2015",
			Self::E2018 => "2018",
			Self::E2021 => "2021",
			Self::E2024 => "2024",
		};

		str.into()
	}
}

/// A way specify or disable README or `build.rs`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(
	untagged,
	expecting = "the value should be either a boolean or a file path"
)]
pub enum OptionalFile {
	/// Opt-in to default, or explicit opt-out
	Flag(bool),
	/// Explicit path
	Path(PathBuf),
}

impl AsTomlValue for OptionalFile {
	fn as_toml_value(&self) -> Item {
		match self {
			Self::Flag(bool) => (*bool).into(),
			Self::Path(path) => path.to_string_lossy().as_ref().into(),
		}
	}
}

/// Forbids or selects custom registry
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(
	untagged,
	expecting = "the value should be either a boolean, or an array of registry names"
)]
pub enum Publish {
	Flag(bool),
	Registry(BTreeSet<String>),
}

impl AsTomlValue for Publish {
	fn as_toml_value(&self) -> Item {
		match self {
			Self::Flag(bool) => (*bool).into(),
			Self::Registry(list) => toml_string_list(list),
		}
	}
}

/// The feature resolver version.
///
/// Needed in [`Workspace`], but implied by [`Edition`] in packages.
#[derive(
	Debug, Default, PartialEq, Eq, Ord, PartialOrd, Copy, Clone, Serialize, Deserialize, JsonSchema,
)]
#[serde(
	expecting = "if there's a newer resolver, then this parser (cargo_toml crate) has to be updated"
)]
#[repr(u8)]
pub enum Resolver {
	#[serde(rename = "1")]
	/// The default for editions prior to 2021.
	V1 = 1,
	/// The default for the 2021 edition.
	#[serde(rename = "2")]
	V2 = 2,
	/// The default for the 2024 edition.
	#[serde(rename = "3")]
	#[default]
	V3 = 3,
}

impl AsTomlValue for Resolver {
	fn as_toml_value(&self) -> Item {
		((*self) as u8 as i64).into()
	}
}

#[derive(
	Debug, Clone, PartialEq, Eq, JsonSchema, Serialize, Deserialize, PartialOrd, Ord, Merge,
)]
#[serde(rename_all = "kebab-case")]
#[merge(strategy = overwrite_if_some)]
/// Cargo uses the term "target" for both "target platform" and "build target" (the thing to build),
/// which makes it ambigous.
/// Here Cargo's bin/lib **target** is renamed to **product**.
pub struct Product {
	/// This field points at where the crate is located, relative to the `Cargo.toml`.
	pub path: Option<String>,

	/// The name of a product is the name of the library or binary that will be generated.
	/// This is defaulted to the name of the package, with any dashes replaced
	/// with underscores. (Rust `extern crate` declarations reference this name;
	/// therefore the value must be a valid Rust identifier to be usable.)
	pub name: Option<String>,

	/// A flag for enabling unit tests for this product. This is used by `cargo test`.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub test: Option<bool>,

	/// A flag for enabling documentation tests for this product. This is only relevant
	/// for libraries, it has no effect on other sections. This is used by
	/// `cargo test`.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub doctest: Option<bool>,

	/// A flag for enabling benchmarks for this product. This is used by `cargo bench`.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub bench: Option<bool>,

	/// A flag for enabling documentation of this product. This is used by `cargo doc`.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub doc: Option<bool>,

	/// If the product is meant to be a "macros 1.1" procedural macro, this field must
	/// be set to true.
	#[serde(
		default,
		alias = "proc_macro",
		alias = "proc-macro",
		skip_serializing_if = "crate::is_false"
	)]
	#[merge(strategy = merge::bool::overwrite_true)]
	pub proc_macro: bool,

	/// If set to false, `cargo test` will omit the `--test` flag to rustc, which
	/// stops it from generating a test harness. This is useful when the binary being
	/// built manages the test runner itself.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub harness: Option<bool>,

	/// Deprecated. Edition should be set only per package.
	///
	/// If set then a product can be configured to use a different edition than the
	/// `[package]` is configured to use, perhaps only compiling a library with the
	/// 2018 edition or only compiling one unit test with the 2015 edition. By default
	/// all products are compiled with the edition specified in `[package]`.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub edition: Option<Edition>,

	/// The available options are "dylib", "rlib", "staticlib", "cdylib", and "proc-macro".
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	#[merge(strategy = merge_btree_sets)]
	pub crate_type: BTreeSet<String>,

	/// The `required-features` field specifies which features the product needs in order to be built.
	/// If any of the required features are not selected, the product will be skipped.
	/// This is only relevant for the `[[bin]]`, `[[bench]]`, `[[test]]`, and `[[example]]` sections,
	/// it has no effect on `[lib]`.
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	#[merge(strategy = merge_btree_sets)]
	pub required_features: BTreeSet<String>,
}

impl AsTomlValue for Product {
	fn as_toml_value(&self) -> Item {
		let mut table = Table::new();

		add_value!(self, table => edition);
		add_string!(self, table => path, name);
		add_bool!(self, table => proc_macro);
		add_string_list!(self, table => crate_type, required_features);

		add_if_false!(self, table => test, doctest, bench, doc, harness);

		table.into()
	}
}
