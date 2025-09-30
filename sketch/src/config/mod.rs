mod config_setup;

use std::path::PathBuf;

use config_setup::extract_config_from_file;
use indexmap::{IndexMap, IndexSet};
use merge::Merge;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
  custom_templating::TemplatingPreset,
  docker::DockerConfig,
  fs::get_parent_dir,
  init_repo::{gitignore::GitignorePreset, pre_commit::PreCommitPreset, RepoPreset},
  merge_index_maps, merge_index_sets, merge_optional_nested, overwrite_if_some,
  rust::CargoTomlPreset,
  ts::TypescriptConfig,
  GenError,
};

impl Config {
  pub fn new() -> Self {
    Self {
      ..Default::default()
    }
  }

  pub(crate) fn can_overwrite(&self) -> bool {
    !self.no_overwrite.unwrap_or_default()
  }
}

/// The global configuration struct.
#[derive(Clone, Debug, Deserialize, Serialize, Merge, PartialEq, JsonSchema)]
#[merge(strategy = overwrite_if_some)]
#[serde(default)]
pub struct Config {
  #[serde(skip)]
  #[merge(strategy = merge::option::overwrite_none)]
  pub(crate) config_file: Option<PathBuf>,

  /// The configuration for typescript projects.
  #[merge(strategy = merge_optional_nested)]
  pub typescript: Option<TypescriptConfig>,

  /// Configuration and presets for Docker.
  #[merge(strategy = merge_optional_nested)]
  pub docker: Option<DockerConfig>,

  /// The shell to use for commands [default: `cmd.exe` on windows and `sh` elsewhere].
  pub shell: Option<String>,

  /// The path to the templates directory.
  pub templates_dir: Option<PathBuf>,

  /// Do not overwrite existing files.
  pub no_overwrite: Option<bool>,

  /// The paths (absolute, or relative to the originating config file) to the config files to extend.
  #[merge(strategy = merge_index_sets)]
  pub extends: IndexSet<PathBuf>,

  /// A map that contains template definitions.
  #[merge(strategy = merge_index_maps)]
  pub templates: IndexMap<String, String>,

  /// A map that contains templating presets.
  #[merge(strategy = merge_index_maps)]
  pub templating_presets: IndexMap<String, TemplatingPreset>,

  /// A map that contains pre-commit presets.
  #[merge(strategy = merge_index_maps)]
  pub pre_commit_presets: IndexMap<String, PreCommitPreset>,

  /// A map that contains gitignore presets.
  #[merge(strategy = merge_index_maps)]
  pub gitignore_presets: IndexMap<String, GitignorePreset>,

  /// A map that contains presets for git repos.
  #[merge(strategy = merge_index_maps)]
  pub git_presets: IndexMap<String, RepoPreset>,

  /// A map that contains presets for `Cargo.toml` files.
  #[merge(strategy = merge_index_maps)]
  pub cargo_toml_presets: IndexMap<String, CargoTomlPreset>,

  /// The global variables that will be available for every template being generated.
  /// They are overridden by vars set in a template's local context or via the cli.
  #[merge(strategy = merge_index_maps)]
  pub vars: IndexMap<String, Value>,
}

impl Config {
  fn merge_configs_recursive(
    &mut self,
    is_initial: bool,
    base: &mut Config,

    processed_sources: &mut IndexSet<PathBuf>,
  ) -> Result<(), GenError> {
    // Safe unwrapping due to the check below
    let current_config_file = self.config_file.as_ref();
    let current_dir = get_parent_dir(current_config_file.unwrap());

    for rel_path in &self.extends {
      let abs_path =
        current_dir
          .join(rel_path)
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

  /// Recursively merges a [`Config`] with its extended configs.
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

    // Should not show up in the `extends` list
    processed_sources.swap_remove(&config_file);

    // Replace rel paths with abs paths for better debugging
    extended.extends = processed_sources;

    Ok(extended)
  }
}

impl Default for Config {
  fn default() -> Self {
    Self {
      docker: None,
      cargo_toml_presets: Default::default(),
      git_presets: Default::default(),
      gitignore_presets: Default::default(),
      pre_commit_presets: Default::default(),
      config_file: None,
      templating_presets: Default::default(),
      typescript: None,
      shell: None,
      templates_dir: Default::default(),
      templates: Default::default(),
      vars: Default::default(),
      extends: Default::default(),
      no_overwrite: None,
    }
  }
}
