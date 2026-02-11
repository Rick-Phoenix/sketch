use super::*;
pub(crate) use oxlint_config::*;

/// A preset for `.oxlintrc.json`
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default, Eq, Merge)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default)]
pub struct OxlintPreset {
	/// The list of extended presets.
	pub extends_presets: IndexSet<String>,

	#[serde(flatten)]
	pub config: OxlintConfig,
}

impl Extensible for OxlintPreset {
	fn get_extended(&self) -> &IndexSet<String> {
		&self.extends_presets
	}
}

impl OxlintPreset {
	pub fn process_data(
		self,
		id: &str,
		store: &IndexMap<String, Self>,
	) -> Result<OxlintConfig, GenError> {
		if self.extends_presets.is_empty() {
			return Ok(self.config);
		}

		let mut processed_ids: IndexSet<String> = IndexSet::new();

		let merged_preset = merge_presets(Preset::Oxlint, id, self, store, &mut processed_ids)?;

		Ok(merged_preset.config)
	}
}

impl Default for OxlintConfigSetting {
	fn default() -> Self {
		Self::Bool(true)
	}
}

/// Settings for generating an `oxlint` configuration file.
/// It can be set to true/false (to use defaults or to disable it entirely) or to a literal configuration.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
pub enum OxlintConfigSetting {
	Bool(bool),
	Id(String),
	Config(OxlintPreset),
}

impl std::str::FromStr for OxlintConfigSetting {
	type Err = std::convert::Infallible;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(Self::Id(s.to_string()))
	}
}

impl OxlintConfigSetting {
	pub const fn is_enabled(&self) -> bool {
		!matches!(self, Self::Bool(false))
	}
}
