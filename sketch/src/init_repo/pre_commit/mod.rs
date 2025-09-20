mod pre_commit_elements;

use std::collections::BTreeSet;

use pre_commit_elements::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::StringBTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct PreCommitConfig {
  pub repos: BTreeSet<Repo>,
  pub ci: Option<CiSettings>,
  pub default_install_hook_types: Option<BTreeSet<String>>,
  pub default_language_version: Option<StringBTreeMap>,
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, Eq)]
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
  Config(PreCommitConfig),
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
