use std::path::PathBuf;

use clap::Parser;
use indexmap::{IndexMap, IndexSet};
use merge::Merge;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
  config_elements::*,
  config_setup::extract_config_from_file,
  custom_templating::TemplateOutput,
  is_default, merge_index_maps, merge_index_sets, overwrite_option,
  package::PackageConfig,
  package_json::{PackageJson, PackageJsonKind, Person, PersonData},
  paths::get_parent_dir,
  pnpm::PnpmWorkspace,
  ts_config::{TsConfig, TsConfigDirective},
  GenError, VersionRange,
};

impl TypescriptConfig {
  pub fn get_contributor(&self, name: &str) -> Option<Person> {
    self
      .people
      .get(name)
      .map(|person| Person::Data(person.clone()))
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, Parser, Merge, PartialEq, JsonSchema)]
#[merge(strategy = overwrite_option)]
#[serde(default)]
pub struct RootPackage {
  /// The name of the root package [default: "root"].
  #[arg(short, long)]
  pub name: Option<String>,

  /// Oxlint configuration for the root package.
  #[arg(skip)]
  pub oxlint: Option<OxlintConfig>,

  /// A list of [`TsConfigDirective`]s for the root package. They can be preset ids or literal configurations. If unset, defaults are used.
  #[arg(help = "One or many tsconfig files for the root package. If unset, defaults are used")]
  #[arg(short, long, value_parser = TsConfigDirective::from_cli, value_name = "output=PATH,id=ID")]
  pub ts_config: Option<Vec<TsConfigDirective>>,

  /// The [`PackageJsonKind`] to use for the root package. It can be a preset id or a literal definition.
  #[arg(short, long, value_parser = PackageJsonKind::from_cli)]
  #[arg(
    help = "The id of the package.json preset to use for the root package",
    value_name = "ID"
  )]
  pub package_json: Option<PackageJsonKind>,

  /// The templates to generate when the root package is generated.
  #[arg(skip)]
  pub generate_templates: Option<Vec<TemplateOutput>>,
}

impl Default for RootPackage {
  fn default() -> Self {
    Self {
      name: None,
      oxlint: Some(Default::default()),
      ts_config: Default::default(),
      generate_templates: Default::default(),
      package_json: Default::default(),
    }
  }
}

#[derive(Clone, Debug, Deserialize, Serialize, Merge, Parser, PartialEq, JsonSchema)]
#[serde(default)]
pub struct TypescriptConfig {
  /// The configuration for the root typescript package.
  #[merge(skip)]
  #[arg(skip)]
  pub root_package: Option<RootPackage>,

  /// The package manager being used. [default: pnpm].
  #[merge(strategy = overwrite_option)]
  #[arg(value_enum, long, value_name = "NAME")]
  pub package_manager: Option<PackageManager>,

  /// Do not add default dependencies to new `package.json` files (typescript and oxlint, plus vitest if enabled)
  #[merge(strategy = merge::bool::overwrite_false)]
  #[arg(long)]
  pub no_default_deps: bool,

  /// The kind of version ranges to use for dependencies that are fetched automatically (such as when a dependency with `catalog:` is listed in a [`PackageJson`] and it's not present in pnpm-workspace.yaml, or when a dependency is set to `latest` and [`TypescriptConfig::convert_latest_to_range`] is set to true).
  #[merge(strategy = overwrite_option)]
  #[arg(value_enum)]
  #[arg(long, value_name = "KIND")]
  #[arg(
    help = "The kind of version range to use for dependencies added automatically [default: minor]"
  )]
  pub version_range: Option<VersionRange>,

  /// Whether to use the pnpm catalog for default dependencies.
  #[merge(strategy = merge::bool::overwrite_false)]
  #[arg(long)]
  pub catalog: bool,

  /// Whether the dependencies with `latest` should be converted to a version range.
  #[merge(strategy = merge::bool::overwrite_false)]
  #[arg(long = "no-convert-latest")]
  pub no_convert_latest_to_range: bool,

  /// A map of individual [`PersonData`] that can be referenced in [`PackageJson::contributors`] or [`PackageJson::maintainers`].
  #[arg(skip)]
  #[merge(strategy = merge_index_maps)]
  pub people: IndexMap<String, PersonData>,

  /// A map containing [`PackageJson`] presets.
  #[merge(strategy = merge_index_maps)]
  #[arg(skip)]
  pub package_json_presets: IndexMap<String, PackageJson>,

  /// A map containing [`TsConfig`] presets.
  #[merge(strategy = merge_index_maps)]
  #[arg(skip)]
  pub tsconfig_presets: IndexMap<String, TsConfig>,

  /// A map of [`PackageConfig`] presets.
  #[merge(strategy = merge_index_maps)]
  #[arg(skip)]
  pub package_presets: IndexMap<String, PackageConfig>,

  /// The settings to use in the generated pnpm-workspace.yaml file, if pnpm is selected as a package manager.
  #[merge(skip)]
  #[arg(skip)]
  pub pnpm_config: Option<PnpmWorkspace>,
}

impl Config {
  pub fn new() -> Self {
    Self {
      ..Default::default()
    }
  }
}

