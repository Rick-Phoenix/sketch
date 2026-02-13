#[macro_use]
mod macros;
mod toml_helpers;
use toml_helpers::*;
mod package;
pub use package::*;
mod profile_settings;
pub use profile_settings::*;
mod workspace;
pub use workspace::*;

use merge_it::*;
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::mem;
use std::path::PathBuf;
use toml_edit::{Array, ArrayOfTables, DocumentMut, InlineTable, Item, Table, Value as TomlValue};

// Some of the code for this module comes from the [`cargo_toml`](https://docs.rs/cargo_toml/0.22.3/cargo_toml/index.html) crate.

/// The top-level `Cargo.toml` structure.
///
/// For more info, visit the [manifest guide](https://doc.rust-lang.org/cargo/reference/manifest.html#the-manifest-format)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Merge, Default)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub struct Manifest {
	/// The workspace definition.
	#[merge(with = merge_options)]
	pub workspace: Option<Workspace>,

	/// Package definition (a cargo crate)
	#[merge(with = merge_options)]
	pub package: Option<Package>,

	/// Library target settings.
	#[merge(with = merge_options)]
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub lib: Option<Product>,

	/// Binary target settings.
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	pub bin: BTreeSet<Product>,

	/// Platform-specific dependencies.
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	pub target: BTreeMap<String, Target>,

	/// Override dependencies.
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	pub patch: BTreeMap<String, BTreeMap<String, Dependency>>,

	/// Compilation/optimization settings
	#[serde(default, skip_serializing_if = "Option::is_none")]
	#[merge(with = merge_options)]
	pub profile: Option<Profiles>,

	/// Benchmarks
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	pub bench: BTreeSet<Product>,

	/// Integration tests
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	pub test: BTreeSet<Product>,

	/// Examples
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	pub example: BTreeSet<Product>,

	/// Lints
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub lints: Option<Inheritable<Lints>>,

	/// The `[features]` section.
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	pub features: BTreeMap<String, BTreeSet<String>>,

	/// Normal dependencies
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	#[merge(with = merge_btree_maps)]
	pub dependencies: BTreeMap<String, Dependency>,

	/// Dev/test-only dependencies
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	#[merge(with = merge_btree_maps)]
	pub dev_dependencies: BTreeMap<String, Dependency>,

	/// Build-time dependencies
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	#[merge(with = merge_btree_maps)]
	pub build_dependencies: BTreeMap<String, Dependency>,
}

impl Manifest {
	/// Turns the manifest into a [`DocumentMut`] that can be used for more customized serialization.
	pub fn as_document(&self) -> DocumentMut {
		let mut document = DocumentMut::new();

		if let Some(workspace) = &self.workspace {
			document.insert("workspace", workspace.as_toml_value());
		}

		add_value!(self, document => package, lib, profile);

		if !self.target.is_empty() {
			let mut table = Table::from_iter(
				self.target
					.iter()
					.map(|(name, target)| (toml_edit::Key::from(name), target.as_toml_value())),
			);

			table.set_implicit(true);

			document["target"] = table.into();
		}

		if !self.bin.is_empty() {
			let array =
				ArrayOfTables::from_iter(self.bin.iter().map(|i| match i.as_toml_value() {
					Item::Table(table) => table,
					_ => panic!("Found non-tables for cargo toml bin"),
				}));

			document["bin"] = array.into();
		}

		if !self.bench.is_empty() {
			let array = ArrayOfTables::from_iter(self.bench.iter().map(
				|i| match i.as_toml_value() {
					Item::Table(table) => table,
					_ => panic!("Found non-tables for cargo toml bench"),
				},
			));

			document["bench"] = array.into();
		}

		if !self.test.is_empty() {
			let array =
				ArrayOfTables::from_iter(self.test.iter().map(|i| match i.as_toml_value() {
					Item::Table(table) => table,
					_ => panic!("Found non-tables for cargo toml test"),
				}));

			document["test"] = array.into();
		}

		if !self.example.is_empty() {
			let array = ArrayOfTables::from_iter(self.example.iter().map(
				|i| match i.as_toml_value() {
					Item::Table(table) => table,
					_ => panic!("Found non-tables for cargo toml examples"),
				},
			));

			document["example"] = array.into();
		}

		if let Some(lints) = &self.lints {
			document["lints"] = match lints {
				Inheritable::Workspace { workspace } => {
					Table::from_iter([("workspace", *workspace)]).into()
				}
				Inheritable::Value(lints) => lints.as_toml_value(),
			};
		}

		add_table!(self, document => dev_dependencies, build_dependencies, dependencies);

		if !self.patch.is_empty() {
			let mut table = Table::from_iter(self.patch.iter().map(|(name, deps)| {
				let mut deps_table =
					Table::from_iter(deps.iter().map(|(dep_name, dep)| {
						(toml_edit::Key::from(dep_name), dep.as_toml_value())
					}));

				deps_table.set_implicit(true);

				(toml_edit::Key::from(name), deps_table)
			}));

			table.set_implicit(true);

			document["patch"] = table.into();
		}

		if !self.features.is_empty() {
			document["features"] =
				Table::from_iter(self.features.iter().map(|(name, features)| {
					let mut array = Array::from_iter(features);
					format_array(&mut array);

					(toml_edit::Key::from(name.as_str()), array)
				}))
				.into();
		}

		document
	}
}

