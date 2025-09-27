mod pre_commit_elements;

use std::collections::BTreeSet;

use indexmap::{IndexMap, IndexSet};
use maplit::btreeset;
use merge::Merge;
use pre_commit_elements::*;
pub(crate) use pre_commit_elements::{FileType, Language, GITLEAKS_REPO};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
  merge_btree_sets, merge_index_sets, merge_nested, merge_optional_btree_maps,
  merge_optional_btree_sets, merge_presets, overwrite_if_some, Extensible, GenError, Preset,
  StringBTreeMap,
};

/// A preset for a `.pre-commit-config.yaml` configuration file.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, Merge, Default)]
#[serde(default)]
pub struct PreCommitPreset {
  /// The ids of the extended configurations.
  #[merge(strategy = merge_index_sets)]
  pub extends: IndexSet<String>,

  #[serde(flatten)]
  #[merge(strategy = merge_nested)]
  pub config: PreCommitConfig,
}

impl Extensible for PreCommitPreset {
  fn get_extended(&self) -> &IndexSet<String> {
    &self.extends
  }
}

impl PreCommitPreset {
  pub fn process_data(
    self,
    id: &str,
    store: &IndexMap<String, PreCommitPreset>,
  ) -> Result<PreCommitConfig, GenError> {
    if self.extends.is_empty() {
      return Ok(self.config);
    }

    let mut processed_ids: IndexSet<String> = IndexSet::new();

    let merged_preset = merge_presets(Preset::PreCommit, id, self, store, &mut processed_ids)?;

    Ok(merged_preset.config)
  }
}

impl Default for PreCommitConfig {
  fn default() -> Self {
    Self {
      repos: btreeset! { GITLEAKS_REPO.clone() },
      ci: None,
      default_install_hook_types: None,
      default_language_version: None,
      default_stages: None,
      files: None,
      exclude: None,
      fail_fast: None,
      minimum_pre_commit_version: None,
    }
  }
}

/// Configuration settings for [`pre-commit`](https://pre-commit.com)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, Merge)]
#[merge(strategy = overwrite_if_some)]
#[serde(default)]
pub struct PreCommitConfig {
  /// A minimum version of pre-commit https://pre-commit.com/#pre-commit-configyaml---top-level
  #[serde(skip_serializing_if = "Option::is_none")]
  pub minimum_pre_commit_version: Option<String>,

  /// A list of hook types which will be used by default when running `pre-commit install` https://pre-commit.com/#pre-commit-configyaml---top-level
  #[merge(strategy = merge_optional_btree_sets)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub default_install_hook_types: Option<BTreeSet<String>>,

  /// Mappings for the default language versions of the current project https://pre-commit.com/#pre-commit-configyaml---top-level
  #[merge(strategy = merge_optional_btree_maps)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub default_language_version: Option<StringBTreeMap>,

  /// The default stages of the current project https://pre-commit.com/#pre-commit-configyaml---top-level
  #[merge(strategy = merge_optional_btree_sets)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub default_stages: Option<BTreeSet<Stage>>,

  /// A file include pattern of the current project https://pre-commit.com/#pre-commit-configyaml---top-level
  #[serde(skip_serializing_if = "Option::is_none")]
  pub files: Option<String>,

  /// A file exclude pattern of the current project https://pre-commit.com/#pre-commit-configyaml---top-level
  #[serde(skip_serializing_if = "Option::is_none")]
  pub exclude: Option<String>,

  /// Whether stop running hooks after a first failure https://pre-commit.com/#pre-commit-configyaml---top-level
  #[serde(skip_serializing_if = "Option::is_none")]
  pub fail_fast: Option<bool>,

  /// pre-commit.ci specific settings https://pre-commit.ci/#configuration
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ci: Option<CiSettings>,

  /// Repository mappings of the current project https://pre-commit.com/#pre-commit-configyaml---top-level
  #[merge(strategy = merge_btree_sets)]
  #[serde(skip_serializing_if = "BTreeSet::is_empty")]
  pub repos: BTreeSet<Repo>,
}

