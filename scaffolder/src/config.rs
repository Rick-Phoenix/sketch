use std::{
  collections::{BTreeMap, BTreeSet},
  fmt::Display,
  path::{Path, PathBuf},
};

use askama::Template;
use figment::{
  value::{Dict, Map},
  Error, Figment, Metadata, Profile, Provider, Source,
};
use maplit::btreemap;
use merge::Merge;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
  merge_maps, merge_path, merge_sets,
  moon::MoonConfig,
  package::{vitest::VitestConfigStruct, PackageConfig},
  tera::TemplateOutput,
  GenError, PackageJson, Person, PersonData, TsConfig,
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

/// The global configuration struct.
#[derive(Clone, Debug, Deserialize, Serialize, Merge)]
#[serde(default)]
pub struct Config {
  /// Whether the default tsconfigs should be used for the root package.
  #[merge(skip)]
  pub root_use_default_tsconfigs: bool,
  /// The templates to generate when the root package is generated.
  /// The output paths will be joined with the `root_dir`.
  #[merge(skip)]
  pub root_generate_templates: Vec<TemplateOutput>,
  /// Whether to use the default oxlint configuration.
  #[merge(skip)]
  pub use_default_oxlint_config: bool,
  /// The gitignore settings.
  #[merge(skip)]
  pub gitignore: GitIgnore,
  /// The extra settings to render in the generated pnpm-workspace.yaml file, if pnpm is selected as a package manager.
  #[merge(strategy = merge_maps)]
  pub pnpm_config: BTreeMap<String, Value>,
  /// A map containing package.json presets.
  #[merge(strategy = merge_maps)]
  pub package_json_presets: BTreeMap<String, PackageJson>,
  /// A map containing tsconfig.json presets.
  #[merge(strategy = merge_maps)]
  pub tsconfig_presets: BTreeMap<String, TsConfig>,
  /// The name of the tsconfig file to use at the root, alongside tsconfig.json.
  /// It will be ignored if moonrepo is not used and if the default tsconfig presets are not used.
  /// It defaults to `tsconfig.options.json`.
  #[merge(skip)]
  pub root_tsconfig_name: String,
  /// The name of the tsconfig file to use inside the individual packages, alongside the default tsconfig.json file.
  /// It will be ignored if moonrepo is not used and if the default tsconfig presets are not used.
  #[merge(skip)]
  pub project_tsconfig_name: String,
  /// The name of the development tsconfig file (which will only typecheck scripts and tests and configs and generate no files) to use inside the individual packages, alongside the default tsconfig.json file.
  /// It will be ignored if moonrepo is not used and if the default tsconfig presets are not used.
  #[merge(skip)]
  pub dev_tsconfig_name: String,
  /// The package manager being used. It defaults to pnpm.
  #[merge(skip)]
  pub package_manager: PackageManager,
  /// Configuration settings for [`moonrepo`](https://moonrepo.dev/).
  #[merge(strategy = merge::option::overwrite_none)]
  pub moonrepo: Option<MoonConfig>,
  /// Configuration settings for [`pre-commit`](https://pre-commit.com/).
  #[merge(skip)]
  pub pre_commit: PreCommitSetting,
  /// The root directory for the monorepo. Defaults to the current working directory.
  #[merge(skip)]
  pub root_dir: String,
  /// The directories where packages will be located.
  /// They will be added to the pnpm-workspace.yaml config, and also generated automatically when the monorepo is generated.
  #[merge(skip)]
  pub packages_dirs: Vec<String>,
  #[merge(strategy = merge_maps)]
  /// A map of package presets.
  pub package_presets: BTreeMap<String, PackageConfig>,
  /// A map of vitest config presets.
  #[merge(strategy = merge_maps)]
  pub vitest_presets: BTreeMap<String, VitestConfigStruct>,
  /// Whether to use the pnpm catalog for default dependencies.
  #[merge(skip)]
  pub catalog: bool,
  /// The kind of version ranges to use for dependencies.
  #[merge(skip)]
  pub version_ranges: VersionRange,
  /// If this is set and the default tsconfigs are used, all tsc output will be directed to a single output directory in the root of the monorepo, with subdirectories for each package.
  #[merge(skip)]
  pub shared_out_dir: SharedOutDir,
  /// A map of individuals that can be referenced in the list of contributors or maintainers in a package.json file.
  #[merge(strategy = merge_maps)]
  pub people: BTreeMap<String, PersonData>,
  /// The directory that contains the template files.
  #[merge(strategy = merge::option::overwrite_none)]
  pub templates_dir: Option<String>,
  /// A map that contains templates defined literally.
  #[merge(strategy = merge_maps)]
  pub templates: BTreeMap<String, String>,
  /// The global variables that will be available for every template being generated.
  /// They are overridden in case a template is rendered with a local context.
  #[merge(strategy = merge_maps)]
  pub global_templates_vars: BTreeMap<String, Value>,
  /// The list of configuration files to merge with the current one.
  #[merge(strategy = merge_sets)]
  pub extends: BTreeSet<PathBuf>,
  /// Whether file generation should always override existing files. Defaults to true.
  #[merge(strategy = merge::bool::overwrite_true)]
  pub overwrite: bool,
}

