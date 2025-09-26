mod config_setup;

use std::path::PathBuf;

use clap::Parser;
use config_setup::extract_config_from_file;
use indexmap::{IndexMap, IndexSet};
use merge::Merge;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
  custom_templating::TemplatingPreset,
  fs::get_parent_dir,
  init_repo::{gitignore::GitignorePreset, pre_commit::PreCommitPreset, RepoPreset},
  is_default, merge_index_maps, merge_index_sets, merge_optional_nested, overwrite_if_some,
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
#[derive(Clone, Debug, Deserialize, Serialize, Merge, Parser, PartialEq, JsonSchema)]
#[merge(strategy = overwrite_if_some)]
#[serde(default)]
pub struct Config {
  #[serde(skip)]
  #[arg(skip)]
  #[merge(strategy = merge::option::overwrite_none)]
  pub(crate) config_file: Option<PathBuf>,

  /// The configuration for typescript projects.
  #[merge(strategy = merge_optional_nested)]
  #[arg(skip)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub typescript: Option<TypescriptConfig>,

  /// The shell to use for commands [default: `cmd.exe` on windows and `sh` elsewhere].
  #[arg(long)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub shell: Option<String>,

  /// Activates debugging mode.
  #[arg(long)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub debug: Option<bool>,

  /// The path to the templates directory.
  #[arg(long, value_name = "DIR")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub templates_dir: Option<PathBuf>,

  /// Do not overwrite existing files.
  #[arg(long)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_overwrite: Option<bool>,

  /// The paths (absolute, or relative to the originating config file) to the config files to extend.
  #[merge(strategy = merge_index_sets)]
  #[arg(skip)]
  #[serde(skip_serializing_if = "is_default")]
  pub extends: IndexSet<PathBuf>,

  /// A map that contains template definitions.
  #[merge(strategy = merge_index_maps)]
  #[arg(skip)]
  #[serde(skip_serializing_if = "is_default")]
  pub templates: IndexMap<String, String>,

  /// A map that contains templating presets.
  #[merge(strategy = merge_index_maps)]
  #[arg(skip)]
  #[serde(skip_serializing_if = "is_default")]
  pub templating_presets: IndexMap<String, TemplatingPreset>,

  /// A map that contains pre-commit presets.
  #[merge(strategy = merge_index_maps)]
  #[arg(skip)]
  #[serde(skip_serializing_if = "is_default")]
  pub pre_commit_presets: IndexMap<String, PreCommitPreset>,

  /// A map that contains gitignore presets.
  #[merge(strategy = merge_index_maps)]
  #[arg(skip)]
  #[serde(skip_serializing_if = "is_default")]
  pub gitignore_presets: IndexMap<String, GitignorePreset>,

  /// A map that contains presets for git repos.
  #[merge(strategy = merge_index_maps)]
  #[arg(skip)]
  #[serde(skip_serializing_if = "is_default")]
  pub git_presets: IndexMap<String, RepoPreset>,

  /// The global variables that will be available for every template being generated.
  /// They are overridden by vars set in a template's local context or via the cli.
  #[merge(strategy = merge_index_maps)]
  #[arg(skip)]
  #[serde(skip_serializing_if = "is_default")]
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
    let current_config_file = self.config_file.clone().unwrap();
    let current_dir = get_parent_dir(&current_config_file);

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

    processed_sources.swap_remove(&config_file);

    // Replace rel paths with abs paths for better debugging
    extended.extends = processed_sources;

    Ok(extended)
  }
}

impl Default for Config {
  fn default() -> Self {
    Self {
      git_presets: Default::default(),
      gitignore_presets: Default::default(),
      pre_commit_presets: Default::default(),
      config_file: None,
      templating_presets: Default::default(),
      typescript: None,
      shell: None,
      debug: None,
      templates_dir: Default::default(),
      templates: Default::default(),
      vars: Default::default(),
      extends: Default::default(),
      no_overwrite: None,
    }
  }
}