/// Lint level.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum LintLevel {
	Allow,
	Warn,
	Deny,
	Forbid,
}

impl AsTomlValue for LintLevel {
	fn as_toml_value(&self) -> Item {
		let str = match self {
			Self::Allow => "allow",
			Self::Warn => "warn",
			Self::Deny => "deny",
			Self::Forbid => "forbid",
		};

		str.into()
	}
}

#[track_caller]
fn item_to_toml_value(item: Item) -> Option<TomlValue> {
	let output = match item {
		Item::Value(value) => value,
		Item::Table(table) => table.into_inline_table().into(),
		Item::ArrayOfTables(arr) => arr.into_array().into(),
		_ => return None,
	};

	Some(output)
}

/// Lint definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Lint {
	/// Lint level.
	pub level: LintLevel,

	/// Controls which lints or lint groups override other lint groups.
	pub priority: Option<i8>,
}

impl AsTomlValue for Lint {
	fn as_toml_value(&self) -> Item {
		let mut table = InlineTable::new();

		if let Some(level) = item_to_toml_value(self.level.as_toml_value()) {
			table.insert("level", level);
		}

		if let Some(priority) = self.priority {
			table.insert("priority", i64::from(priority).into());
		}

		table.into()
	}
}

/// Dependencies that are platform-specific or enabled through custom `cfg()`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default, rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct Target {
	/// platform-specific normal dependencies
	pub dependencies: BTreeMap<String, Dependency>,
	/// platform-specific dev-only/test-only dependencies
	pub dev_dependencies: BTreeMap<String, Dependency>,
	/// platform-specific build-time dependencies
	pub build_dependencies: BTreeMap<String, Dependency>,
}

impl AsTomlValue for Target {
	fn as_toml_value(&self) -> Item {
		let mut table = Table::new();

		table.set_implicit(true);

		add_table!(self, table => dev_dependencies, dependencies, build_dependencies);

		table.into()
	}
}

/// Dependency definition.
///
/// It can be a simple version number, or detailed settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
pub enum Dependency {
	/// Simple version requirement (e.g. `^1.5`)
	Simple(String),

	/// A dependency inherited from the workspace.
	Inherited(InheritedDependencyDetail), // Must be placed before `detailed` to deserialize correctly

	/// A detailed dependency table.
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
	/// Checks whether the dependency is marked as optional.
	pub fn optional(&self) -> bool {
		match self {
			Self::Simple(_) => false,
			Self::Inherited(dep) => dep.optional,
			Self::Detailed(dep) => dep.optional,
		}
	}

	/// Returns the features enabled in the dependency.
	pub fn features(&self) -> Option<&BTreeSet<String>> {
		match self {
			Self::Simple(_) => None,
			Self::Inherited(dep) => Some(&dep.features),
			Self::Detailed(dep) => Some(&dep.features),
		}
	}

	pub const fn as_simple(&self) -> Option<&String> {
		if let Self::Simple(v) = self {
			Some(v)
		} else {
			None
		}
	}

	pub const fn as_inherited(&self) -> Option<&InheritedDependencyDetail> {
		if let Self::Inherited(v) = self {
			Some(v)
		} else {
			None
		}
	}

	pub const fn as_detailed(&self) -> Option<&DependencyDetail> {
		if let Self::Detailed(v) = self {
			Some(v)
		} else {
			None
		}
	}
}

