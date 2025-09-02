use std::collections::BTreeMap;

use askama::Template;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::rendering::render_json_val;

#[derive(Debug, Template)]
#[template(path = "tsconfig.options.json.j2")]
pub struct RootTsConfig;

#[derive(Debug, Template)]
#[template(path = "tsconfig.json.j2")]
pub struct TsConfig {
  pub root_tsconfig_path: String,
}

impl Default for PackageJson {
  fn default() -> Self {
    Self {
      package_name: "my-awesome-package".to_string(),
      dependencies: Default::default(),
      dev_dependencies: Default::default(),
      scripts: Default::default(),
      metadata: Default::default(),
      pnpm: Default::default(),
    }
  }
}

#[derive(Debug, Deserialize, Serialize, Template)]
#[template(path = "package.json.j2")]
#[serde(default)]
pub struct PackageJson {
  pub package_name: String,
  pub dependencies: BTreeMap<String, String>,
  pub dev_dependencies: BTreeMap<String, String>,
  pub scripts: BTreeMap<String, String>,
  pub metadata: BTreeMap<String, Value>,
  pub pnpm: Option<BTreeMap<String, Value>>,
}
