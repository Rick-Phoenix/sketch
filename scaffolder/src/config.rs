use std::{collections::BTreeMap, fmt::Display};

use askama::Template;
use figment::{
  value::{Dict, Map},
  Error, Figment, Metadata, Profile, Provider,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
  moon::MoonConfig,
  package::{vitest::VitestConfigStruct, PackageConfig},
  tera::TemplateOutput,
  PackageJson, PackageJsonData, Person, PersonData, TsConfig,
};

#[derive(Debug, Deserialize, Serialize, Default, Clone, Copy)]
pub enum VersionRange {
  Patch,
  #[default]
  Minor,
  Exact,
}

impl VersionRange {
  pub fn create(&self, version: String) -> String {
    match self {
      VersionRange::Patch => format!("~{}", version),
      VersionRange::Minor => format!("^{}", version),
      VersionRange::Exact => version,
    }
  }
}

impl Config {
  pub fn get_contributor(&self, name: &str) -> Option<Person> {
    self
      .people
      .get(name)
      .map(|person| Person::Data(person.clone()))
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
  pub package_name: String,
  pub gitignore: GitIgnore,
  pub pnpm_config: BTreeMap<String, Value>,
  pub root_package_json: PackageJsonData,
  pub package_json_presets: BTreeMap<String, PackageJson>,
  pub tsconfig_presets: BTreeMap<String, TsConfig>,
  pub root_tsconfig_name: String,
  pub project_tsconfig_name: String,
  pub dev_tsconfig_name: String,
  pub package_manager: PackageManager,
  pub moonrepo: Option<MoonConfig>,
  pub pre_commit: PreCommitConfig,
  pub root_dir: String,
  pub package_dirs: Vec<String>,
  pub package_presets: BTreeMap<String, PackageConfig>,
  pub vitest_presets: BTreeMap<String, VitestConfigStruct>,
  pub catalog: bool,
  pub version_ranges: VersionRange,
  pub out_dir: String,
  pub people: BTreeMap<String, PersonData>,
  pub templates_dir: Option<String>,
  pub templates: BTreeMap<String, String>,
  pub global_templates_vars: BTreeMap<String, Value>,
  pub generate_root_templates: Vec<TemplateOutput>,
}

#[derive(Debug, Template)]
#[template(path = "oxlint.json.j2")]
pub struct OxlintConfig;

impl Default for Config {
  fn default() -> Self {
    Self {
      package_name: "my-awesome-package".to_string(),
      gitignore: Default::default(),
      package_json_presets: Default::default(),
      root_package_json: Default::default(),
      package_manager: Default::default(),
      root_tsconfig_name: "tsconfig.options.json".to_string(),
      project_tsconfig_name: "tsconfig.src.json".to_string(),
      dev_tsconfig_name: "tsconfig.dev.json".to_string(),
      moonrepo: None,
      pre_commit: Default::default(),
      root_dir: ".".to_string(),
      package_dirs: vec!["packages".to_string()],
      package_presets: {
        let mut map: BTreeMap<String, PackageConfig> = BTreeMap::new();
        map.insert("default".to_string(), PackageConfig::default());
        map
      },
      vitest_presets: {
        let mut map: BTreeMap<String, VitestConfigStruct> = BTreeMap::new();
        map.insert("default".to_string(), VitestConfigStruct::default());
        map
      },
      catalog: true,
      version_ranges: Default::default(),
      tsconfig_presets: Default::default(),
      out_dir: ".out".to_string(),
      people: Default::default(),
      pnpm_config: Default::default(),
      templates_dir: Default::default(),
      templates: Default::default(),
      global_templates_vars: Default::default(),
      generate_root_templates: Default::default(),
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
  Yarn,
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
      PackageManager::Yarn => {
        write!(f, "yarn")
      }
    }
  }
}

#[derive(Clone, Debug, Template, Serialize, Deserialize)]
#[template(path = "gitignore.j2")]
#[serde(untagged)]
pub enum GitIgnore {
  Additions(Vec<String>),
  Replacement(String),
}

impl Default for GitIgnore {
  fn default() -> Self {
    Self::Additions(Default::default())
  }
}

#[derive(Clone, Debug, Template, Default, Serialize, Deserialize)]
#[template(path = "pre-commit-config.yaml.j2")]
pub struct PreCommitConfig {
  pub repos: Vec<PreCommitRepo>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
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
