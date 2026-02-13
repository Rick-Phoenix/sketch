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

impl DockerConfig {
	pub fn get_file_preset(&self, id: &str) -> AppResult<ComposePreset> {
		Ok(self
			.compose_presets
			.get(id)
			.ok_or_else(|| AppError::PresetNotFound {
				kind: PresetKind::ComposeFile,
				name: id.to_string(),
			})?
			.clone())
	}

	pub fn get_service_preset(&self, id: &str) -> AppResult<DockerServicePreset> {
		self.service_presets
			.get(id)
			.ok_or_else(|| AppError::PresetNotFound {
				kind: PresetKind::DockerService,
				name: id.to_string(),
			})?
			.clone()
			.merge_presets(id, &self.service_presets)
	}
}

/// A preset for Docker Compose files.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default, Merge)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default)]
pub struct ComposePreset {
	/// The list of extended presets.
	#[merge(skip)]
	pub extends_presets: IndexSet<String>,

	#[serde(flatten)]
	pub config: ComposeFile,
}

impl ExtensiblePreset for ComposePreset {
	fn kind() -> PresetKind {
		PresetKind::ComposeFile
	}

	fn extended_ids(&mut self) -> &mut IndexSet<String> {
		&mut self.extends_presets
	}
}

impl ComposePreset {
	pub fn process_data(self, id: &str, config: &DockerConfig) -> Result<ComposeFile, AppError> {
		if self.extends_presets.is_empty()
			&& !self
				.config
				.services
				.values()
				.any(|s| s.requires_processing())
		{
			return Ok(self.config);
		}

		let mut merged_preset = self.merge_presets(id, &config.compose_presets)?;

		for service_data in merged_preset.config.services.values_mut() {
			match service_data {
				ServicePresetRef::PresetId(id) => {
					let service_preset = config.get_service_preset(id)?;

					*service_data = ServicePresetRef::Config(service_preset.into());
				}
				ServicePresetRef::Config(preset) => {
					if !preset.extends_presets.is_empty() {
						let mut data = std::mem::take(preset);

						*data = data.merge_presets("__inlined", &config.service_presets)?;

						*preset = data;
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

	fn extended_ids(&mut self) -> &mut IndexSet<String> {
		&mut self.extends_presets
	}
}
