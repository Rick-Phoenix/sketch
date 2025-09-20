use askama::Template;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
