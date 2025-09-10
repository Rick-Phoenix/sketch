use std::{
  collections::BTreeSet,
  path::{Path, PathBuf},
};

use clap::Parser;
use figment::{
  providers::{Env, Format, Json, Toml, Yaml},
  value::{Dict, Map},
  Error, Figment, Metadata, Profile, Provider, Source,
};
use indexmap::{IndexMap, IndexSet};
use maplit::btreeset;
use merge::Merge;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
  cli::parsers::parse_btreeset_from_csv,
  config_elements::*,
  merge_config_file, merge_index_maps, merge_index_sets, merge_optional_nested,
  moon::MoonConfigKind,
  overwrite_option,
  package::{vitest::VitestConfigStruct, PackageConfig},
  package_json::{PackageJson, PackageJsonKind, Person, PersonData},
  tera::TemplateOutput,
  ts_config::{TsConfig, TsConfigDirective},
  GenError, SharedOutDir, VersionRange,
};

impl TypescriptConfig {
  pub fn get_contributor(&self, name: &str) -> Option<Person> {
    self
      .people
      .get(name)
      .map(|person| Person::Data(person.clone()))
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, Parser, Merge)]
#[merge(strategy = overwrite_option)]
#[serde(default)]
pub struct RootPackage {
  #[arg(short, long)]
  pub name: Option<String>,

  #[arg(skip)]
  pub oxlint: Option<OxlintConfig>,

  #[arg(short, long, value_parser = TsConfigDirective::from_cli)]
  pub ts_config: Option<Vec<TsConfigDirective>>,

  #[arg(skip)]
  pub generate_templates: Option<Vec<TemplateOutput>>,

  #[arg(short, long, value_parser = PackageJsonKind::from_cli)]
  pub package_json: Option<PackageJsonKind>,

  /// Configuration settings for [`moonrepo`](https://moonrepo.dev/).
  #[merge(strategy = merge::option::overwrite_none)]
  #[arg(skip)]
  pub moonrepo: Option<MoonConfigKind>,
}

impl Default for RootPackage {
  fn default() -> Self {
    Self {
      name: None,
      oxlint: Some(Default::default()),
      ts_config: Default::default(),
      generate_templates: Default::default(),
      package_json: Default::default(),
      moonrepo: None,
    }
  }
}

#[derive(Clone, Debug, Deserialize, Serialize, Merge, Parser)]
#[serde(default)]
pub struct TypescriptConfig {
  /// The configuration for the root typescript package
  #[merge(skip)]
  #[arg(skip)]
  pub root_package: Option<RootPackage>,

  /// The name of the tsconfig file to use at the root, alongside tsconfig.json.
  /// Ignored if moonrepo is not used and if the default tsconfig presets are not used.
  /// Defaults to "tsconfig.options.json".
  #[merge(strategy = overwrite_option)]
  #[arg(long)]
  pub root_tsconfig_name: Option<String>,

  /// The name of the tsconfig file to use inside the individual packages, alongside the default tsconfig.json file.
  /// Ignored if moonrepo is not used and if the default tsconfig presets are not used.
  /// Defaults to "tsconfig.src.json"
  #[merge(strategy = overwrite_option)]
  #[arg(long)]
  pub project_tsconfig_name: Option<String>,

  /// The name of the development tsconfig file (which will only typecheck scripts and tests and configs and generate no files) to use inside the individual packages, alongside the default tsconfig.json file.
  /// Ignored if moonrepo is not used and if the default tsconfig presets are not used.
  /// Defaults to "tsconfig.dev.json".
  #[merge(strategy = overwrite_option)]
  #[arg(long)]
  pub dev_tsconfig_name: Option<String>,

  /// The package manager being used. Defaults to pnpm.
  #[merge(strategy = overwrite_option)]
  #[arg(value_enum, long)]
  pub package_manager: Option<PackageManager>,