impl Config {
  pub fn get_extended_configs(
    &mut self,
    base_path: &Path,
    current_path: &Path,
    processed_sources: &mut Vec<PathBuf>,
  ) -> Result<(), GenError> {
    processed_sources.push(current_path.to_path_buf());

    for path in self.extends.clone() {
      let path =
        base_path
          .join(&path)
          .canonicalize()
          .map_err(|e| GenError::PathCanonicalization {
            path: path.clone(),
            source: e,
          })?;

      let extended_figment = merge_path(Config::figment(), &path)?;

      for data in extended_figment.metadata() {
        if let Some(Source::File(extended_source)) = &data.source
          && processed_sources.contains(extended_source) {
            let was_absent = !processed_sources.contains(extended_source);
            processed_sources.push(extended_source.clone());

            if !was_absent {
            let chain: Vec<_> = processed_sources.iter().map(|source| source.to_string_lossy()).collect();

              return Err(GenError::CircularDependency(format!(
                "Found circular dependency to the config file in '{}'. The full processed path is: {}",
                extended_source.display(), chain.join(" -> ")
              )));
            }
          }
      }

      let mut extended_config: Config = extended_figment
        .extract()
        .map_err(|e| GenError::ConfigParsing { source: e })?;

      extended_config.get_extended_configs(base_path, &path, processed_sources)?;

      self.merge(extended_config);
    }

    Ok(())
  }

  pub fn merge_configs(mut self, initial_config_file: &Path) -> Result<Self, GenError> {
    let mut processed_sources: Vec<PathBuf> = Default::default();

    let base_path = initial_config_file
      .parent()
      .ok_or(GenError::Custom(format!(
        "Could not get the parent directory of file '{}' to get the extended configs.",
        initial_config_file.display()
      )))?;

    self.get_extended_configs(base_path, initial_config_file, &mut processed_sources)?;

    Ok(self)
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
      package_json_presets: btreemap! { "root".to_string() => PackageJson::default() },
      package_manager: Default::default(),
      root_tsconfig_name: "tsconfig.options.json".to_string(),
      project_tsconfig_name: "tsconfig.src.json".to_string(),
      dev_tsconfig_name: "tsconfig.dev.json".to_string(),
      moonrepo: None,
      pre_commit: PreCommitSetting::Bool(true),
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
      root_generate_templates: Default::default(),
      extends: Default::default(),
      overwrite: true,
      root_use_default_tsconfigs: true,
      use_default_oxlint_config: true,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PreCommitSetting {
  Bool(bool),
  Config(PreCommitConfig),
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
  pub fn from<T: Provider>(provider: T) -> Result<Config, Error> {
    Figment::from(provider).extract()
  }

  pub fn figment() -> Figment {
    Figment::from(Config::default())
  }
}

// Make `Config` a provider itself for composability.
impl Provider for Config {
  fn metadata(&self) -> Metadata {
    Metadata::named("default")
  }

  fn data(&self) -> Result<Map<Profile, Dict>, Error> {
    figment::providers::Serialized::defaults(Config::default()).data()
  }
}
