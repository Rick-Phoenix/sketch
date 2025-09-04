#![allow(clippy::large_enum_variant)]

use std::{collections::BTreeMap, fmt::Display};

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
      repository: Repository::Workspace { workspace: true },
      description: Some(Description::Data(
        "A package that solves all of your problems...".to_string(),
      )),
      package_manager: Default::default(),
      config: Default::default(),
      publish_config: Default::default(),
      man: Default::default(),
      exports: Exports::Workspace { workspace: true },
      files: Default::default(),
      engines: Default::default(),
      maintainers: Default::default(),
      contributors: Default::default(),
      author: Author::Workspace { workspace: true },
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
  Workspace {
    workspace: bool,
  },
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
    paste::paste! {
      #[derive(Debug, Serialize, Deserialize, Clone)]
      #[serde(untagged)]
      pub enum $name {
        Workspace { workspace: bool },
        Data($data_type),
      }

      impl Default for $name {
        fn default() -> Self {
          Self::Workspace { workspace: true }
        }
      }
    }
  };
}

impl_workspace_field!(Description, String);
impl_workspace_field!(Main, String);
impl_workspace_field!(Browser, String);
impl_workspace_field!(Keywords, Vec<String>);
impl_workspace_field!(Homepage, String);
impl_workspace_field!(License, String);
impl_workspace_field!(PackageManagerJson, String);
impl_workspace_field!(Files, Vec<String>);
impl_workspace_field!(Engines, StringKeyVal);
impl_workspace_field!(EngineStrict, bool);
impl_workspace_field!(Os, Vec<String>);
impl_workspace_field!(Cpu, Vec<String>);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BugsData {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub url: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub email: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Bugs {
  Workspace { workspace: bool },
  Data(BugsData),
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, Template)]
#[template(path = "person.j2")]
pub struct Person {
  pub name: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub url: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub email: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Author {
  Workspace { workspace: bool },
  Data(Person),
}

#[derive(Clone, Debug, Serialize, Deserialize, Template)]
#[template(path = "export_path.j2")]
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
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[serde(flatten)]
    other: StringKeyVal,
  },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Exports {
  Workspace { workspace: bool },
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
#[derive(Clone)]
pub enum Contributor {
  Workspace(String),
  Data(Person),
}

#[derive(Debug, Serialize, Deserialize, Template)]
#[template(path = "man.j2")]
#[serde(untagged)]
#[derive(Clone)]
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

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct DependenciesPreset {
  pub dependencies: StringKeyVal,
  pub optional_dependencies: StringKeyVal,
  pub peer_dependencies: StringKeyVal,
  pub bundle_dependencies: StringKeyVal,
  pub dev_dependencies: StringKeyVal,
}

impl PackageJson {
  pub fn merge_dependencies_preset(&mut self, preset: DependenciesPreset) {
    self.dependencies.extend(preset.dependencies);
    self.dev_dependencies.extend(preset.dev_dependencies);
    self.peer_dependencies.extend(preset.peer_dependencies);
    self.bundle_dependencies.extend(preset.bundle_dependencies);
    self
      .optional_dependencies
      .extend(preset.optional_dependencies);
  }
}

pub(crate) fn merge_maps(left: &mut StringKeyVal, right: StringKeyVal) {
  left.extend(right)
}

#[derive(Debug, Deserialize, Serialize, Template, Merge, Clone)]
#[template(path = "package.json.j2")]
#[serde(default)]
pub struct PackageJson {
  #[merge(skip)]
  pub package_name: String,
  #[merge(strategy = merge::vec::append)]
  pub extends: Vec<String>,
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
  #[merge(skip)]
  pub metadata: StringKeyValMap,
  #[merge(skip)]
  pub repository: Repository,
  #[merge(skip)]
  pub description: Option<Description>,
  #[merge(skip)]
  pub keywords: Keywords,
  #[merge(skip)]
  pub homepage: Homepage,
  #[merge(skip)]
  pub bugs: Option<Bugs>,
  #[merge(skip)]
  pub license: License,
  #[merge(skip)]
  pub author: Author,
  #[merge(skip)]
  pub contributors: Option<Vec<Contributor>>,
  #[merge(skip)]
  pub maintainers: Option<Vec<Contributor>>,
  #[merge(skip)]
  pub files: Files,
  #[merge(skip)]
  pub exports: Exports,
  #[merge(skip)]
  pub man: Option<Man>,
  #[merge(skip)]
  pub config: Option<StringKeyValMap>,
  #[merge(skip)]
  pub package_manager: PackageManagerJson,
  #[merge(skip)]
  pub publish_config: Option<PublishConfig>,
  #[merge(skip)]
  pub engines: Engines,
  #[merge(skip)]
  pub os: Os,
  #[merge(skip)]
  pub cpu: Cpu,
  #[merge(skip)]
  pub main: Main,
  #[merge(skip)]
  pub browser: Option<Browser>,
  #[merge(skip)]
  pub workspaces: Option<Vec<String>>,
}
