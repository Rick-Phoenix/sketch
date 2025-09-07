use std::collections::BTreeMap;

use askama::Template;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::PackageManager;

/// A struct for representing the values being used in the various configuration files for moonrepo.
/// The tasks correspond to the list of tasks in the .moon/tasks.yml config file, and the tasks_config field represents all of the other key-value pairs being used in the same file.
/// The `toolchain` field contains the key-value pairs belonging to the .moon/toolchain.yml file.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MoonConfig {
  pub tasks: Option<BTreeMap<String, Value>>,
  pub toolchain: Option<BTreeMap<String, Value>>,
  pub tasks_config: Option<BTreeMap<String, Value>>,
}

/// A struct that represents the key-value pairs being used in a .moon/toolchain.yml file.
/// The `config` field represents any top level key-value pair that can be used in the file.
#[derive(Clone, Debug, Template, Default)]
#[template(path = "moon/toolchain.yml.j2")]
pub struct MoonToolchain {
  pub root_tsconfig_name: String,
  pub package_manager: PackageManager,
  pub project_tsconfig_name: String,
  pub config: BTreeMap<String, Value>,
}

/// A struct that represents the key-value pairs being used in a .moon/tasks.yml file.
/// The `config` field represents any top level key-value pair that can be used in the file.
#[derive(Clone, Debug, Template, Default)]
#[template(path = "moon/tasks.yml.j2")]
pub struct MoonTasks {
  pub tasks: BTreeMap<String, Value>,
  pub config: BTreeMap<String, Value>,
  pub root_tsconfig_name: String,
  pub project_tsconfig_name: String,
  pub out_dir: Option<String>,
}

/// A struct that represents the key-value pairs being used in a moon.yml file.
/// The `config` field represents any top level key-value pair that can be used in the file.
#[derive(Clone, Debug, Template, Default, Serialize, Deserialize)]
#[template(path = "moon/moon.yml.j2")]
#[serde(default)]
pub struct MoonDotYml {
  pub tasks: BTreeMap<String, Value>,
  pub config: BTreeMap<String, Value>,
}
