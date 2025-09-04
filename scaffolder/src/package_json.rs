#![allow(clippy::large_enum_variant)]

use std::{
  collections::{BTreeMap, BTreeSet},
  fmt::Display,
};

use askama::Template;
use merge::Merge;
use serde::{Deserialize, Serialize};

use crate::{rendering::render_json_val, StringKeyVal, StringKeyValMap};

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PackageJsonData {
  Named(String),
  Definition(PackageJson),
}

impl Default for PackageJsonData {
  fn default() -> Self {
    Self::Definition(PackageJson::default())
  }
}

impl Default for PackageJson {
  fn default() -> Self {
    Self {
      package_name: "my-awesome-package".to_string(),
      extends: Default::default(),
      default_deps: true,
      dependencies: Default::default(),
      dev_dependencies: Default::default(),
      scripts: Default::default(),
      metadata: Default::default(),
      repository: None,
      description: Some("A package that solves all of your problems...".to_string()),
      package_manager: Default::default(),
      config: Default::default(),
      publish_config: Default::default(),
      man: Default::default(),
      exports: Default::default(),
      files: Default::default(),
      engines: Default::default(),
      maintainers: Default::default(),
      contributors: Default::default(),
      author: None,
      license: Default::default(),
      bugs: Default::default(),
      os: Default::default(),
      cpu: Default::default(),
      keywords: Default::default(),
      homepage: Default::default(),
      main: Default::default(),
      browser: Default::default(),
      bundle_dependencies: Default::default(),
      peer_dependencies: Default::default(),
      optional_dependencies: Default::default(),
      workspaces: Default::default(),
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Template, Clone)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bugs {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub url: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, Ord, Eq)]
#[serde(untagged)]
#[derive(Clone)]
pub enum Person {
  Workspace(String),
  Data(PersonData),
}

#[derive(
  Clone, Debug, Serialize, Deserialize, Default, Template, Ord, PartialEq, PartialOrd, Eq,
)]
#[template(path = "person.j2")]
pub struct PersonData {
  pub name: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub url: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub email: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Template)]
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
    other: StringKeyVal,
  },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Directories {
  pub bin: Option<String>,
  pub doc: Option<String>,
  pub example: Option<String>,
  pub lib: Option<String>,
  pub man: Option<String>,
  pub test: Option<String>,
  pub other: Option<StringKeyVal>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Template)]
#[template(path = "man.j2")]
#[serde(untagged)]
pub enum Man {
  Path(String),
  List(Vec<String>),
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
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

#[derive(Clone, Debug, Serialize, Deserialize, Template)]
#[template(path = "publish_config.j2")]
pub struct PublishConfig {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub access: Option<PublishConfigAccess>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub tag: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub registry: Option<String>,
  #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
  pub other: StringKeyVal,
}

pub(crate) fn merge_maps<T>(left: &mut BTreeMap<String, T>, right: BTreeMap<String, T>) {
  left.extend(right)
}

pub(crate) fn merge_sets<T>(left: &mut BTreeSet<T>, right: BTreeSet<T>)
where
  T: Ord,
{
  left.extend(right)
}

pub(crate) fn overwrite_option<T>(left: &mut Option<T>, right: Option<T>) {
  if let Some(new) = right {
    *left = Some(new)
  }
}

pub(crate) fn overwrite_always<T>(left: &mut T, right: T) {
  *left = right
}

#[derive(Debug, Deserialize, Serialize, Template, Merge, Clone)]
#[template(path = "package.json.j2")]
#[serde(default)]
pub struct PackageJson {
  #[merge(skip)]
  pub package_name: String,
  #[merge(strategy = merge_sets)]
  pub extends: BTreeSet<String>,
  #[merge(strategy = merge::bool::overwrite_false)]
  pub default_deps: bool,
  #[merge(strategy = merge_maps)]
  pub dependencies: StringKeyVal,
  #[merge(strategy = merge_maps)]
  pub optional_dependencies: StringKeyVal,
  #[merge(strategy = merge_maps)]
  pub peer_dependencies: StringKeyVal,
  #[merge(strategy = merge_maps)]
  pub bundle_dependencies: StringKeyVal,
  #[merge(strategy = merge_maps)]
  pub dev_dependencies: StringKeyVal,
  #[merge(strategy = merge_maps)]
  pub scripts: StringKeyVal,
  #[merge(strategy = merge_maps)]
  pub metadata: StringKeyValMap,
  #[merge(strategy = overwrite_option)]
  pub repository: Option<Repository>,
  #[merge(strategy = overwrite_option)]
  pub description: Option<String>,
  #[merge(strategy = merge_sets)]
  pub keywords: BTreeSet<String>,
  #[merge(strategy = overwrite_option)]
  pub homepage: Option<String>,
  #[merge(strategy = overwrite_option)]
  pub bugs: Option<Bugs>,
  #[merge(strategy = overwrite_option)]
  pub license: Option<String>,
  #[merge(strategy = overwrite_option)]
  pub author: Option<Person>,
  #[merge(strategy = merge_sets)]
  pub contributors: BTreeSet<Person>,
  #[merge(strategy = merge_sets)]
  pub maintainers: BTreeSet<Person>,
  #[merge(strategy = merge_sets)]
  pub files: BTreeSet<String>,
  #[merge(strategy = merge_maps)]
  pub exports: BTreeMap<String, Exports>,
  #[merge(strategy = overwrite_option)]
  pub man: Option<Man>,
  #[merge(strategy = overwrite_option)]
  pub config: Option<StringKeyValMap>,
  #[merge(strategy = overwrite_option)]
  pub package_manager: Option<String>,
  #[merge(strategy = overwrite_option)]
  pub publish_config: Option<PublishConfig>,
  #[merge(strategy = merge_maps)]
  pub engines: StringKeyVal,
  #[merge(strategy = merge_sets)]
  pub os: BTreeSet<String>,
  #[merge(strategy = merge_sets)]
  pub cpu: BTreeSet<String>,
  #[merge(strategy = overwrite_option)]
  pub main: Option<String>,
  #[merge(strategy = overwrite_option)]
  pub browser: Option<String>,
  #[merge(strategy = overwrite_always)]
  pub workspaces: Option<Vec<String>>,
}
