use askama::Template;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Template, Serialize, Deserialize, PartialEq, JsonSchema)]
#[template(path = "repo_root/gitignore.j2")]
#[serde(untagged)]
/// A definition for a gitignore template. It can be a list of strings (to append to the defaults) or a single string to define the entire content of the file.
pub enum GitIgnore {
  Additions(Vec<String>),
  Replacement(String),
}

impl Default for GitIgnore {
  fn default() -> Self {
    Self::Additions(Default::default())
  }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, Template)]
#[serde(untagged)]
#[template(path = "repo_root/pre-commit-config.yaml.j2")]
/// The setting for the .pre-commit-config.yaml template. It can be a boolean (to use the defaults with `true` or to disable pre-commit entirely with `false`) or a literal configuration for the file.
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

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, JsonSchema)]
/// Configuration for .pre-commit-config.yaml.
pub struct PreCommitConfig {
  pub repos: Vec<PreCommitRepo>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, JsonSchema)]
/// Configuration for a .pre-commit-config.yaml repo.
pub struct PreCommitRepo {
  pub path: String,
  pub rev: Option<String>,
  pub hooks: Vec<Value>,
}
