use std::{collections::BTreeMap, fmt::Display};

use askama::Template;
use figment::{
  value::{Dict, Map},
  Error, Figment, Metadata, Profile, Provider,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{moon::MoonConfig, rendering::render_json_val};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
  pub package_name: String,
  pub gitignore_additions: Vec<String>,
  pub gitignore_replacement: Option<String>,
  pub package_json: PackageJson,
  pub root_tsconfig_name: String,
  pub project_tsconfig_name: String,
  pub package_manager: PackageManager,
  pub moonrepo: Option<MoonConfig>,
  pub pre_commit: PreCommitConfig,
  pub root_dir: String,
  pub packages_dir: String,
}

#[derive(Debug, Template)]
#[template(path = "tsconfig.options.json.j2")]
pub struct RootTsConfig;

#[derive(Debug, Template)]
#[template(path = "oxlint.json.j2")]
pub struct OxlintConfig;

#[derive(Debug, Template)]
#[template(path = "pnpm-workspace.yaml.j2")]
pub struct PnpmWorkspace;

#[derive(Debug, Template)]
#[template(path = "tsconfig.json.j2")]
pub struct TsConfig {
  pub root_tsconfig_name: String,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      package_name: "my-awesome-package".to_string(),
      gitignore_additions: Default::default(),
      gitignore_replacement: Default::default(),
      package_json: Default::default(),
      package_manager: Default::default(),
      root_tsconfig_name: "tsconfig.options".to_string(),
      project_tsconfig_name: "tsconfig.dev".to_string(),
      moonrepo: None,
      pre_commit: Default::default(),
      root_dir: ".".to_string(),
      packages_dir: "packages".to_string(),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PackageManager {
  #[default]
  Pnpm,
  Npm,
  Deno,
  Bun,
}

impl Display for PackageManager {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      PackageManager::Pnpm => {
        write!(f, "pnpm")
      }
      PackageManager::Npm => {
        write!(f, "npm")
      }
      PackageManager::Deno => {
        write!(f, "deno")
      }
      PackageManager::Bun => {
        write!(f, "bun")
      }
    }
  }
}

#[derive(Debug, Template)]
#[template(path = "gitignore.j2")]
pub enum GitIgnore {
  Additions(Vec<String>),
  Replacement(String),
}

#[derive(Debug, Deserialize, Serialize, Default, Template)]
#[template(path = "package.json.j2")]
pub struct PackageJson {
  pub package_name: String,
  pub dependencies: BTreeMap<String, String>,
  pub dev_dependencies: BTreeMap<String, String>,
  pub scripts: BTreeMap<String, String>,
  pub metadata: BTreeMap<String, Value>,
  pub pnpm: Option<BTreeMap<String, Value>>,
}

#[derive(Debug, Template, Default, Serialize, Deserialize)]
#[template(path = "pre-commit-config.yaml.j2")]
pub struct PreCommitConfig {
  pub repos: Vec<PreCommitRepo>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PreCommitRepo {
  pub path: String,
  pub rev: Option<String>,
  pub hooks: Vec<Value>,
}

impl Config {
  // Allow the configuration to be extracted from any `Provider`.
  fn from<T: Provider>(provider: T) -> Result<Config, Error> {
    Figment::from(provider).extract()
  }

  // Provide a default provider, a `Figment`.
  pub fn figment() -> Figment {
    use figment::providers::Env;

    // In reality, whatever the library desires.
    Figment::from(Config::default()).merge(Env::prefixed("APP_"))
  }
}

// Make `Config` a provider itself for composability.
impl Provider for Config {
  fn metadata(&self) -> Metadata {
    Metadata::named("Library Config")
  }

  fn data(&self) -> Result<Map<Profile, Dict>, Error> {
    figment::providers::Serialized::defaults(Config::default()).data()
  }
}
