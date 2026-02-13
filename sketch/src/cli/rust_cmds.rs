use super::*;

#[derive(Subcommand, Debug, Clone)]
pub enum RustCommands {
	Crate {
		/// The output directory for the new crate. Also the name of the generated crate by default.
		dir: PathBuf,

		/// The crate preset to use.
		#[arg(short, long)]
		preset: Option<String>,

		/// The `Cargo.toml` manifest preset to use (overrides the one in the preset if one was selected).
		#[arg(short, long)]
		manifest: Option<String>,

		/// The name of the generated crate (by default, it uses the name of the output dir).
		#[arg(short, long)]
		name: Option<String>,

		#[command(flatten)]
		config: Option<CratePreset>,
	},

	/// Generates a new `Cargo.toml` file from a preset.
	Manifest {
		/// The id of the preset.
		preset: String,

		/// The output path [default: `Cargo.toml`]
		output: Option<PathBuf>,
	},
}

impl RustCommands {
	pub fn execute(self, config: &Config) -> AppResult {
		match self {
			Self::Manifest { output, preset } => {
				let content = config.rust.get_cargo_toml_preset(&preset)?.config;

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
					config.rust.get_crate_preset(&preset_id)?
				} else {
					CratePreset::default()
				};

				if let Some(overrides) = overrides {
					preset.merge(overrides);
				}

				if let Some(manifest_id) = manifest {
					preset.manifest = CargoTomlPresetRef::PresetId(manifest_id);
				}

				let crate_data = preset.process_data(config)?;

				if dir.exists() {
					return Err(anyhow!("Directory `{}` already exists", dir.display()).into());
				}

				crate_data.generate(&dir, name, config)?;
			}
		};

		Ok(())
	}
}
