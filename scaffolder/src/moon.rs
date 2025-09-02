use std::collections::BTreeMap;

use askama::Template;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::PackageManager;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct MoonConfig {
  pub tasks: Option<BTreeMap<String, Value>>,
  pub toolchain: Option<BTreeMap<String, Value>>,
  pub tasks_config: Option<BTreeMap<String, Value>>,
}

#[derive(Debug, Template, Default)]
#[template(path = "moon/toolchain.yml.j2")]
pub struct MoonToolchain {
  pub root_tsconfig_name: String,
  pub package_manager: PackageManager,
  pub project_tsconfig_name: String,
  pub config: BTreeMap<String, Value>,
}

#[derive(Debug, Template, Default)]
#[template(path = "moon/tasks.yml.j2")]
pub struct MoonTasks {
  pub root_tsconfig_name: String,
  pub tasks: BTreeMap<String, Value>,
  pub config: BTreeMap<String, Value>,
}

#[derive(Debug, Template, Default)]
#[template(path = "moon/moon.yml.j2")]
pub struct MoonDotYml {
  pub tasks: BTreeMap<String, Value>,
  pub config: BTreeMap<String, Value>,
}
