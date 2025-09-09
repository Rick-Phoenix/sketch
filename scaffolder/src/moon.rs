use askama::Template;
use serde::{Deserialize, Serialize};

use crate::{OrderedMap, PackageManager};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MoonConfigKind {
  Bool(bool),
  Config(Box<MoonConfig>),
}

impl Default for MoonConfigKind {
  fn default() -> Self {
    Self::Config(Default::default())
  }
}

/// A struct for representing the values being used in the various configuration files for moonrepo.
/// The tasks correspond to the list of tasks in the .moon/tasks.yml config file, and the tasks_config field represents all of the other key-value pairs being used in the same file.
/// The `toolchain` field contains the key-value pairs belonging to the .moon/toolchain.yml file.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct MoonConfig {
  pub tasks: Option<MoonTasks>,
  pub toolchain: Option<MoonToolchain>,
}

/// A struct that represents the key-value pairs being used in a .moon/toolchain.yml file.
/// The `config` field represents any top level key-value pair that can be used in the file.
#[derive(Clone, Debug, Template, Serialize, Deserialize)]
#[template(path = "moon/toolchain.yml.j2")]
#[serde(default)]
pub struct MoonToolchain {
  pub root_tsconfig_name: String,
  pub package_manager: PackageManager,
  pub project_tsconfig_name: String,
  #[serde(flatten)]
  pub config: OrderedMap,
}

impl Default for MoonToolchain {
  fn default() -> Self {
    Self {
      root_tsconfig_name: "tsconfig.options.json".to_string(),
      package_manager: PackageManager::Pnpm,
      project_tsconfig_name: "tsconfig.src.json".to_string(),
      config: Default::default(),
    }
  }
}

/// A struct that represents the key-value pairs being used in a .moon/tasks.yml file.
/// The `config` field represents any top level key-value pair that can be used in the file.
#[derive(Clone, Debug, Template, Serialize, Deserialize)]
#[template(path = "moon/tasks.yml.j2")]
#[serde(default)]
pub struct MoonTasks {
  pub tasks: OrderedMap,
  pub root_tsconfig_name: String,
  pub project_tsconfig_name: String,
  pub out_dir: Option<String>,
  #[serde(flatten)]
  pub config: OrderedMap,
}

impl Default for MoonTasks {
  fn default() -> Self {
    Self {
      tasks: Default::default(),
      root_tsconfig_name: "tsconfig.options.json".to_string(),
      project_tsconfig_name: "tsconfig.src.json".to_string(),
      out_dir: Some(".out".to_string()),
      config: Default::default(),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MoonDotYmlKind {
  Bool(bool),
  Config(MoonDotYml),
}

impl Default for MoonDotYmlKind {
  fn default() -> Self {
    Self::Config(Default::default())
  }
}

/// A struct that represents the key-value pairs being used in a moon.yml file.
/// The `config` field represents any top level key-value pair that can be used in the file.
#[derive(Clone, Debug, Template, Default, Serialize, Deserialize)]
#[template(path = "moon/moon.yml.j2")]
#[serde(default)]
pub struct MoonDotYml {
  pub tasks: OrderedMap,
  #[serde(flatten)]
  pub config: OrderedMap,
}
