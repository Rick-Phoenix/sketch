use crate::*;

use globset::{Glob, GlobSetBuilder};
use tera::{Context, Error, Map, Tera, Value as TeraValue};
use walkdir::WalkDir;

pub(crate) mod custom_templating;

pub(crate) mod tera_filters;
use tera_filters::*;

pub(crate) mod tera_functions;
use tera_functions::*;

pub(crate) mod tera_setup;
use tera_setup::*;

pub(crate) fn templates_dir() -> PathBuf {
	PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/templates"))
}

/// A reference to a templating preset, or a new preset definition.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
pub enum TemplatingPresetReference {
	/// A reference to a templating preset, with some optional context
	Preset {
		/// The id of the preset to select.
		preset_id: String,
		/// Additional context for the templates in this preset. It overrides previously set values, but not values set via the cli.
		#[serde(default)]
		context: IndexMap<String, Value>,
	},
	/// The definition for a new templating preset.
	Definition(TemplatingPreset),
}

impl FromStr for TemplatingPresetReference {
	type Err = std::convert::Infallible;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(Self::Preset {
			preset_id: s.to_string(),
			context: Default::default(),
		})
	}
}

impl TemplatingPresetReference {
	pub fn resolve(self, store: &IndexMap<String, TemplatingPreset>) -> Result<Self, GenError> {
		let mut preset_id: Option<String> = None;

		let mut content = match self {
			Self::Preset {
				preset_id: id,
				context,
			} => {
				preset_id = Some(id.clone());

				let mut data = store
					.get(&id)
					.ok_or_else(|| GenError::PresetNotFound {
						kind: PresetKind::Templates,
						name: id,
					})?
					.clone();

				data.context.extend(context);

				data
			}
			Self::Definition(data) => data,
		};

		content = content.merge_presets(preset_id.as_deref().unwrap_or("__inlined"), store)?;

		Ok(Self::Definition(content))
	}
}

/// A templating preset. It stores information about one or many templates, such as their source, output paths and contextual variables.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Default, Merge)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct TemplatingPreset {
	/// The list of extended preset IDs.
	pub extends_presets: IndexSet<String>,

	/// The list of templates for this preset. Each element can be an individual template or a path to a directory inside `templates_dir` to render all the templates inside of it.
	pub templates: Vec<TemplateKind>,

	// Context on templating presets may seem redundant, but it is useful because it gathers
	// multiple templates and may set a context for them
	/// Additional context for the templates in this preset. It overrides previously set values, but not values set via the cli.
	pub context: IndexMap<String, Value>,
}

impl ExtensiblePreset for TemplatingPreset {
	fn kind() -> PresetKind {
		PresetKind::Templates
	}

	fn get_extended_ids(&self) -> &IndexSet<String> {
		&self.extends_presets
	}
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
pub enum TemplateKind {
	/// The data for a single template.
	Single(TemplateData),

	/// A path to a directory inside `templates_dir`, where all templates will be recursively extracted and rendered in the output directory, following the same file tree structure.
	Structured(StructuredPreset),

	/// A preset defined in a git repository.
	Remote(RemotePreset),
}

/// A preset defined in a git repository.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct RemotePreset {
	/// The link of the repo where the preset is defined
	repo: String,
	/// A list of glob patterns for the templates to exclude
	#[serde(default)]
	exclude: Vec<String>,
}

/// A structured preset. It points to a directory within `templates_dir`, and optionally adds additional context. All of the templates inside the specified directory will be recursively rendered in the destination directory, with the same exact directory structure and names. If a template file ends with a `jinja` extension such as `.j2`, that gets stripped automatically.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct StructuredPreset {
	/// A relative path to a directory starting from `templates_dir`
	dir: PathBuf,
	/// A list of glob patterns for the templates to exclude
	#[serde(default)]
	exclude: Vec<String>,
}

/// The types of configuration values for a template's data.
/// It can either be an id (which points to the key used to store a literal template in the config, or to a file path starting from the root of the templates directory specified in the config.)
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
pub enum TemplateRef {
	/// A literal definition for a template.
	Inline {
		/// The id of the newly created template. Mostly useful for organizational and debugging purposes.
		name: String,
		/// The content of the new template.
		content: String,
	},
	/// An id pointing to a template defined in a configuration file or inside `templates_dir`.
	Id(String),
}

impl TemplateRef {
	pub fn name(&self) -> &str {
		match self {
			Self::Inline { name, .. } | Self::Id(name) => name,
		}
	}
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum TemplateOutputKind {
	/// Render the output to stdout
	#[serde(skip)]
	Stdout,
	/// Render the output to a file
	#[serde(untagged)]
	Path(PathBuf),
}

/// The data for outputting a new template.
/// The context specified here will override the global context (but not the variables set via cli).
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct TemplateData {
	/// The definition or id for the template to use.
	pub template: TemplateRef,
	/// The output path for the generated file.
	pub output: TemplateOutputKind,
}