impl Merge for Dependency {
	fn merge(&mut self, other: Self) {
		match self {
			Self::Simple(left_simple) => {
				match other {
					Self::Simple(right_simple) => *left_simple = right_simple,
					Self::Inherited(right_inherited) => *self = Self::Inherited(right_inherited),
					Self::Detailed(mut right_detailed) => {
						if right_detailed.version.is_none() {
							let version = mem::take(left_simple);

							right_detailed.version = Some(version);
						}

						*self = Self::Detailed(right_detailed);
					}
				};
			}
			Self::Inherited(left_inherited) => match other {
				// Merging inherited with a version is awkward, but reasonably
				// it should be converted to detailed
				Self::Simple(right_simple) => {
					let features = mem::take(&mut left_inherited.features);

					*self = Self::Detailed(
						DependencyDetail {
							version: Some(right_simple),
							optional: left_inherited.optional,
							features,
							..Default::default()
						}
						.into(),
					);
				}
				Self::Inherited(right) => left_inherited.merge(right),
				Self::Detailed(mut right) => {
					if left_inherited.optional {
						right.optional = true;
					}

					let left_features = mem::take(&mut left_inherited.features);

					right.features.extend(left_features);

					*self = Self::Detailed(right);
				}
			},
			Self::Detailed(left_detailed) => match other {
				Self::Simple(right_simple) => {
					if left_detailed.version.is_none() {
						left_detailed.version = Some(right_simple);
					}
				}
				Self::Inherited(mut right) => {
					if left_detailed.optional {
						right.optional = true;
					}

					let left_features = mem::take(&mut left_detailed.features);

					right.features.extend(left_features);

					*self = Self::Inherited(right);
				}
				Self::Detailed(right) => {
					left_detailed.merge(*right);
				}
			},
		}
	}
}

// serde helper
#[allow(clippy::trivially_copy_pass_by_ref)]
pub(crate) const fn is_false(boolean: &bool) -> bool {
	!*boolean
}

/// Describes a dependency inherited from the workspace.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, Merge)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct InheritedDependencyDetail {
	/// The features for the dependency.
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	pub features: BTreeSet<String>,

	/// Makes the dependency optional.
	#[serde(default, skip_serializing_if = "crate::is_false")]
	#[merge(with = overwrite_if_true)]
	pub optional: bool,

	// Cannot be `default` or it breaks deserialization
	/// Inherits the dependency from the workspace.
	#[serde(skip_serializing_if = "crate::is_false")]
	#[merge(with = overwrite_if_true)]
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

/// A detailed definition for a dependency.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Merge, Default)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default, rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct DependencyDetail {
	/// Semver requirement. Note that a plain version number implies this version *or newer* compatible one.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub version: Option<String>,

	/// If defined, use this as the crate name instead of `[dependencies]`'s table key.
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

	/// This path is usually relative to the crate's manifest, but when using workspace inheritance, it may be relative to the workspace.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub path: Option<String>,

	/// Read dependency from git repo URL.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub git: Option<String>,

	/// Read dependency from git branch.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub branch: Option<String>,

	/// Read dependency from git tag.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub tag: Option<String>,

	/// Read dependency from git commit.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub rev: Option<String>,

	/// Enable these features of the dependency.
	///
	/// Note that Cargo interprets `default` in a special way.
	#[serde(skip_serializing_if = "BTreeSet::is_empty")]
	#[merge(with = BTreeSet::extend)]
	pub features: BTreeSet<String>,

	/// Makes the dependency optional.\
	///
	/// NB: Not allowed at workspace level
	///
	/// If not used with `dep:` or `?/` syntax in `[features]`, this also creates an implicit feature.
	#[serde(skip_serializing_if = "crate::is_false")]
	#[merge(with = overwrite_if_true)]
	pub optional: bool,

	/// Enable the `default` set of features of the dependency (enabled by default).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub default_features: Option<bool>,
}

impl AsTomlValue for DependencyDetail {
	fn as_toml_value(&self) -> Item {
		let mut table = InlineTable::new();

		add_string!(self, table => version, path, package, registry, registry_index, git, branch, tag, rev);

		add_bool!(self, table => optional);

		add_if_false!(self, table => default_features);

		add_string_list!(self, table => features);

		table.into()
	}
}

/// A value that can be set to `{ workspace = true }`
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
pub enum Inheritable<T> {
	/// Inherit this setting from the `workspace`
	#[serde(rename = "workspace")]
	Workspace {
		workspace: bool,
	},
	Value(T),
}

