use std::collections::BTreeMap;

use askama::Template;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::PackageManager;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MoonConfig {
  pub tasks: Option<BTreeMap<String, Value>>,
  pub toolchain: Option<BTreeMap<String, Value>>,
  pub tasks_config: Option<BTreeMap<String, Value>>,
}

#[derive(Clone, Debug, Template, Default)]
#[template(path = "moon/toolchain.yml.j2")]
pub struct MoonToolchain {
  pub root_tsconfig_name: String,
  pub package_manager: PackageManager,
  pub project_tsconfig_name: String,
  pub config: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Template, Default)]
#[template(path = "moon/tasks.yml.j2")]
pub struct MoonTasks {
  pub tasks: BTreeMap<String, Value>,
  pub config: BTreeMap<String, Value>,
  pub root_tsconfig_name: String,
  pub project_tsconfig_name: String,
  pub out_dir: String,
}

#[derive(Clone, Debug, Template, Default, Serialize, Deserialize)]
#[template(path = "moon/moon.yml.j2")]
#[serde(default)]
pub struct MoonDotYml {
  pub tasks: BTreeMap<String, Value>,
  pub config: BTreeMap<String, Value>,
}
