use std::{
  collections::{BTreeMap, BTreeSet},
  path::PathBuf,
};

use merge::Merge;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
  merge_optional_btree_maps, overwrite_if_some,
  rust::{merge_inheritable_set, Edition, Inheritable, OptionalFile, Publish, Resolver},
};

/// The `[package]` section of the [`Manifest`]. This is where crate properties are.
///
/// Note that most of these properties can be inherited from a workspace, and therefore not available just from reading a single `Cargo.toml`. See [`Manifest::inherit_workspace`].
///
/// You can replace `Metadata` generic type with your own
/// to parse into something more useful than a generic toml `Value`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, Merge)]
#[merge(strategy = overwrite_if_some)]
#[serde(rename_all = "kebab-case")]
pub struct Package {
  /// Careful: some names are uppercase, case-sensitive. `-` changes to `_` when used as a Rust identifier.
  #[merge(skip)]
  pub name: String,

  /// See [the `version()` getter for more info](`Package::version()`).
  ///
  /// Must parse as semver, e.g. "1.9.0"
  ///
  /// This field may have unknown value when using workspace inheritance,
  /// and when the `Manifest` has been loaded without its workspace.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub version: Option<Inheritable<String>>,

  /// Package's edition opt-in. Use [`Package::edition()`] to read it.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub edition: Option<Inheritable<Edition>>,

  /// MSRV 1.x (beware: does not require semver formatting)
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub rust_version: Option<Inheritable<String>>,

  /// Build script definition
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub build: Option<OptionalFile>,

  /// Workspace this package is a member of (`None` if it's implicit)
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub workspace: Option<PathBuf>,

  /// It doesn't link to anything
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub links: Option<String>,

  /// A short blurb about the package. This is not rendered in any format when
  /// uploaded to crates.io (aka this is not markdown).
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub description: Option<Inheritable<String>>,

  /// Project's homepage
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub homepage: Option<Inheritable<String>>,

  /// Path to your custom docs. Unnecssary if you rely on docs.rs.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub documentation: Option<Inheritable<String>>,

  /// This points to a file under the package root (relative to this `Cargo.toml`).
  /// implied if README.md, README.txt or README exists.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub readme: Option<Inheritable<OptionalFile>>,

  /// Up to 5, for search
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[merge(strategy= merge_inheritable_set)]
  pub keywords: Option<Inheritable<BTreeSet<String>>>,

  /// This is a list of up to five categories where this crate would fit.
  /// e.g. `["command-line-utilities", "development-tools::cargo-plugins"]`
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[merge(strategy= merge_inheritable_set)]
  pub categories: Option<Inheritable<BTreeSet<String>>>,

  /// Don't publish these files
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[merge(strategy= merge_inheritable_set)]
  pub exclude: Option<Inheritable<BTreeSet<String>>>,

  /// Publish these files
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[merge(strategy= merge_inheritable_set)]
  pub include: Option<Inheritable<BTreeSet<String>>>,

  /// e.g. "MIT"
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub license: Option<Inheritable<String>>,

  /// If `license` is not standard
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub license_file: Option<Inheritable<PathBuf>>,

  /// (HTTPS) URL to crate's repository
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub repository: Option<Inheritable<String>>,

  /// The default binary to run by cargo run.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub default_run: Option<String>,

  /// Discover binaries from the file system
  ///
  /// This may be incorrectly set to `true` if the crate uses 2015 edition and has explicit `[[bin]]` sections
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub autobins: Option<bool>,

  /// Discover libraries from the file system
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub autolib: Option<bool>,

  /// Discover examples from the file system
  ///
  /// This may be incorrectly set to `true` if the crate uses 2015 edition and has explicit `[[example]]` sections
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub autoexamples: Option<bool>,

  /// Discover tests from the file system
  ///
  /// This may be incorrectly set to `true` if the crate uses 2015 edition and has explicit `[[test]]` sections
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub autotests: Option<bool>,

  /// Discover benchmarks from the file system
  ///
  /// This may be incorrectly set to `true` if the crate uses 2015 edition and has explicit `[[bench]]` sections
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub autobenches: Option<bool>,

  /// Disable publishing or select custom registries.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub publish: Option<Inheritable<Publish>>,

  /// The feature resolver version.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub resolver: Option<Resolver>,

  /// Arbitrary metadata of any type, an extension point for 3rd party tools.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy= merge_optional_btree_maps)]
  pub metadata: Option<BTreeMap<String, Value>>,
}
