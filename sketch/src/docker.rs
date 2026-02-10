use crate::*;

pub mod compose;
use compose::{ComposePreset, service::DockerServicePreset};

/// All settings and presets related to Docker.
#[derive(Clone, Debug, Deserialize, Serialize, Merge, PartialEq, Default)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default)]
pub struct DockerConfig {
	/// A map that contains presets for Docker Compose files.
	pub compose_presets: IndexMap<String, ComposePreset>,

	/// A map that contains presets for Docker services.
	pub service_presets: IndexMap<String, DockerServicePreset>,
}
