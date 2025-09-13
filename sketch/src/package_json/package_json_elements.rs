use std::{collections::BTreeMap, fmt::Display};

use askama::Template;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{filters, StringBTreeMap};

/// An enum representing valid formats for the `repository` field in a package.json file.
#[derive(Debug, Serialize, Deserialize, Template, Clone, PartialEq, Eq, JsonSchema)]
#[template(path = "repository.j2")]
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

/// A struct representing the `bugs` field in a package.json file.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Template, JsonSchema)]
#[template(path = "bugs.j2")]
pub struct Bugs {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub url: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub email: Option<String>,
}

/// The possible values used for representing an individual in a package.json file, which can be used to populate the `contributors` and `maintainers` fields.
/// If a plain string is used, it will be interpreted as an id for a [`Person`] that is stored in the global config.
#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, Ord, Eq, JsonSchema)]
#[serde(untagged)]
#[derive(Clone)]
pub enum Person {
  Id(String),
  Data(PersonData),
}

/// A struct that matches how an individual's information is represented in a package.json file in the author, maintainers and contributors fields.
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
#[template(path = "person.j2")]
pub struct PersonData {
  pub name: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub url: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub email: Option<String>,
}

/// A struct matching a value in the `exports` object inside a package.json file.
#[derive(Clone, Debug, Serialize, Deserialize, Template, PartialEq, Eq, JsonSchema)]
#[template(path = "export_path.j2")]
#[serde(untagged)]
pub enum Exports {
  Path(String),
  Data {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    require: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    import: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    node: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    default: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    types: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[serde(flatten)]
    other: StringBTreeMap,
  },
}

/// A struct that matches the value of the `directories` field in a package.json file.
#[derive(Clone, Debug, Serialize, Default, Deserialize, PartialEq, Eq, Template, JsonSchema)]
#[serde(default)]
#[template(path = "directories.j2")]
pub struct Directories {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub bin: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub doc: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub example: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub lib: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub man: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub test: Option<String>,
  #[serde(skip_serializing_if = "BTreeMap::is_empty")]
  #[serde(flatten)]
  pub other: StringBTreeMap,
}

/// A struct that matches the possible values for the `man` field of a package.json file.
#[derive(Clone, Debug, Serialize, Deserialize, Template, PartialEq, Eq, JsonSchema)]
#[template(path = "man.j2")]
#[serde(untagged)]
pub enum Man {
  Path(String),
  List(Vec<String>),
}

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

/// A struct that matches the `publishConfig` field in a package.json file.
#[derive(Clone, Debug, Serialize, Deserialize, Default, Template, PartialEq, Eq, JsonSchema)]
#[serde(default)]
#[template(path = "publish_config.j2")]
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

#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy, PartialEq, Eq, JsonSchema)]
pub enum JsModuleType {
  #[serde(rename = "module")]
  #[default]
  Module,
  CommonJs,
}

impl Display for JsModuleType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      JsModuleType::Module => write!(f, "module"),
      JsModuleType::CommonJs => write!(f, "CommonJs"),
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum DepKind {
  Dependency,
  DevDependency,
  OptionalDependency,
  PeerDependency,
  BundleDependency,
}