impl<T> Inheritable<T> {
	pub const fn is_workspace(&self) -> bool {
		matches!(self, Self::Workspace { workspace: true })
	}

	pub const fn as_value(&self) -> Option<&T> {
		if let Self::Value(val) = self {
			Some(val)
		} else {
			None
		}
	}
}

impl<T: Merge> Merge for Inheritable<T> {
	fn merge(&mut self, other: Self) {
		match self {
			Self::Workspace { .. } => {
				*self = other;
			}
			Self::Value(content_left) => {
				match other {
					Self::Workspace { workspace } => *self = Self::Workspace { workspace },
					Self::Value(content_right) => content_left.merge(content_right),
				};
			}
		}
	}
}

impl<T: AsTomlValue> AsTomlValue for Inheritable<T> {
	fn as_toml_value(&self) -> Item {
		match self {
			Self::Workspace { workspace } => {
				InlineTable::from_iter([("workspace", *workspace)]).into()
			}
			Self::Value(value) => value.as_toml_value(),
		}
	}
}

impl<T: Default> Default for Inheritable<T> {
	fn default() -> Self {
		Self::Value(T::default())
	}
}

impl<T: Default + PartialEq> Inheritable<T> {
	/// Checks if the [`Inheritable`] is set to [`Value`](Inheritable::Value), and that the value is the default for that type.
	pub fn is_default(&self) -> bool {
		match self {
			Self::Workspace { .. } => false,
			Self::Value(v) => T::default() == *v,
		}
	}
}

/// Edition setting, which opts in to new Rust/Cargo behaviors.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
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
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
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
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
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
#[derive(Debug, Default, PartialEq, Eq, Ord, PartialOrd, Copy, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(expecting = "if there's a newer resolver, then this parser has to be updated")]
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
		match self {
			Self::V1 => "1".into(),
			Self::V2 => "2".into(),
			Self::V3 => "3".into(),
		}
	}
}

/// A library/binary/test target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord, Merge, Default)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default, rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct Product {
	/// This field points at where the crate is located, relative to the `Cargo.toml`.
	pub path: Option<String>,

	/// The name of a product is the name of the library or binary that will be generated.
	///
	/// This is defaulted to the name of the package, with any dashes replaced
	/// with underscores. (Rust `extern crate` declarations reference this name;
	/// therefore the value must be a valid Rust identifier to be usable.)
	pub name: Option<String>,

	/// A flag for enabling unit tests for this product. This is used by `cargo test`.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub test: Option<bool>,

	/// A flag for enabling documentation tests for this product. This is only relevant
	/// for libraries, it has no effect on other sections. This is used by
	/// `cargo test`.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub doctest: Option<bool>,

	/// A flag for enabling benchmarks for this product. This is used by `cargo bench`.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub bench: Option<bool>,

	/// A flag for enabling documentation of this product. This is used by `cargo doc`.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub doc: Option<bool>,

	/// If the product is meant to be a "macros 1.1" procedural macro, this field must
	/// be set to true.
	#[serde(
		alias = "proc_macro",
		alias = "proc-macro",
		skip_serializing_if = "crate::is_false"
	)]
	#[merge(with = overwrite_if_true)]
	pub proc_macro: bool,

	/// If set to false, `cargo test` will omit the `--test` flag to rustc, which
	/// stops it from generating a test harness. This is useful when the binary being
	/// built manages the test runner itself.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub harness: Option<bool>,

	/// The available options are "dylib", "rlib", "staticlib", "cdylib", and "proc-macro".
	#[serde(skip_serializing_if = "BTreeSet::is_empty")]
	pub crate_type: BTreeSet<String>,

	/// The `required-features` field specifies which features the product needs in order to be built.
	/// If any of the required features are not selected, the product will be skipped.
	/// This is only relevant for the `[[bin]]`, `[[bench]]`, `[[test]]`, and `[[example]]` sections,
	/// it has no effect on `[lib]`.
	#[serde(skip_serializing_if = "BTreeSet::is_empty")]
	pub required_features: BTreeSet<String>,
}

impl AsTomlValue for Product {
	fn as_toml_value(&self) -> Item {
		let mut table = Table::new();

		add_string!(self, table => path, name);
		add_bool!(self, table => proc_macro);
		add_string_list!(self, table => crate_type, required_features);

		add_if_false!(self, table => test, doctest, bench, doc, harness);

		table.into()
	}
}