/// The global configuration struct.
#[derive(Clone, Debug, Deserialize, Serialize, Merge, Parser, PartialEq, JsonSchema)]
#[serde(default)]
pub struct Config {
  #[serde(skip)]
  #[arg(skip)]
  #[merge(strategy = merge::option::overwrite_none)]
  pub(crate) config_file: Option<PathBuf>,

  /// The configuration for typescript projects.
  #[merge(strategy = merge::option::overwrite_none)]
  #[arg(skip)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub typescript: Option<TypescriptConfig>,

  /// The shell to use for commands [default: `cmd.exe` on windows and `sh` elsewhere].
  #[merge(strategy = merge::option::overwrite_none)]
  #[arg(long)]
  pub shell: Option<String>,

  /// Activates debugging mode.
  #[merge(strategy = merge::bool::overwrite_false)]
  #[arg(long)]
  #[serde(skip_serializing_if = "is_default")]
  pub debug: bool,

  /// The root directory for the project [default: "."].
  #[merge(strategy = overwrite_option)]
  #[arg(long, value_name = "DIR")]
  pub root_dir: Option<PathBuf>,

  /// The path to the directory with the template files.
  #[merge(strategy = overwrite_option)]
  #[arg(long, value_name = "DIR")]
  pub templates_dir: Option<PathBuf>,

  /// Exits with error if a file being created already exists.
  #[merge(strategy = merge::bool::overwrite_false)]
  #[arg(long)]
  pub no_overwrite: bool,

  /// Configuration settings for [`pre-commit`](https://pre-commit.com/).
  #[merge(skip)]
  #[arg(skip)]
  pub pre_commit: PreCommitSetting,

  /// Settings for the gitignore file. You can either add more entries on top of the defaults, or replace the defaults altogether.
  #[merge(skip)]
  #[arg(skip)]
  #[serde(skip_serializing_if = "is_default")]
  pub gitignore: GitIgnore,

  /// The relative paths, from the current file, to the other config files to merge with the current one.
  #[merge(strategy = merge_index_sets)]
  #[arg(skip)]
  #[serde(skip_serializing_if = "is_default")]
  pub extends: IndexSet<PathBuf>,

  /// A map that contains templates defined literally.
  #[merge(strategy = merge_index_maps)]
  #[arg(skip)]
  pub templates: IndexMap<String, String>,

  /// A map that contains templating presets.
  #[merge(strategy = merge_index_maps)]
  #[arg(skip)]
  pub templating_presets: IndexMap<String, Vec<TemplateOutput>>,

  /// The global variables that will be available for every template being generated.
  /// They are overridden by vars set as a template's local context or via a cli command.
  #[merge(strategy = merge_index_maps)]
  #[arg(skip)]
  pub global_templates_vars: IndexMap<String, Value>,
}

impl Config {
  fn merge_configs_recursive(
    &mut self,
    is_initial: bool,
    base: &mut Config,

    processed_sources: &mut IndexSet<PathBuf>,
  ) -> Result<(), GenError> {
    // Safe unwrapping due to the check below
    let current_config_file = self.config_file.clone().unwrap();
    let current_dir = get_parent_dir(&current_config_file);

    for rel_path in self.extends.clone() {
      let abs_path =
        current_dir
          .join(&rel_path)
          .canonicalize()
          .map_err(|e| GenError::PathCanonicalization {
            path: rel_path.clone(),
            source: e,
          })?;

      let mut extended_config = extract_config_from_file(&abs_path)?;

      let was_absent = processed_sources.insert(abs_path.to_path_buf());

      if !was_absent {
        let chain: Vec<_> = processed_sources
          .iter()
          .map(|source| source.to_string_lossy())
          .collect();

        return Err(GenError::CircularDependency(format!(
          "Found circular dependency to the config file {}. The full processed path is: {}",
          abs_path.display(),
          chain.join(" -> ")
        )));
      }

      extended_config.merge_configs_recursive(false, base, processed_sources)?;

      base.merge(extended_config);
    }

    if !is_initial {
      base.merge(self.clone());
    }

    Ok(())
  }

  pub fn merge_config_files(mut self) -> Result<Self, GenError> {
    let mut processed_sources: IndexSet<PathBuf> = Default::default();

    let config_file = self
      .config_file
      .clone()
      .expect("Cannot use merge_config_files with a config that has no source file.");

    processed_sources.insert(config_file.clone());

    let mut extended = Config::default();

    self.merge_configs_recursive(true, &mut extended, &mut processed_sources)?;

    extended.merge(self);

    processed_sources.swap_remove(&config_file);

    // Replace rel paths with abs paths for better debugging
    extended.extends = processed_sources;

    Ok(extended)
  }
}

impl Default for TypescriptConfig {
  fn default() -> Self {
    Self {
      no_default_deps: false,
      no_convert_latest_to_range: false,
      package_json_presets: Default::default(),
      package_manager: Default::default(),
      package_presets: Default::default(),
      catalog: false,
      version_range: Default::default(),
      tsconfig_presets: Default::default(),
      people: Default::default(),
      pnpm_config: Default::default(),
      root_package: Default::default(),
    }
  }
}

impl Default for Config {
  fn default() -> Self {
    Self {
      config_file: None,
      templating_presets: Default::default(),
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
      no_overwrite: false,
    }
  }
}
