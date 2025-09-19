use std::{collections::BTreeMap, fmt::Display};

use askama::Template;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{filters, JsonValueBTreeMap, StringBTreeMap};

/// When a user installs your package, warnings are emitted if packages specified in "peerDependencies" are not already installed. The "peerDependenciesMeta" field serves to provide more information on how your peer dependencies are utilized. Most commonly, it allows peer dependencies to be marked as optional. Metadata for this field is specified with a simple hash of the package name to a metadata object.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Default, Template)]
#[template(path = "ts/package_json/peer_dependencies_meta.j2")]
#[serde(default)]
pub struct PeerDependencyMeta {
  /// Specifies that this peer dependency is optional and should not be installed automatically.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub optional: Option<bool>,

  #[serde(flatten, skip_serializing_if = "BTreeMap::is_empty")]
  pub extras: JsonValueBTreeMap,
}

/// You can specify an object containing a URL that provides up-to-date information about ways to help fund development of your package, a string URL, or an array of objects and string URLs.
#[derive(Debug, Serialize, Deserialize, Template, Clone, PartialEq, Eq, JsonSchema)]
#[template(path = "ts/package_json/funding.j2")]
#[serde(untagged)]
pub enum Funding {
  Url(String),
  Data(FundingData),
  List(Vec<Funding>),
}

/// Used to inform about ways to help fund development of the package.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Template)]
#[template(path = "ts/package_json/funding_data.j2")]
pub struct FundingData {
  /// The type of funding or the platform through which funding can be provided, e.g. patreon, opencollective, tidelift or github
  #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
  pub type_: Option<String>,

  pub url: String,
}

/// The single path for this package's binary, or a map of several binaries.
#[derive(Debug, Serialize, Deserialize, Template, Clone, PartialEq, Eq, JsonSchema)]
#[template(path = "ts/package_json/bin.j2")]
#[serde(untagged)]
pub enum Bin {
  Single(String),
  Map(StringBTreeMap),
}

/// An enum representing formats for the `repository` field in a `package.json` file.
#[derive(Debug, Serialize, Deserialize, Template, Clone, PartialEq, Eq, JsonSchema)]
#[template(path = "ts/package_json/repository.j2")]
#[serde(untagged)]
pub enum Repository {
  Path(String),
  Data {
    #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
    type_: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    directory: Option<String>,
  },
}

/// A struct representing the `bugs` field in a `package.json` file.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Template, JsonSchema)]
#[template(path = "ts/package_json/bugs.j2")]
pub struct Bugs {
  /// The url to your project's issue tracker.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub url: Option<String>,

  /// The email address to which issues should be reported.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub email: Option<String>,
}

/// The kinds of values used for representing an individual in a `package.json` file, which can be used to populate the `contributors` and `maintainers` fields.
/// If a plain string is used, it will be interpreted as an id for a [`PersonData`] that is stored in the global config.
#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, Ord, Eq, JsonSchema)]
#[serde(untagged)]
#[derive(Clone)]
pub enum Person {
  Id(String),
  Data(PersonData),
}

/// A struct that represents how an individual's information is represented in a `package.json` file in the author, maintainers and contributors fields.
#[derive(
  Clone,
  Debug,
  Serialize,
  Deserialize,
  Default,
  Template,
  Ord,
  PartialEq,
  PartialOrd,
  Eq,
  JsonSchema,
)]
#[template(path = "ts/package_json/person.j2")]
pub struct PersonData {
  pub name: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub email: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub url: Option<String>,
}

/// A struct that represents a value in the `exports` object inside a `package.json` file.
#[derive(Clone, Debug, Serialize, Deserialize, Template, PartialEq, Eq, JsonSchema)]
#[template(path = "ts/package_json/export_path.j2")]
#[serde(untagged)]
pub enum Exports {
  Path(String),
  Data {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    types: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    import: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    require: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    node: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[serde(flatten)]
    other: StringBTreeMap,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    default: Option<String>,
  },
}

/// A struct that represents the value of the `directories` field in a `package.json` file.
#[derive(Clone, Debug, Serialize, Default, Deserialize, PartialEq, Eq, Template, JsonSchema)]
#[serde(default)]
#[template(path = "ts/package_json/directories.j2")]
pub struct Directories {
  /// If you specify a `bin` directory, then all the files in that folder will be used as the `bin` hash.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub bin: Option<String>,

  /// Tell people where the bulk of your library is. Nothing special is done with the lib folder in any way, but it's useful meta info.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub lib: Option<String>,

  /// Put markdown files in here. Eventually, these will be displayed nicely, maybe, someday.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub doc: Option<String>,

  /// Put example scripts in here. Someday, it might be exposed in some clever way.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub example: Option<String>,

  /// A folder that is full of man pages. Sugar to generate a 'man' array by walking the folder.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub man: Option<String>,

  /// The tests directory.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub test: Option<String>,

  #[serde(skip_serializing_if = "BTreeMap::is_empty")]
  #[serde(flatten)]
  pub other: StringBTreeMap,
}

/// A struct that represents the kinds of values for the `man` field of a `package.json` file.
#[derive(Clone, Debug, Serialize, Deserialize, Template, PartialEq, Eq, JsonSchema)]
#[template(path = "ts/package_json/man.j2")]
#[serde(untagged)]
pub enum Man {
  Path(String),
  List(Vec<String>),
}

/// The values that can be used to define `access` in a [`PublishConfig`]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum PublishConfigAccess {
  Public,
  Restricted,
}

impl Display for PublishConfigAccess {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Public => write!(f, "public"),
      Self::Restricted => write!(f, "restricted"),
    }
  }
}

/// A set of config values that will be used at publish-time. It's especially handy if you want to set the tag, registry or access, so that you can ensure that a given package is not tagged with "latest", published to the global public registry or that a scoped module is private by default.
#[derive(Clone, Debug, Serialize, Deserialize, Default, Template, PartialEq, Eq, JsonSchema)]
#[serde(default)]
#[template(path = "ts/package_json/publish_config.j2")]
pub struct PublishConfig {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub access: Option<PublishConfigAccess>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tag: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub registry: Option<String>,
  #[serde(skip_serializing_if = "BTreeMap::is_empty")]
  #[serde(flatten)]
  pub other: StringBTreeMap,
}

/// The type of JS package.
#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy, PartialEq, Eq, JsonSchema)]
pub enum JsPackageType {
  #[serde(rename = "module")]
  #[default]
  Module,
  CommonJs,
}

impl Display for JsPackageType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      JsPackageType::Module => write!(f, "module"),
      JsPackageType::CommonJs => write!(f, "CommonJs"),
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum DepKind {
  Dependency,
  DevDependency,
  OptionalDependency,
  PeerDependency,
}
