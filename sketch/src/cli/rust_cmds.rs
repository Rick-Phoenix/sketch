use super::*;

#[derive(Subcommand, Debug, Clone)]
pub enum RustCommands {
	Crate {
		dir: PathBuf,

		#[arg(short, long)]
		preset: Option<String>,

		#[arg(short, long)]
		manifest: Option<String>,

		#[arg(short, long)]
		name: Option<String>,

		#[command(flatten)]
		config: Option<CratePreset>,
	},

	Manifest {
		preset: String,

		output: Option<PathBuf>,
	},
}

impl RustCommands {
	pub fn execute(self, config: &Config) -> AppResult {
		match self {
			Self::Manifest { output, preset } => {
				let content = config
					.rust_presets
					.manifest_presets
					.get(&preset)
					.ok_or_else(|| AppError::PresetNotFound {
						kind: PresetKind::RustCrate,
						name: preset.clone(),
					})?
					.clone()
					.merge_presets(&preset, &config.rust_presets.manifest_presets)?
					.config;

				let output_path = output.unwrap_or_else(|| "Cargo.toml".into());

				write_file(
					&output_path,
					&content.as_document().to_string(),
					config.can_overwrite(),
				)?;
			}
			Self::Crate {
				dir,
				name,
				preset: preset_id,
				config: overrides,
				manifest,
			} => {
				let mut preset = if let Some(preset_id) = preset_id {
					config
						.rust_presets
						.crate_presets
						.get(&preset_id)
						.ok_or_else(|| AppError::PresetNotFound {
							kind: PresetKind::RustCrate,
							name: preset_id,
						})?
						.clone()
				} else {
					CratePreset::default()
				};

				if let Some(overrides) = overrides {
					preset.merge(overrides);
				}

				if let Some(manifest_id) = manifest {
					preset.manifest = CargoTomlPresetRef::PresetId(manifest_id);
				}

				let crate_data = preset.process_data(
					&config.rust_presets.manifest_presets,
					&config.gitignore_presets,
				)?;

				if dir.exists() && !config.can_overwrite() {
					return Err(anyhow!("Directory `{}` already exists", dir.display()).into());
				}

				crate_data.generate(&dir, name, config)?;
			}
		};

		Ok(())
	}
}
