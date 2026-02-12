use merge_it::*;
#[cfg(feature = "schemars")]
use schemars::{JsonSchema, JsonSchema_repr};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::{BTreeMap, BTreeSet};
use std::mem;
use std::path::PathBuf;

// Some of the code for this module comes from the [`cargo_toml`](https://docs.rs/cargo_toml/0.22.3/cargo_toml/index.html) crate.

macro_rules! prop_name {
	($name:ident) => {
		&stringify!($name).replace("_", "-")
	};
}

macro_rules! add_if_false {
	($target:ident, $table:ident => $($names:ident),*) => {
		$(
			if $target.$names.is_some_and(|v| !v) {
				$table.insert(prop_name!($names), false.into());
			}
		)*
	};
}

macro_rules! add_string {
	($target:ident, $table:ident => $($names:ident),*) => {
		$(
			if let Some(str) = &$target.$names {
				$table.insert(prop_name!($names), str.into());
			}
		)*
	};
}

macro_rules! add_bool {
	($target:ident, $table:ident => $($names:ident),*) => {
		$(
			if $target.$names {
				$table.insert(prop_name!($names), true.into());
			}
		)*
	};
}

macro_rules! add_optional_bool {
	($target:ident, $table:ident => $($names:ident),*) => {
		$(
			if let Some(bool) = $target.$names {
				$table.insert(prop_name!($names), bool.into());
			}
		)*
	};
}

macro_rules! add_value {
	($target:ident, $table:ident => $($names:ident),*) => {
		$(
			if let Some(val) = &$target.$names {
				$table.insert(prop_name!($names), val.as_toml_value().into());
			}
		)*
	};
}

macro_rules! add_table {
	($target:ident, $table:ident => $($names:ident),*) => {
		$(
			if !$target.$names.is_empty() {
				let mut table = Table::from_iter(
					$target.$names.iter().map(
						|(k, v)| (toml_edit::Key::from(k), Item::from(v.as_toml_value()))
					)
				);

				table.set_implicit(true);
				$table.insert(prop_name!($names), table.into());
			}
		)*
	};
}

macro_rules! add_string_list {
	($target:ident, $table:ident => $($names:ident),*) => {
		$(
			if !$target.$names.is_empty() {
				let mut array = Array::from_iter(&$target.$names);

				format_array(&mut array);

				$table.insert(prop_name!($names), array.into());
			}
		)*
	};
}

pub fn json_to_toml(json: &Value) -> Option<Item> {
	match json {
		Value::Null => None,

		Value::Bool(b) => Some(Item::Value(TomlValue::from(*b))),

		Value::Number(n) => {
			if let Some(i) = n.as_i64() {
				Some(Item::Value(TomlValue::from(i)))
			} else if let Some(f) = n.as_f64() {
				Some(Item::Value(TomlValue::from(f)))
			} else {
				Some(Item::Value(TomlValue::from(n.to_string())))
			}
		}

		Value::String(s) => Some(Item::Value(TomlValue::from(s))),

		Value::Array(vec) => {
			if vec.is_empty() {
				return Some(Item::Value(TomlValue::Array(Array::new())));
			}

			let all_objects = vec.iter().all(|v| v.is_object());

			if all_objects {
				// CASE A: [[bin]] style (Array of Tables)
				let mut array_of_tables = ArrayOfTables::new();

				for val in vec {
					// We know it's an object, so we force conversion to a standard Table
					if let Some(table) = json_to_standard_table(val) {
						array_of_tables.push(table);
					}
				}
				Some(Item::ArrayOfTables(array_of_tables))
			} else {
				// CASE B: features = ["a", "b"] style (Inline Array)
				let mut arr = Array::new();
				for val in vec {
					if let Some(item) = json_to_toml(val) {
						match item {
							Item::Value(v) => arr.push(v),
							Item::Table(t) => {
								// Inline arrays can't hold standard tables, convert to inline
								let mut inline = t.into_inline_table();
								InlineTable::fmt(&mut inline);
								arr.push(TomlValue::InlineTable(inline));
							}
							_ => {} // formatting error or invalid structure
						}
					}
				}

				format_array(&mut arr);
				Some(Item::Value(TomlValue::Array(arr)))
			}
		}

		Value::Object(_) => json_to_item_table(json),
	}
}

