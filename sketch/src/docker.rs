use crate::*;

pub mod compose;
use compose::{ComposePreset, service::DockerServicePreset};

/// All settings and presets related to Docker.
#[derive(Clone, Debug, Deserialize, Serialize, Merge, PartialEq, JsonSchema, Default)]
#[merge(with = overwrite_if_some)]
#[serde(default)]
pub struct DockerConfig {
	/// A map that contains presets for Docker Compose files.
	#[merge(with = IndexMap::extend)]
	pub compose_presets: IndexMap<String, ComposePreset>,

	/// A map that contains presets for Docker services.
	#[merge(with = IndexMap::extend)]
	pub service_presets: IndexMap<String, DockerServicePreset>,
}
