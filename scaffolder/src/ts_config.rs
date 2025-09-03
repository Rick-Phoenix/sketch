use std::collections::BTreeMap;

use askama::Template;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::package::PackageKind;

#[derive(Debug, Template)]
#[template(path = "tsconfig.options.json.j2")]
pub struct RootTsConfig;

#[derive(Debug, Template)]
#[template(path = "tsconfig.dev.json.j2")]
pub struct DevTsConfig {
  pub project_tsconfig_name: String,
  pub out_dir: String,
}

#[derive(Debug, Template)]
#[template(path = "tsconfig.src.json.j2")]
pub struct SrcTsConfig {
  pub kind: TsConfigKind,
  pub out_dir: String,
}

#[derive(Debug, Template)]
#[template(path = "tsconfig.json.j2")]
pub struct TsConfig {
  pub root_tsconfig_path: String,
  pub references: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct TsConfigReference {
  pub path: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct TsConfigData {
  pub references: Vec<TsConfigReference>,
  #[serde(flatten)]
  pub extra: BTreeMap<String, Value>,
}

impl From<PackageKind> for TsConfigKind {
  fn from(value: PackageKind) -> Self {
    match value {
      PackageKind::Library => Self::Library,
      PackageKind::App => Self::App,
    }
  }
}

#[derive(Debug, Default, Copy, Clone)]
pub enum TsConfigKind {
  Root,
  App,
  #[default]
  Library,
}