/// Used specifically for populating ArrayOfTables
fn json_to_standard_table(json: &Value) -> Option<Table> {
	if let Value::Object(map) = json {
		let mut table = Table::new();
		table.set_implicit(true);
		for (k, v) in map {
			if let Some(item) = json_to_toml(v) {
				table.insert(k, item);
			}
		}
		Some(table)
	} else {
		None
	}
}

/// Helper to decide between InlineTable vs Standard Table (for single objects)
fn json_to_item_table(json: &Value) -> Option<Item> {
	if let Value::Object(map) = json {
		// 1. Dependency Heuristic
		let is_dependency =
			map.contains_key("version") || map.contains_key("git") || map.contains_key("path");

		// 2. Complexity Heuristic
		let has_nested_objects = map.values().any(|v| v.is_object());
		let is_small = map.len() <= 3;

		if is_dependency || (is_small && !has_nested_objects) {
			// Inline Table: { version = "1.0" }
			let mut inline = InlineTable::new();
			for (k, v) in map {
				// We need values, not Items, for InlineTable
				if let Some(Item::Value(val)) = json_to_toml(v) {
					inline.insert(k, val);
				}
			}
			InlineTable::fmt(&mut inline);
			Some(Item::Value(TomlValue::InlineTable(inline)))
		} else {
			// Standard Table: [section]
			json_to_standard_table(json).map(Item::Table)
		}
	} else {
		None
	}
}

mod package;
pub use package::*;
mod profile_settings;
pub use profile_settings::*;
mod workspace;
pub use workspace::*;

use toml_edit::{Array, ArrayOfTables, DocumentMut, InlineTable, Item, Table, Value as TomlValue};

pub fn toml_string_list<'a>(strings: impl IntoIterator<Item = &'a String>) -> Item {
	let mut arr = Array::from_iter(strings);

	format_array(&mut arr);

	arr.into()
}

pub fn format_array(arr: &mut Array) {
	const MAX_INLINE_ITEMS: usize = 4;
	const MAX_INLINE_CHARS: usize = 50;

	let count = arr.len();

	let total_chars: usize = arr
		.iter()
		.map(|item| item.to_string().len())
		.sum();

	let has_tables = arr.iter().any(|item| item.is_inline_table());

	let should_expand = count > MAX_INLINE_ITEMS || total_chars > MAX_INLINE_CHARS || has_tables;

	if should_expand {
		for item in arr.iter_mut() {
			item.decor_mut().set_prefix("\n\t");
		}

		arr.set_trailing_comma(true);

		arr.set_trailing("\n");
	} else {
		arr.fmt();
	}
}

/// The top-level `Cargo.toml` structure. **This is the main type in this library.**
///
/// The `Metadata` is a generic type for `[package.metadata]` table. You can replace it with
/// your own struct type if you use the metadata and don't want to use the catch-all `Value` type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Merge, Default)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct Manifest {
	/// Workspace-wide settings
	#[merge(with = merge_options)]
	pub workspace: Option<Workspace>,

	/// Package definition (a cargo crate)
	#[merge(with = merge_options)]
	pub package: Option<Package>,

	/// Note that due to autolibs feature this is not the complete list
	/// unless you run [`Manifest::complete_from_path`]
	#[merge(with = merge_options)]
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub lib: Option<Product>,

	/// Note that due to autobins feature this is not the complete list
	/// unless you run [`Manifest::complete_from_path`]
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	pub bin: BTreeSet<Product>,

	/// `[target.cfg.dependencies]`
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	pub target: BTreeMap<String, Target>,

	/// `[patch.crates-io]` section
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

	/// The `[features]` section. This set may be incomplete!
	///
	/// Optional dependencies may create implied Cargo features.
	/// This features section also supports microsyntax with `dep:`, `/`, and `?`
	/// for managing dependencies and their features.io
	///
	/// This crate has an optional [`features`] module for dealing with this
	/// complexity and getting the real list of features.
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	pub features: BTreeMap<String, BTreeSet<String>>,

	/// Normal dependencies
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	#[merge(with = merge_btree_maps)]
	pub dependencies: BTreeMap<String, Dependency>,

	/// Dev/test-only deps
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	#[merge(with = merge_btree_maps)]
	pub dev_dependencies: BTreeMap<String, Dependency>,

	/// Build-time deps
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	#[merge(with = merge_btree_maps)]
	pub build_dependencies: BTreeMap<String, Dependency>,
}

