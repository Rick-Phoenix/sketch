use crate::*;
pub(crate) use docker_compose_config::*;

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

/// A preset for Docker Compose files.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default, Merge)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default)]
pub struct ComposePreset {
	/// The list of extended presets.
	pub extends_presets: IndexSet<String>,

	#[serde(flatten)]
	pub config: ComposeFile,
}

impl Extensible for ComposePreset {
	fn get_extended(&self) -> &IndexSet<String> {
		&self.extends_presets
	}
}

impl ComposePreset {
	pub fn process_data(
		self,
		id: &str,
		store: &IndexMap<String, Self>,
		services_store: &IndexMap<String, DockerServicePreset>,
	) -> Result<ComposeFile, GenError> {
		let mut processed_ids: IndexSet<String> = IndexSet::new();

		// Must not skip here in case of no extended presets, because services must be processed regardless
		let merged_preset = if self.extends_presets.is_empty() {
			self
		} else {
			merge_presets(Preset::DockerCompose, id, self, store, &mut processed_ids)?
		};

		let mut config = merged_preset.config;

		for (_, service_data) in config.services.iter_mut() {
			match service_data {
				ServicePresetRef::Id(id) => {
					let preset = services_store
						.get(id)
						.ok_or(GenError::PresetNotFound {
							kind: Preset::DockerService,
							name: id.clone(),
						})?
						.clone();

					*service_data = ServicePresetRef::Config(
						process_docker_service_preset(preset, id, services_store)?.into(),
					);
				}
				ServicePresetRef::Config(config) => {
					if !config.extends_presets.is_empty() {
						let data = std::mem::take(config);

						*service_data = ServicePresetRef::Config(
							process_docker_service_preset(*data, "__inlined", services_store)?
								.into(),
						);
					}
				}
			};
		}

		Ok(config)
	}
}

#[derive(Clone, Debug)]
pub struct ServiceFromCli {
	pub preset_id: String,
	pub name: Option<String>,
}

impl ServiceFromCli {
	pub fn from_cli(s: &str) -> Result<Self, String> {
		let s = s.trim();

		let mut service_name: Option<String> = None;
		let mut preset_name: Option<String> = None;

		let parts: Vec<&str> = s.split(',').collect();

		if parts.len() == 1 {
			preset_name = Some(parts[0].to_string());
		} else {
			for part in parts {
				let (key, val) = parse_single_key_value_pair("--service", part)?;
				if key == "id" {
					preset_name = Some(val.to_string());
				} else if key == "name" {
					service_name = Some(val.to_string());
				} else {
					return Err(format!("Unknown parameter `{key}` in --service"));
				}
			}
		}

		Ok(Self {
			name: service_name,
			preset_id: preset_name.ok_or("No preset id for docker service has been provided")?,
		})
	}
}

impl Extensible for DockerServicePreset {
	fn get_extended(&self) -> &IndexSet<String> {
		&self.extends_presets
	}
}

pub fn process_docker_service_preset(
	preset: DockerServicePreset,
	id: &str,
	store: &IndexMap<String, DockerServicePreset>,
) -> Result<DockerServicePreset, GenError> {
	if preset.extends_presets.is_empty() {
		return Ok(preset);
	}

	let mut processed_ids: IndexSet<String> = IndexSet::new();

	let merged_preset =
		merge_presets(Preset::DockerService, id, preset, store, &mut processed_ids)?;

	Ok(merged_preset)
}
