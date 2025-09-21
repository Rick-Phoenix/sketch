use std::fmt::Display;

use indexmap::{IndexMap, IndexSet};
use merge::Merge;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{merge_index_sets, merge_presets, Extensible, GenError, Preset};

fn merge_gitignore(left: &mut GitIgnore, right: GitIgnore) {
  let mut left_as_list = left.as_list();
  let right_as_list = right.as_list();

  let new: Vec<String> = left_as_list.splice(0..0, right_as_list).collect();

  *left = GitIgnore::List(new)
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, Merge, Default)]
#[serde(default)]
pub struct GitignorePreset {
  /// The ids of the extended configurations.
  #[merge(strategy = merge_index_sets)]
  pub extends: IndexSet<String>,

  #[merge(strategy = merge_gitignore)]
  pub config: GitIgnore,
}

impl Extensible for GitignorePreset {
  fn get_extended(&self) -> &IndexSet<String> {
    &self.extends
  }
}

impl GitignorePreset {
  pub fn process_data(
    self,
    id: &str,
    store: &IndexMap<String, GitignorePreset>,
  ) -> Result<GitIgnore, GenError> {
    if self.extends.is_empty() {
      return Ok(self.config);
    }

    let mut processed_ids: IndexSet<String> = IndexSet::new();

    let merged_preset = merge_presets(Preset::Gitignore, id, self, store, &mut processed_ids)?;

    Ok(merged_preset.config)
  }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(untagged)]
/// Settings for a .gitignore file. It can be a preset id or a literal configuration.
pub enum GitIgnoreSetting {
  Id(String),
  Config(GitIgnore),
}

impl Default for GitIgnoreSetting {
  fn default() -> Self {
    Self::Config(GitIgnore::default())
  }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(untagged)]
/// A definition for a gitignore template. It can be a list of strings (to define each element) or a single string (to define the entire file).
pub enum GitIgnore {
  List(Vec<String>),
  String(String),
}

impl GitIgnore {
  pub fn as_list(&self) -> Vec<String> {
    match self {
      GitIgnore::List(items) => items.clone(),
      GitIgnore::String(entire) => entire.split('\n').map(|s| s.to_string()).collect(),
    }
  }
}

impl Display for GitIgnore {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      GitIgnore::List(items) => {
        write!(f, "{}", items.join("\n"))
      }
      GitIgnore::String(entire) => write!(f, "{}", entire),
    }
  }
}

impl Default for GitIgnore {
  fn default() -> Self {
    Self::String(
      r###"
      # Caches
      .task
      .cache 

      # Build Output
      target
      *.js.map
      *.d.ts 
      *.tsbuildInfo
      .out
      .output
      .vercel
      .netlify
      .wrangler
      .svelte-kit
      dist
      build

      # Llm Files
      llms.txt
      llms.md

      # Node Modules
      node_modules

      # Env
      .env
      .env.*
      !.env.example
      !.env.test

      # Temporary Files
      *.tmp
      *.swp
      *.swo
      vite.config.js.timestamp-*
      vite.config.ts.timestamp-*

      # Logs
      logs/
      *.log
      pnpm-debug.log*

      # Operating System Generated Files
      .DS_Store
      Thumbs.db
      desktop.ini

      # Test Reports & Coverage
      coverage/
      lcov-report/
      *.lcov
      .nyc_output/

    "###
        .trim()
        .to_string(),
    )
  }
}
