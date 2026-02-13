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

impl ExtensiblePreset for OxlintPreset {
	fn kind() -> PresetKind {
		PresetKind::Oxlint
	}

	fn get_extended_ids(&self) -> &IndexSet<String> {
		&self.extends_presets
	}
}

impl Default for OxlintPresetRef {
	fn default() -> Self {
		Self::Bool(true)
	}
}

/// Settings for generating an `oxlint` configuration file.
/// It can be set to true/false (to use defaults or to disable it entirely) or to a literal configuration.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
pub enum OxlintPresetRef {
	Bool(bool),
	Id(String),
	Config(OxlintPreset),
}

impl std::str::FromStr for OxlintPresetRef {
	type Err = std::convert::Infallible;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(Self::Id(s.to_string()))
	}
}

impl OxlintPresetRef {
	pub const fn is_enabled(&self) -> bool {
		!matches!(self, Self::Bool(false))
	}
}
