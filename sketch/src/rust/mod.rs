//! Some of the code for this module comes from the [`cargo_toml`](https://docs.rs/cargo_toml/0.22.3/cargo_toml/index.html) crate.
pub mod package;
pub mod profile_settings;
pub mod workspace;

use std::{
  collections::{BTreeMap, BTreeSet},
  path::PathBuf,
};

use indexmap::{IndexMap, IndexSet};
use merge::Merge;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
  merge_btree_maps, merge_btree_sets, merge_index_sets, merge_nested, merge_optional_nested,
  merge_presets, overwrite_if_some,
  rust::{package::Package, profile_settings::Profiles, workspace::Workspace},
  serde_utils::StringOrNum,
  Extensible, GenError, Preset,
};

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
  pub fn process_data(
    self,
    id: &str,
    store: &IndexMap<String, CargoTomlPreset>,
  ) -> Result<Manifest, GenError> {
    if self.extends_presets.is_empty() {
      return Ok(self.config);
    }

    let mut processed_ids: IndexSet<String> = IndexSet::new();

    let merged_preset = merge_presets(Preset::CargoToml, id, self, store, &mut processed_ids)?;

    Ok(merged_preset.config)
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
  #[merge(strategy = merge_inheritable_map)]
  pub lints: Option<Inheritable<BTreeMap<String, BTreeMap<String, Lint>>>>,

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
  #[merge(strategy = merge_btree_maps)]
  pub dependencies: BTreeMap<String, Dependency>,

  /// Dev/test-only deps
  #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
  #[merge(strategy = merge_btree_maps)]
  pub dev_dependencies: BTreeMap<String, Dependency>,

  /// Build-time deps
  #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
  #[merge(strategy = merge_btree_maps)]
  pub build_dependencies: BTreeMap<String, Dependency>,
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

/// Lint definition.
#[derive(Debug, Clone, PartialEq, JsonSchema, Serialize, Deserialize)]
pub struct Lint {
  /// allow/warn/deny
  pub level: LintLevel,

  /// Controls which lints or lint groups override other lint groups.
  pub priority: i8,

  /// Unstable
  pub config: BTreeMap<String, Value>,
}

/// Dependencies that are platform-specific or enabled through custom `cfg()`.
#[derive(Debug, Clone, PartialEq, Default, JsonSchema, Serialize, Deserialize)]
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

/// Dependency definition. Note that this struct doesn't carry it's key/name, which you need to read from its section.
///
/// It can be simple version number, or detailed settings, or inherited.
#[derive(Debug, Clone, PartialEq, JsonSchema, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
  /// Version requirement (e.g. `^1.5`)
  Simple(String),
  /// Incomplete data
  Inherited(InheritedDependencyDetail), // order is important for serde
  /// `{ version = "^1.5", features = ["a", "b"] }` etc.
  Detailed(Box<DependencyDetail>),
}

/// When a dependency is defined as `{ workspace = true }`,
/// and workspace data hasn't been applied yet.
#[derive(Debug, Clone, PartialEq, Eq, Default, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct InheritedDependencyDetail {
  #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
  pub features: BTreeSet<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub optional: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub workspace: Option<bool>,
}

/// When definition of a dependency is more than just a version string.
#[derive(Debug, Clone, PartialEq, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
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

  /// If true, the dependency has been defined at the workspace level, so the `path` is joined with workspace's base path.
  ///
  /// This is a field added by this crate, does not exist in TOML.
  /// Note that `Dependency::Simple` won't have this flag, even if it was inherited.
  #[serde(skip)]
  pub inherited: bool,

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
  pub features: BTreeSet<String>,

  /// NB: Not allowed at workspace level
  ///
  /// If not used with `dep:` or `?/` syntax in `[features]`, this also creates an implicit feature.
  /// See the [`features`] module for more info.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub optional: Option<bool>,

  /// Enable the `default` set of features of the dependency (enabled by default).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub default_features: Option<bool>,

  /// Contains the remaining unstable keys and values for the dependency.
  #[serde(flatten)]
  pub unstable: BTreeMap<String, Value>,
}

/// A value that can be set to `workspace`
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(untagged)]
pub enum Inheritable<T> {
  /// Inherit this setting from the `workspace`
  #[serde(rename = "workspace")]
  Workspace {
    workspace: Option<bool>,
  },
  Set(T),
}

pub(crate) fn merge_inheritable_set<T: Ord>(
  left: &mut Option<Inheritable<BTreeSet<T>>>,
  right: Option<Inheritable<BTreeSet<T>>>,
) {
  if let Some(right) = right {
    if let Some(left) = left {
      match left {
        Inheritable::Workspace { .. } => {
          *left = right;
        }
        Inheritable::Set(left_list) => {
          match right {
            Inheritable::Workspace { workspace } => *left = Inheritable::Workspace { workspace },
            Inheritable::Set(right_list) => left_list.extend(right_list),
          };
        }
      }
    } else {
      *left = Some(right);
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
            Inheritable::Workspace { workspace } => *left = Inheritable::Workspace { workspace },
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
      Inheritable::Workspace { .. } => false,
      Inheritable::Set(v) => T::default() == *v,
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

/// The feature resolver version.
///
/// Needed in [`Workspace`], but implied by [`Edition`] in packages.
#[derive(
  Debug, Default, PartialEq, Eq, Ord, PartialOrd, Copy, Clone, Serialize, Deserialize, JsonSchema,
)]
#[serde(
  expecting = "if there's a newer resolver, then this parser (cargo_toml crate) has to be updated"
)]
pub enum Resolver {
  #[serde(rename = "1")]
  #[default]
  /// The default for editions prior to 2021.
  V1 = 1,
  /// The default for the 2021 edition.
  #[serde(rename = "2")]
  V2 = 2,
  /// The default for the 2024 edition.
  #[serde(rename = "3")]
  V3 = 3,
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

  /// If the product is meant to be a compiler plugin, this field must be set to true
  /// for Cargo to correctly compile it and make it available for all dependencies.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub plugin: Option<bool>,

  /// If the product is meant to be a "macros 1.1" procedural macro, this field must
  /// be set to true.
  #[serde(
    default,
    alias = "proc_macro",
    alias = "proc-macro",
    skip_serializing_if = "Option::is_none"
  )]
  pub proc_macro: Option<bool>,

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
