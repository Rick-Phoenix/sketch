use std::fmt::Display;

use indexmap::{IndexMap, IndexSet};
use merge::Merge;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{merge_index_sets, merge_presets, Extensible, GenError, Preset};

fn merge_gitignore(left: &mut GitIgnore, right: GitIgnore) {
  let left_as_list = left.as_list();
  let right_as_list = right.as_list();

  let new: Vec<String> = right_as_list
    .into_iter()
    .chain(left_as_list.into_iter())
    .collect();

  *left = GitIgnore::List(new)
}

/// A preset for a `.gitignore` file.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, Merge, Default)]
#[serde(default)]
pub struct GitignorePreset {
  /// The ids of the extended presets.
  #[merge(strategy = merge_index_sets)]
  pub extends: IndexSet<String>,

  #[merge(strategy = merge_gitignore)]
  pub content: GitIgnore,
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
      return Ok(self.content);
    }

    let mut processed_ids: IndexSet<String> = IndexSet::new();

    let merged_preset = merge_presets(Preset::Gitignore, id, self, store, &mut processed_ids)?;

    Ok(merged_preset.content)
  }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(untagged)]
/// Settings for a .gitignore file. It can be a preset id, a list of strings (to define each element) or a single string (to define the entire file)
pub enum GitIgnoreSetting {
  Id(String),
  Config(GitIgnore),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(untagged)]
/// A definition for a gitignore template. It can be a list of strings (to define each element) or a single string (to define the entire file).
pub enum GitIgnore {
  List(Vec<String>),
  String(String),
}

impl Default for GitIgnore {
  fn default() -> Self {
    Self::String(Default::default())
  }
}

impl GitIgnore {
  pub fn as_list(&self) -> Vec<String> {
    match self {
      GitIgnore::List(items) => items.clone(),
      GitIgnore::String(entire) => entire.trim().split('\n').map(|s| s.to_string()).collect(),
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

pub(crate) const DEFAULT_GITIGNORE: &str = r###"
# caches
.task
.cache 

# build output
target
*.js.map
*.d.ts 
*.tsbuildinfo
.out
.output
.vercel
.netlify
.wrangler
.svelte-kit
dist
build

# llm files
llms.txt
llms.md

# node modules
node_modules

# env
.env
.env.*
!.env.example
!.env.test

# temporary files
*.tmp
*.swp
*.swo
vite.config.js.timestamp-*
vite.config.ts.timestamp-*

# logs
logs/
*.log
pnpm-debug.log*

# operating system generated files
.ds_store
thumbs.db
desktop.ini

# test reports & coverage
coverage/
lcov-report/
*.lcov
.nyc_output/
"###;
