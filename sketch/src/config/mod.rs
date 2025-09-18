mod config_elements;
mod config_setup;

use std::path::PathBuf;

use clap::Parser;
pub use config_elements::*;
pub(crate) use config_setup::*;
use indexmap::{IndexMap, IndexSet};
use merge::Merge;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
  custom_templating::TemplateOutput, is_default, merge_index_maps, merge_index_sets,
  overwrite_option, paths::get_parent_dir, ts::TypescriptConfig, GenError,
};

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

  /// The base path for the generated files [default: "."].
  #[merge(strategy = overwrite_option)]
  #[arg(long, value_name = "DIR")]
  pub out_dir: Option<PathBuf>,

  /// The path to the templates directory, starting from the cwd (when set via cli) or from the config file (when defined in one of them).
  #[merge(strategy = overwrite_option)]
  #[arg(long, value_name = "DIR")]
  pub templates_dir: Option<PathBuf>,

  /// Does not overwrite existing files.
  #[merge(strategy = merge::bool::overwrite_false)]
  #[arg(long)]
  pub no_overwrite: bool,

  /// Configuration settings for [`pre-commit`](https://pre-commit.com/), to use when creating a new repo.
  #[merge(skip)]
  #[arg(skip)]
  pub pre_commit: PreCommitSetting,

  /// Settings for the gitignore file to generate in new repos. It can be a list of directives to append to the defaults or a string, to replace the defaults entirely.
  #[merge(skip)]
  #[arg(skip)]
  #[serde(skip_serializing_if = "is_default")]
  pub gitignore: GitIgnore,

  /// The relative paths, from the current file, to the config files to merge with the current one.
  #[merge(strategy = merge_index_sets)]
  #[arg(skip)]
  #[serde(skip_serializing_if = "is_default")]
  pub extends: IndexSet<PathBuf>,

  /// A map that contains template definitions.
  #[merge(strategy = merge_index_maps)]
  #[arg(skip)]
  pub templates: IndexMap<String, String>,

  /// A map that contains templating presets.
  #[merge(strategy = merge_index_maps)]
  #[arg(skip)]
  pub templating_presets: IndexMap<String, Vec<TemplateOutput>>,

  /// The global variables that will be available for every template being generated.
  /// They are overridden by vars set in a template's local context or via the cli.
  #[merge(strategy = merge_index_maps)]
  #[arg(skip)]
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
      out_dir: None,
      templates_dir: Default::default(),
      templates: Default::default(),
      vars: Default::default(),
      extends: Default::default(),
      no_overwrite: false,
    }
  }
}
