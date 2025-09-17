use askama::Template;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Template, Serialize, Deserialize, PartialEq, JsonSchema)]
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, Template)]
#[serde(untagged)]
#[template(path = "pre-commit-config.yaml.j2")]
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
pub struct PreCommitConfig {
  pub repos: Vec<PreCommitRepo>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct PreCommitRepo {
  pub path: String,
  pub rev: Option<String>,
  pub hooks: Vec<Value>,
}