  /// The root directory for the project. Defaults to the current working directory.
  #[merge(strategy = overwrite_option)]
  #[arg(long)]
  pub root_dir: Option<String>,

  /// The directories where packages will be located.
  /// They will be added to the pnpm-workspace.yaml config, and also generated automatically when the monorepo is generated.
  #[merge(strategy = overwrite_option)]
  #[arg(long, value_parser = parse_btreeset_from_csv)]
  pub packages_dirs: Option<BTreeSet<String>>,

  /// The kind of version ranges to use for dependencies that are fetched automatically.
  /// When a dependency with `catalog:` is listed in a [`PackageJson`] and it's not present in pnpm-workspace.yaml, the crate will fetch the latest version using the Npm api, and use the selected version range with the latest version.
  #[merge(strategy = overwrite_option)]
  #[arg(value_enum)]
  #[arg(long)]
  pub version_ranges: Option<VersionRange>,

  /// Whether to use the pnpm catalog for default dependencies.
  #[merge(strategy = merge::bool::overwrite_true)]
  #[arg(skip)]
  pub catalog: bool,

  /// Whether the dependencies with 'latest' should be transformed in their actual latest version + the selected version range.
  #[merge(strategy = merge::bool::overwrite_true)]
  #[arg(skip)]
  pub convert_latest_to_range: bool,

  /// A map of individuals btree_that can be referenced in the list of contributors or maintainers in a package.json file.
  #[arg(skip)]
  #[merge(strategy = merge_index_maps)]
  pub people: IndexMap<String, PersonData>,

  /// A map containing package.json presets.
  #[merge(strategy = merge_index_maps)]
  #[arg(skip)]
  pub package_json_presets: IndexMap<String, PackageJson>,

  /// A map containing tsconfig.json presets.
  #[merge(strategy = merge_index_maps)]
  #[arg(skip)]
  pub tsconfig_presets: IndexMap<String, TsConfig>,

  /// A map of package presets.
  #[merge(strategy = merge_index_maps)]
  #[arg(skip)]
  pub package_presets: IndexMap<String, PackageConfig>,

  /// A map of vitest config presets.
  #[arg(skip)]
  #[merge(strategy = merge_index_maps)]
  pub vitest_presets: IndexMap<String, VitestConfigStruct>,

  /// If this is set and the default tsconfigs are used, all tsc output will be directed to a single output directory in the root of the monorepo, with subdirectories for each package.
  #[merge(skip)]
  #[arg(skip)]
  pub shared_out_dir: SharedOutDir,
  /// The extra settings to render in the generated pnpm-workspace.yaml file, if pnpm is selected as a package manager.
  #[merge(strategy = merge_index_maps)]
  #[arg(skip)]
  pub pnpm_config: IndexMap<String, Value>,
}

/// The global configuration struct.
#[derive(Clone, Debug, Deserialize, Serialize, Merge, Parser)]
#[serde(default)]
pub struct Config {
  #[merge(strategy = merge_optional_nested)]
  #[arg(skip)]
  pub typescript: Option<TypescriptConfig>,

  /// The shell to use for commands. Defaults to 'cmd.exe' on windows and 'sh' elsewhere.
  #[merge(strategy = merge::option::overwrite_none)]
  #[arg(long)]
  pub shell: Option<String>,

  /// Activates debugging mode.
  #[merge(strategy = merge::bool::overwrite_false)]
  #[arg(long)]
  pub debug: bool,

  /// The root directory for the project. Defaults to the current working directory.
  #[merge(strategy = overwrite_option)]
  #[arg(long)]
  pub root_dir: Option<String>,

  /// The directory that contains the template files.
  #[merge(strategy = merge::option::overwrite_none)]
  #[arg(long)]
  pub templates_dir: Option<String>,

  /// Whether file generation should always override existing files. Defaults to true.
  #[merge(strategy = merge::bool::overwrite_true)]
  #[arg(skip)]
  pub overwrite: bool,