impl Manifest {
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
		add_table!(self, document => target);

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
fn item_to_toml_value(item: Item) -> TomlValue {
	match item {
		Item::Value(value) => value,
		Item::Table(table) => table.into_inline_table().into(),
		Item::ArrayOfTables(arr) => arr.into_array().into(),
		_ => panic!("Failed to convert item to value"),
	}
}

/// Lint definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Lint {
	/// allow/warn/deny
	pub level: LintLevel,

	/// Controls which lints or lint groups override other lint groups.
	pub priority: Option<i8>,
}

impl AsTomlValue for Lint {
	fn as_toml_value(&self) -> Item {
		let mut table = InlineTable::new();

		table.insert("level", item_to_toml_value(self.level.as_toml_value()));

		if let Some(priority) = self.priority {
			table.insert("priority", i64::from(priority).into());
		}

		table.into()
	}
}

/// Dependencies that are platform-specific or enabled through custom `cfg()`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
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

		table.set_implicit(true);

		add_table!(self, table => dependencies, dev_dependencies, build_dependencies);

		table.into()
	}
}

/// Dependency definition. Note that this struct doesn't carry it's key/name, which you need to read from its section.
///
/// It can be simple version number, or detailed settings, or inherited.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
pub enum Dependency {
	/// Version requirement (e.g. `^1.5`)
	Simple(String),

	/// Incomplete data
	Inherited(InheritedDependencyDetail), // Must be placed first to deserialize correctly

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
			Self::Simple(_) => {
				*self = other;
			}
			Self::Inherited(left_options) => match other {
				Self::Simple(_) => *self = other,
				Self::Inherited(right) => left_options.merge(right),
				Self::Detailed(mut right) => {
					if right.optional {
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

#[allow(clippy::trivially_copy_pass_by_ref)]
pub(crate) const fn is_false(boolean: &bool) -> bool {
	!*boolean
}

/// When a dependency is defined as `{ workspace = true }`,
/// and workspace data hasn't been applied yet.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, Merge)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct InheritedDependencyDetail {
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	pub features: BTreeSet<String>,

	#[serde(default, skip_serializing_if = "crate::is_false")]
	#[merge(with = overwrite_if_true)]
	pub optional: bool,

	// Cannot be `default` or it breaks deserialization
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

/// When definition of a dependency is more than just a version string.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Merge, Default)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default, rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
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
	#[serde(skip_serializing_if = "BTreeSet::is_empty")]
	#[merge(with = BTreeSet::extend)]
	pub features: BTreeSet<String>,

	/// NB: Not allowed at workspace level
	///
	/// If not used with `dep:` or `?/` syntax in `[features]`, this also creates an implicit feature.
	/// See the [`features`] module for more info.
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

		add_string!(self, table => version, package, registry, registry_index, path, git, branch, tag, rev);

		add_string_list!(self, table => features);

		add_bool!(self, table => optional);

		add_if_false!(self, table => default_features);

		table.into()
	}
}

/// A value that can be set to `workspace`
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
			Self::Value(set) => set.as_toml_value(),
		}
	}
}

pub trait AsTomlValue {
	fn as_toml_value(&self) -> Item;
}

impl<T: Into<Item> + Clone> AsTomlValue for T {
	fn as_toml_value(&self) -> Item {
		self.clone().into()
	}
}

impl<T: Default> Default for Inheritable<T> {
	fn default() -> Self {
		Self::Value(T::default())
	}
}

impl<T: Default + PartialEq> Inheritable<T> {
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
		match self {
			Self::V1 => "1".into(),
			Self::V2 => "2".into(),
			Self::V3 => "3".into(),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord, Merge, Default)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default, rename_all = "kebab-case")]
/// Cargo uses the term "target" for both "target platform" and "build target" (the thing to build),
/// which makes it ambigous.
/// Here Cargo's bin/lib **target** is renamed to **product**.
#[serde(deny_unknown_fields)]
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
	#[merge(with = overwrite_if_true)]
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
	pub crate_type: BTreeSet<String>,

	/// The `required-features` field specifies which features the product needs in order to be built.
	/// If any of the required features are not selected, the product will be skipped.
	/// This is only relevant for the `[[bin]]`, `[[bench]]`, `[[test]]`, and `[[example]]` sections,
	/// it has no effect on `[lib]`.
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
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
