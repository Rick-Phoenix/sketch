#![allow(clippy::large_enum_variant)]

use std::collections::BTreeMap;

use askama::Template;
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
      dependencies: Default::default(),
      dependencies_presets: Default::default(),
      dev_dependencies: Default::default(),
      dev_dependencies_presets: Default::default(),
      scripts: Default::default(),
      metadata: Default::default(),
      repository: Default::default(),
      description: Some(Description::Data(
        "A package that solves all of your problems...".to_string(),
      )),
      package_manager: Default::default(),
      config: Default::default(),
      publish_config: Default::default(),
      man: Default::default(),
      exports: Default::default(),
      files: Default::default(),
      engines: Default::default(),
      maintainers: Default::default(),
      contributors: Default::default(),
      author: Default::default(),
      license: Default::default(),
      engine_strict: Default::default(),
      bugs: Default::default(),
      os: Default::default(),
      cpu: Default::default(),
      keywords: Default::default(),
      homepage: Default::default(),
      entry_point: Default::default(),
      bundled_dependencies: Default::default(),
      peer_dependencies: Default::default(),
      optional_dependencies: Default::default(),
      workspaces: Default::default(),
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Default, Template)]
#[template(path = "repository.j2")]
#[serde(untagged)]
pub enum Repository {
  #[serde(rename = "workspace")]
  #[default]
  Workspace,
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

macro_rules! impl_workspace_field {
  ($name:ident, $data_type:ty) => {
    #[derive(Debug, Serialize, Deserialize, Default)]
    #[serde(untagged)]
    pub enum $name {
      #[serde(rename = "workspace")]
      #[default]
      Workspace,
      Data($data_type),
    }
  };
}

impl_workspace_field!(Description, String);
impl_workspace_field!(Keywords, Vec<String>);
impl_workspace_field!(Homepage, String);
impl_workspace_field!(License, String);
impl_workspace_field!(PackageManagerJson, String);
impl_workspace_field!(Files, Vec<String>);
impl_workspace_field!(Engines, StringKeyVal);
impl_workspace_field!(EngineStrict, bool);
impl_workspace_field!(Os, Vec<String>);
impl_workspace_field!(Cpu, Vec<String>);

#[derive(Debug, Serialize, Deserialize)]
pub struct BugsData {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub url: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(untagged)]
pub enum Bugs {
  #[serde(rename = "workspace")]
  #[default]
  Workspace,
  Data(BugsData),
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Person {
  pub name: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub url: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(untagged)]
pub enum Author {
  #[default]
  Workspace,
  Data(Person),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExportPath {
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    other: Option<StringKeyVal>,
  },
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(untagged)]
pub enum Exports {
  #[serde(rename = "workspace")]
  #[default]
  Workspace,
  Data(BTreeMap<String, ExportPath>),
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Contributor {
  Workspace(String),
  Data(Person),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Man {
  Path(String),
  List(Vec<String>),
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(untagged)]
pub enum EntryPoint {
  #[serde(rename = "workspace")]
  #[default]
  Workspace,
  Main(String),
  Browser(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PublishConfigAccess {
  Public,
  Restricted,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublishConfig {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub access: Option<PublishConfigAccess>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub tag: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub registry: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub other: Option<StringKeyVal>,
}

#[derive(Debug, Deserialize, Serialize, Template)]
#[template(path = "package.json.j2")]
#[serde(default)]
pub struct PackageJson {
  pub package_name: String,
  pub dependencies: StringKeyVal,
  pub optional_dependencies: Option<StringKeyVal>,
  pub peer_dependencies: Option<StringKeyVal>,
  pub bundled_dependencies: Option<StringKeyVal>,
  pub dependencies_presets: Vec<String>,
  pub dev_dependencies_presets: Vec<String>,
  pub dev_dependencies: StringKeyVal,
  pub scripts: StringKeyVal,
  pub metadata: StringKeyValMap,
  pub repository: Repository,
  pub description: Option<Description>,
  pub keywords: Keywords,
  pub homepage: Homepage,
  pub bugs: Option<Bugs>,
  pub license: License,
  pub author: Author,
  pub contributors: Option<Vec<Contributor>>,
  pub maintainers: Option<Vec<Contributor>>,
  pub files: Files,
  pub exports: Exports,
  pub man: Option<Man>,
  pub config: Option<StringKeyValMap>,
  pub package_manager: PackageManagerJson,
  pub publish_config: Option<PublishConfig>,
  pub engines: Engines,
  pub engine_strict: EngineStrict,
  pub os: Os,
  pub cpu: Cpu,
  #[serde(flatten)]
  pub entry_point: EntryPoint,
  pub workspaces: Option<Vec<String>>,
}