/// A pre-commit repo.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, Eq, PartialOrd, Ord)]
#[serde(untagged)]
pub enum Repo {
  /// Hooks for checking the pre-commit configuration itself. https://pre-commit.com/#meta-hooks
  MetaRepo {
    repo: MetaRepo,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    hooks: BTreeSet<MetaRepoHook>,
  },
  /// Hooks for the local repo https://pre-commit.com/#repository-local-hooks
  LocalRepo {
    repo: LocalRepo,
    /// A list of local hooks https://pre-commit.com/#2-add-a-pre-commit-configuration
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    hooks: BTreeSet<Hook>,
  },
  /// A remote repo
  UriRepo {
    /// A repository url https://pre-commit.com/#2-add-a-pre-commit-configuration
    repo: String,
    /// A revision or tag to clone at https://pre-commit.com/#2-add-a-pre-commit-configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
    /// A list of hook mappings https://pre-commit.com/#pre-commit-configyaml---hooks.
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    hooks: BTreeSet<Hook>,
  },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum MetaRepo {
  Meta,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum MetaRepoId {
  CheckHooksApply,
  CheckUselessExcludes,
  Identity,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, Eq, PartialOrd, Ord)]
pub struct MetaRepoHook {
  pub id: MetaRepoId,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum LocalRepo {
  Local,
}

/// Description for a pre-commit hook. https://pre-commit.com/#pre-commit-configyaml---hooks
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, Eq, Default)]
#[serde(default)]
pub struct Hook {
  /// An identifier of the current hook https://pre-commit.com/#pre-commit-configyaml---hooks
  pub id: String,

  /// A list of additional_dependencies of the current hook https://pre-commit.com/#pre-commit-configyaml---hooks
  #[serde(skip_serializing_if = "Option::is_none")]
  pub additional_dependencies: Option<BTreeSet<String>>,

  /// An additional identifier of the current hook for `pre-commit run <hookid>` https://pre-commit.com/#pre-commit-configyaml---hooks
  #[serde(skip_serializing_if = "Option::is_none")]
  pub alias: Option<String>,

  /// Run the current hook when no files matched https://pre-commit.com/#pre-commit-configyaml---hooks
  #[serde(skip_serializing_if = "Option::is_none")]
  pub always_run: Option<bool>,

  /// List of additional parameters to pass to the current hook https://pre-commit.com/#pre-commit-configyaml---hooks
  #[serde(skip_serializing_if = "Option::is_none")]
  pub args: Option<Vec<String>>,

  /// A command of the current hook https://pre-commit.com/#pre-commit-configyaml---hooks
  #[serde(skip_serializing_if = "Option::is_none")]
  pub entry: Option<String>,

  /// Exclude files that were matched by files.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub exclude: Option<String>,

  /// A list of file types to exclude of the current hook https://pre-commit.com/#pre-commit-configyaml---hooks
  #[serde(skip_serializing_if = "Option::is_none")]
  pub exclude_types: Option<BTreeSet<FileType>>,

  /// Description of the hook. used for metadata purposes only.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,

  /// The pattern of files to run on.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub files: Option<String>,

  /// A language the current hook is written in https://pre-commit.com/#pre-commit-configyaml---hooks
  #[serde(skip_serializing_if = "Option::is_none")]
  pub language: Option<Language>,

  /// Mappings for the default language versions of the current project https://pre-commit.com/#pre-commit-configyaml---top-level
  #[serde(skip_serializing_if = "Option::is_none")]
  pub language_version: Option<String>,

  /// A log file of the current hook
  #[serde(skip_serializing_if = "Option::is_none")]
  pub log_file: Option<String>,

  /// Allows one to indicate a minimum compatible pre-commit version.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub minimum_pre_commit_version: Option<usize>,

  /// Name of the hook - shown during hook execution.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,

  /// Whether to pass filenames to the current hook or not https://pre-commit.com/#pre-commit-configyaml---hooks
  #[serde(skip_serializing_if = "Option::is_none")]
  pub pass_filenames: Option<bool>,

  /// If true this hook will execute using a single process instead of in parallel.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub require_serial: Option<bool>,

  /// A stage of the current hook https://pre-commit.com/#pre-commit-configyaml---hooks
  #[serde(skip_serializing_if = "Option::is_none")]
  pub stages: Option<BTreeSet<Stage>>,

  /// List of file types to run on (AND).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub types: Option<BTreeSet<FileType>>,

  /// List of file types to run on (OR).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub types_or: Option<BTreeSet<FileType>>,

  /// Display an output of the current hook even it passes https://pre-commit.com/#pre-commit-configyaml---hooks
  #[serde(skip_serializing_if = "Option::is_none")]
  pub verbose: Option<bool>,
}

impl PartialOrd for Hook {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(&other))
  }
}

impl Ord for Hook {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.id.cmp(&other.id)
  }
}

/// Settings for [`pre-commit`](https://pre-commit.com)  Can be a preset id, a newly defined configuration, or a boolean to use defaults or to disable pre-commit.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum PreCommitSetting {
  Bool(bool),
  Id(String),
  Config(PreCommitPreset),
}

impl Default for PreCommitSetting {
  fn default() -> Self {
    Self::Bool(true)
  }
}

impl PreCommitSetting {
  pub(crate) fn is_enabled(&self) -> bool {
    !matches!(self, Self::Bool(false))
  }
}
