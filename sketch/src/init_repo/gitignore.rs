use super::*;

impl Config {
	pub fn get_gitignore_preset(&self, id: &str) -> AppResult<GitignorePreset> {
		self.gitignore_presets
			.get(id)
			.ok_or_else(|| AppError::PresetNotFound {
				kind: PresetKind::Gitignore,
				name: id.to_string(),
			})?
			.clone()
			.merge_presets(id, &self.gitignore_presets)
	}
}

impl Merge for GitIgnore {
	fn merge(&mut self, right: Self) {
		match self {
			Self::List(left_items) => match right {
				Self::List(right_items) => {
					left_items.extend(right_items);
				}
				Self::String(right_string) => {
					left_items.extend(right_string.lines().map(|s| s.to_string()));
				}
			},
			Self::String(left_string) => match right {
				Self::List(right_items) => {
					let mut new_list: Vec<String> = Vec::new();

					new_list.extend(left_string.lines().map(|s| s.to_string()));
					new_list.extend(right_items);

					*self = Self::List(new_list);
				}
				Self::String(right_string) => {
					let mut new_list: Vec<String> = Vec::new();

					new_list.extend(left_string.lines().map(|s| s.to_string()));
					new_list.extend(right_string.lines().map(|s| s.to_string()));

					*self = Self::List(new_list);
				}
			},
		};
	}
}

/// A preset for a `.gitignore` file.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Merge, Default)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct GitignorePreset {
	/// The ids of the extended presets.
	#[serde(default)]
	#[merge(skip)]
	pub extends_presets: IndexSet<String>,

	pub content: GitIgnore,
}

impl ExtensiblePreset for GitignorePreset {
	fn kind() -> PresetKind {
		PresetKind::Gitignore
	}

	fn extended_ids(&mut self) -> &mut IndexSet<String> {
		&mut self.extends_presets
	}
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
/// Settings for a .gitignore file. It can be a preset id, a list of strings (to define each element) or a single string (to define the entire file)
pub enum GitIgnorePresetRef {
	PresetId(String),
	Preset(GitignorePreset),
}

impl GitIgnorePresetRef {
	/// Returns `true` if the git ignore preset ref is [`PresetId`].
	///
	/// [`PresetId`]: GitIgnorePresetRef::PresetId
	#[must_use]
	pub const fn is_preset_id(&self) -> bool {
		matches!(self, Self::PresetId(..))
	}
}

impl std::str::FromStr for GitIgnorePresetRef {
	type Err = std::convert::Infallible;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(Self::PresetId(s.to_string()))
	}
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
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
			Self::List(items) => {
				write!(f, "{}", items.join("\n"))
			}
			Self::String(entire) => write!(f, "{entire}"),
		}
	}
}

pub(crate) const DEFAULT_GITIGNORE: &str = r"
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
";
