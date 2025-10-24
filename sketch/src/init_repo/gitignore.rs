use std::fmt::Display;

use indexmap::{IndexMap, IndexSet};
use merge::Merge;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{merge_index_sets, merge_presets, Extensible, GenError, Preset};

fn merge_gitignore(left: &mut GitIgnore, right: GitIgnore) {
  match left {
    GitIgnore::List(left_items) => match right {
      GitIgnore::List(right_items) => {
        for entry in right_items.into_iter().rev() {
          left_items.insert(0, entry);
        }
      }
      GitIgnore::String(mut right_string) => {
        for entry in left_items.iter() {
          right_string.push('\n');
          right_string.push_str(entry);
        }

        *left = GitIgnore::String(right_string);
      }
    },
    GitIgnore::String(left_string) => match right {
      GitIgnore::List(right_items) => {
        if !right_items.is_empty() {
          left_string.insert(0, '\n');
          let len = right_items.len();

          for (i, entry) in right_items.into_iter().rev().enumerate() {
            left_string.insert_str(0, entry.as_str());

            if i != len - 1 {
              left_string.insert(0, '\n');
            }
          }
        }
      }
      GitIgnore::String(right_string) => {
        left_string.insert(0, '\n');
        left_string.insert_str(0, &right_string);
      }
    },
  };
}

/// A preset for a `.gitignore` file.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, Merge, Default)]
#[serde(default)]
pub struct GitignorePreset {
  /// The ids of the extended presets.
  #[merge(strategy = merge_index_sets)]
  pub extends_presets: IndexSet<String>,

  #[merge(strategy = merge_gitignore)]
  pub content: GitIgnore,
}

impl Extensible for GitignorePreset {
  fn get_extended(&self) -> &IndexSet<String> {
    &self.extends_presets
  }
}

impl GitignorePreset {
  pub fn process_data(
    self,
    id: &str,
    store: &IndexMap<String, GitignorePreset>,
  ) -> Result<GitIgnore, GenError> {
    if self.extends_presets.is_empty() {
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
