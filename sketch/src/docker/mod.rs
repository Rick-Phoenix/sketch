use indexmap::IndexMap;
use merge::Merge;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
  docker::compose::{service::DockerServicePreset, ComposePreset},
  merge_index_maps,
};

pub mod compose;

/// All settings and presets related to Docker.
#[derive(Clone, Debug, Deserialize, Serialize, Merge, PartialEq, JsonSchema, Default)]
#[merge(strategy = overwrite_if_some)]
#[serde(default)]
pub struct DockerConfig {
  /// A map that contains presets for Docker Compose files.
  #[merge(strategy = merge_index_maps)]
  pub compose_presets: IndexMap<String, ComposePreset>,

  /// A map that contains presets for Docker services.
  #[merge(strategy = merge_index_maps)]
  pub service_presets: IndexMap<String, DockerServicePreset>,
}
