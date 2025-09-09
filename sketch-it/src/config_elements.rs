use std::fmt::Display;

use askama::Template;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize, Default, Clone, Copy, ValueEnum)]
pub enum VersionRange {
  Patch,
  #[default]
  Minor,
  Exact,
}

impl VersionRange {
  pub fn create(&self, version: String) -> String {
    match self {
      VersionRange::Patch => format!("~{}", version),
      VersionRange::Minor => format!("^{}", version),
      VersionRange::Exact => version,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SharedOutDir {
  Bool(bool),
  Name(String),
}

impl Default for SharedOutDir {
  fn default() -> Self {
    Self::Name(".out".to_string())
  }
}

impl SharedOutDir {
  pub fn get_name(&self) -> Option<String> {
    match self {
      Self::Bool(v) => {
        if *v {
          Some(".out".to_string())
        } else {
          None
        }
      }
      Self::Name(v) => Some(v.clone()),
    }
  }
}

#[derive(Debug, Template, Serialize, Deserialize, Clone)]
#[template(path = "oxlint.json.j2")]
#[serde(untagged)]
pub enum OxlintConfig {
  Bool(bool),
  Text(String),
}

impl Default for OxlintConfig {
  fn default() -> Self {
    Self::Bool(true)
  }
}

#[derive(Clone, Debug, Template, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Template, Default, Serialize, Deserialize)]
#[template(path = "pre-commit-config.yaml.j2")]
pub struct PreCommitConfig {
  pub repos: Vec<PreCommitRepo>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PreCommitRepo {
  pub path: String,
  pub rev: Option<String>,
  pub hooks: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default, ValueEnum, Copy)]
#[serde(rename_all = "lowercase")]
pub enum PackageManager {
  #[default]
  Pnpm,
  Npm,
  Deno,
  Bun,
  Yarn,
}

impl Display for PackageManager {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      PackageManager::Pnpm => {
        write!(f, "pnpm")
      }
      PackageManager::Npm => {
        write!(f, "npm")
      }
      PackageManager::Deno => {
        write!(f, "deno")
      }
      PackageManager::Bun => {
        write!(f, "bun")
      }
      PackageManager::Yarn => {
        write!(f, "yarn")
      }
    }
  }
}
