use std::{
  collections::{BTreeMap, BTreeSet},
  fmt::Display,
};

use askama::Template;
use figment::{
  providers::{Format, Toml, Yaml},
  value::{Dict, Map},
  Error, Figment, Metadata, Profile, Provider,
};
use merge::Merge;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
  merge_maps, merge_sets,
  moon::MoonConfig,
  package::{vitest::VitestConfigStruct, PackageConfig},
  tera::TemplateOutput,
  GenError, PackageJson, PackageJsonData, Person, PersonData, TsConfig,
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

#[derive(Clone, Debug, Deserialize, Serialize, Merge)]
#[serde(default)]
pub struct Config {
  #[merge(skip)]
  pub root_package_json: PackageJsonData,
  #[merge(skip)]
  pub gitignore: GitIgnore,
  #[merge(strategy = merge_maps)]
  pub pnpm_config: BTreeMap<String, Value>,
  #[merge(strategy = merge_maps)]
  pub package_json_presets: BTreeMap<String, PackageJson>,
  #[merge(strategy = merge_maps)]
  pub tsconfig_presets: BTreeMap<String, TsConfig>,
  #[merge(skip)]
  pub root_tsconfig_name: String,
  #[merge(skip)]
  pub project_tsconfig_name: String,
  #[merge(skip)]
  pub dev_tsconfig_name: String,
  #[merge(skip)]
  pub package_manager: PackageManager,
  #[merge(strategy = merge::option::overwrite_none)]
  pub moonrepo: Option<MoonConfig>,
  #[merge(skip)]
  pub pre_commit: PreCommitConfig,
  #[merge(skip)]
  pub root_dir: String,
  #[merge(skip)]
  pub packages_dirs: Vec<String>,
  #[merge(strategy = merge_maps)]
  pub package_presets: BTreeMap<String, PackageConfig>,
  #[merge(strategy = merge_maps)]
  pub vitest_presets: BTreeMap<String, VitestConfigStruct>,
  #[merge(skip)]
  pub catalog: bool,
  #[merge(skip)]
  pub version_ranges: VersionRange,
  #[merge(skip)]
  pub shared_out_dir: SharedOutDir,
  #[merge(strategy = merge_maps)]
  pub people: BTreeMap<String, PersonData>,
  #[merge(strategy = merge::option::overwrite_none)]
  pub templates_dir: Option<String>,
  #[merge(strategy = merge_maps)]
  pub templates: BTreeMap<String, String>,
  #[merge(strategy = merge_maps)]
  pub global_templates_vars: BTreeMap<String, Value>,
  #[merge(skip)]
  pub generate_root_templates: Vec<TemplateOutput>,
  #[merge(strategy = merge_sets)]
  pub extends: BTreeSet<String>,
  #[merge(strategy = merge::bool::overwrite_true)]
  pub overwrite: bool,
}

impl Config {
  pub fn merge_configs(&mut self) -> Result<(), GenError> {
    for path in self.extends.clone() {
      let figment = Config::figment();

      let extended = if path.ends_with(".toml") {
        figment.merge(Toml::file(path))
      } else if path.ends_with(".yaml") {
        figment.merge(Yaml::file(path))
      } else {
        return Err(GenError::InvalidConfigFormat {
          file: path.to_string(),
        });
      };

      let extended_config: Config = extended
        .extract()
        .map_err(|e| GenError::ConfigParsing { source: e })?;

      self.merge(extended_config);
    }

    self.extends.clear();

    Ok(())
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SharedOutDir {
  Bool(bool),
  Name(String),
}

impl Default for SharedOutDir {
  fn default() -> Self {
    Self::Name(".out".to_string())
  }
}

impl SharedOutDir {
  pub fn get_name(&self) -> Option<String> {
    match self {
      Self::Bool(v) => {
        if *v {
          Some(".out".to_string())
        } else {
          None
        }
      }
      Self::Name(v) => Some(v.clone()),
    }
  }
}

#[derive(Debug, Template)]
#[template(path = "oxlint.json.j2")]
pub struct OxlintConfig;

impl Default for Config {
  fn default() -> Self {
    Self {
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
      packages_dirs: vec!["packages/*".to_string(), "apps/*".to_string()],
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
      shared_out_dir: SharedOutDir::Name(".out".to_string()),
      people: Default::default(),
      pnpm_config: Default::default(),
      templates_dir: Default::default(),
      templates: Default::default(),
      global_templates_vars: Default::default(),
      generate_root_templates: Default::default(),
      extends: Default::default(),
      overwrite: true,
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

  pub fn figment() -> Figment {
    Figment::from(Config::default())
      .merge(Toml::file("scaffolder/config.toml"))
      .merge(Yaml::file("scaffolder/config.yaml"))
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
