use super::*;
use crate::cli::parsers::parse_key_value_pairs;
pub(crate) use ::ts_config::*;

pub(crate) mod tsconfig_defaults;

/// A preset for a `tsconfig` file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, Merge)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default)]
pub struct TsConfigPreset {
	/// The list of extended presets.
	pub extends_presets: IndexSet<String>,

	#[serde(flatten)]
	pub tsconfig: TsConfig,
}

impl Extensible for TsConfigPreset {
	fn get_extended(&self) -> &IndexSet<String> {
		&self.extends_presets
	}
}

impl TsConfigPreset {
	pub fn process_data(
		self,
		id: &str,
		store: &IndexMap<String, Self>,
	) -> Result<TsConfig, GenError> {
		if self.extends_presets.is_empty() {
			return Ok(self.tsconfig);
		}

		let mut processed_ids: IndexSet<String> = IndexSet::new();

		let merged_preset = merge_presets(Preset::TsConfig, id, self, store, &mut processed_ids)?;

		Ok(merged_preset.tsconfig)
	}
}

/// The kind of data for a [`TsConfig`]. It can be a string indicating a preset it, or a full configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
pub enum TsConfigKind {
	Id(String),
	Config(TsConfigPreset),
}

impl Default for TsConfigKind {
	fn default() -> Self {
		Self::Config(TsConfigPreset::default())
	}
}

/// A struct representing instructions for generating a tsconfig file.
/// If the output path is relative, it will be joined to the root path of its package.
#[derive(Deserialize, Debug, Clone, Serialize, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct TsConfigDirective {
	/// The output path of the config file [default: `tsconfig.json`]
	pub output: Option<String>,

	/// The configuration for the output file. It can be a preset id or a new definition.
	pub config: Option<TsConfigKind>,
}

impl Default for TsConfigDirective {
	fn default() -> Self {
		Self {
			output: Some("tsconfig.json".to_string()),
			config: Some(TsConfigKind::default()),
		}
	}
}

impl TsConfigDirective {
	pub(crate) fn from_cli(s: &str) -> Result<Self, String> {
		let mut directive: Self = Default::default();

		let pairs = parse_key_value_pairs("TsConfigDirective", s)?;

		for (key, val) in pairs {
			match key {
				"output" => {
					directive.output = if val.is_empty() {
						None
					} else {
						Some(val.to_string())
					}
				}
				"id" => {
					directive.config = if val.is_empty() {
						None
					} else {
						Some(TsConfigKind::Id(val.to_string()))
					}
				}
				_ => return Err(format!("Invalid key for TsConfigDirective: {key}")),
			};
		}

		Ok(directive)
	}
}
