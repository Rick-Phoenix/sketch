use crate::*;
pub(crate) use docker_compose_config::*;

/// All settings and presets related to Docker.
#[derive(Clone, Debug, Deserialize, Serialize, Merge, PartialEq, Default)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default)]
#[serde(deny_unknown_fields)]
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

impl ExtensiblePreset for ComposePreset {
	fn kind() -> PresetKind {
		PresetKind::DockerCompose
	}

	fn get_extended_ids(&self) -> &IndexSet<String> {
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
		if self.extends_presets.is_empty()
			&& !self
				.config
				.services
				.values()
				.any(|s| s.requires_processing())
		{
			return Ok(self.config);
		}

		let mut merged_preset = if self.extends_presets.is_empty() {
			self
		} else {
			self.merge_presets(id, store)?
		};

		for service_data in merged_preset.config.services.values_mut() {
			match service_data {
				ServicePresetRef::PresetId(id) => {
					let mut service_preset = services_store
						.get(id)
						.ok_or(GenError::PresetNotFound {
							kind: PresetKind::DockerService,
							name: id.clone(),
						})?
						.clone();

					if !service_preset.extends_presets.is_empty() {
						service_preset = service_preset.merge_presets(id, services_store)?;
					}

					*service_data = ServicePresetRef::Config(service_preset.into());
				}
				ServicePresetRef::Config(config) => {
					if !config.extends_presets.is_empty() {
						let data = std::mem::take(config);

						*config = data
							.merge_presets("__inlined", services_store)?
							.into();
					}
				}
			};
		}

		Ok(merged_preset.config)
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

impl ExtensiblePreset for DockerServicePreset {
	fn kind() -> PresetKind {
		PresetKind::DockerService
	}

	fn get_extended_ids(&self) -> &IndexSet<String> {
		&self.extends_presets
	}
}
