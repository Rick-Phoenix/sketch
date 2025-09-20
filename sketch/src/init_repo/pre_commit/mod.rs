mod pre_commit_elements;

use std::collections::BTreeSet;

use indexmap::{IndexMap, IndexSet};
use maplit::btreeset;
use merge::Merge;
use pre_commit_elements::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
  merge_btree_sets, merge_index_sets, merge_nested, merge_optional_btree_maps,
  merge_optional_btree_sets, merge_presets, overwrite_if_some, Extensible, GenError, Preset,
  StringBTreeMap,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, Merge, Default)]
#[serde(default)]
pub struct PreCommitPreset {
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, Merge)]
#[merge(strategy = overwrite_if_some)]
pub struct PreCommitConfig {
  #[merge(strategy = merge_btree_sets)]
  pub repos: BTreeSet<Repo>,
  pub ci: Option<CiSettings>,
  #[merge(strategy = merge_optional_btree_sets)]
  pub default_install_hook_types: Option<BTreeSet<String>>,
  #[merge(strategy = merge_optional_btree_maps)]
  pub default_language_version: Option<StringBTreeMap>,
  #[merge(strategy = merge_optional_btree_sets)]
  pub default_stages: Option<BTreeSet<Stage>>,
  pub files: Option<String>,
  pub exclude: Option<String>,
  pub fail_fast: Option<bool>,
  pub minimum_pre_commit_version: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, Eq, PartialOrd, Ord)]
#[serde(untagged)]
pub enum Repo {
  MetaRepo {
    repo: MetaRepo,
    hooks: Option<BTreeSet<MetaRepoHook>>,
  },
  LocalRepo {
    repo: LocalRepo,
    hooks: Option<BTreeSet<Hook>>,
  },
  UriRepo {
    repo: Option<String>,
    rev: Option<String>,
    hooks: Option<BTreeSet<Hook>>,
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, Eq, Default)]
pub struct Hook {
  pub id: String,
  pub additional_dependencies: Option<BTreeSet<String>>,
  pub alias: Option<String>,
  pub always_run: Option<bool>,
  pub args: Option<Vec<String>>,
  pub entry: Option<String>,
  pub exclude: Option<String>,
  pub exclude_types: Option<BTreeSet<FileType>>,
  pub description: Option<String>,
  pub files: Option<String>,
  pub language: Option<Language>,
  pub language_version: Option<String>,
  pub log_file: Option<String>,
  pub minimum_pre_commit_version: Option<usize>,
  pub name: Option<String>,
  pub pass_filenames: Option<bool>,
  pub require_serial: Option<bool>,
  pub stages: Option<BTreeSet<Stage>>,
  pub types: Option<BTreeSet<FileType>>,
  pub types_or: Option<BTreeSet<FileType>>,
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