  /// Configuration settings for [`pre-commit`](https://pre-commit.com/).
  #[merge(skip)]
  #[arg(skip)]
  pub pre_commit: PreCommitSetting,

  #[merge(skip)]
  #[arg(skip)]
  pub gitignore: GitIgnore,

  /// The list of configuration files to merge with the current one.
  #[merge(strategy = merge_index_sets)]
  #[arg(skip)]
  pub extends: IndexSet<PathBuf>,

  /// A map that contains templates defined literally.
  #[merge(strategy = merge_index_maps)]
  #[arg(skip)]
  pub templates: IndexMap<String, String>,

  /// The global variables that will be available for every template being generated.
  /// They are overridden in case a template is rendered with a local context.
  #[merge(strategy = merge_index_maps)]
  #[arg(skip)]
  pub global_templates_vars: IndexMap<String, Value>,
}

impl Config {
  fn merge_configs_recursive(
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

      let extended_figment = merge_config_file(Config::figment(), &path)?;

      for data in extended_figment.metadata() {
        if let Some(Source::File(extended_source)) = &data.source
          && processed_sources.contains(extended_source) {
            let was_absent = !processed_sources.contains(extended_source);
            processed_sources.push(extended_source.clone());

            if !was_absent {
            let chain: Vec<_> = processed_sources.iter().map(|source| source.to_string_lossy()).collect();

              return Err(GenError::CircularDependency(format!(
                "Found circular dependency to the config file {}. The full processed path is: {}",
                extended_source.display(), chain.join(" -> ")
              )));
            }
          }
      }

      let mut extended_config: Config = extended_figment
        .extract()
        .map_err(|e| GenError::ConfigParsing { source: e })?;

      extended_config.merge_configs_recursive(base_path, &path, processed_sources)?;

      self.merge(extended_config);
    }

    Ok(())
  }

  pub fn merge_configs(mut self, base_path: &Path) -> Result<Self, GenError> {
    let mut processed_sources: Vec<PathBuf> = Default::default();

    self.merge_configs_recursive(base_path, base_path, &mut processed_sources)?;

    Ok(self)
  }
}

impl Default for TypescriptConfig {
  fn default() -> Self {
    Self {
      convert_latest_to_range: true,
      package_json_presets: Default::default(),
      package_manager: Default::default(),
      root_tsconfig_name: None,
      project_tsconfig_name: None,
      dev_tsconfig_name: None,
      packages_dirs: Some(btreeset!["packages/*".to_string(), "apps/*".to_string()]),
      package_presets: Default::default(),
      vitest_presets: Default::default(),
      catalog: true,
      version_ranges: Default::default(),
      tsconfig_presets: Default::default(),
      shared_out_dir: SharedOutDir::Name(".out".to_string()),
      people: Default::default(),
      pnpm_config: Default::default(),
      root_package: Default::default(),
      root_dir: None,
    }
  }
}

impl Default for Config {
  fn default() -> Self {
    Self {
      typescript: None,
      shell: None,
      debug: false,
      gitignore: Default::default(),
      pre_commit: PreCommitSetting::Bool(true),
      root_dir: None,
      templates_dir: Default::default(),
      templates: Default::default(),
      global_templates_vars: Default::default(),
      extends: Default::default(),
      overwrite: true,
    }
  }
}

impl Config {
  pub fn from<T: Provider>(provider: T) -> Result<Config, Error> {
    Figment::from(provider).extract()
  }

  pub fn figment() -> Figment {
    Figment::from(Config::default())
      .merge(Yaml::file("sketch.yaml"))
      .merge(Toml::file("sketch.toml"))
      .merge(Json::file("sketch.json"))
      .merge(Env::prefixed("SKETCH_"))
  }
}

impl Provider for Config {
  fn metadata(&self) -> Metadata {
    Metadata::named("Config Struct")
  }

  fn data(&self) -> Result<Map<Profile, Dict>, Error> {
    figment::providers::Serialized::defaults(Config::default()).data()
  }
}
