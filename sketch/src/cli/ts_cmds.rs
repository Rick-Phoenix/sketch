use super::*;

use globset::{Glob, GlobSetBuilder};
use walkdir::WalkDir;

mod ts_barrel;
use ts_barrel::*;

use crate::{
	ts::{
		oxlint::OxlintPresetRef,
		package::{PackageType, TsPackagePreset, TsPackagePresetRef, TsPackageSetup},
		pnpm::PnpmWorkspace,
		vitest::VitestPresetRef,
	},
	*,
};

#[derive(Subcommand, Debug, Clone)]
pub enum TsCommands {
	/// Generates a new typescript monorepo
	Monorepo {
		/// The root directory for the new monorepo.
		dir: PathBuf,

		/// The `pnpm-workspace.yaml` preset to use for the new monorepo. If it's unset and `pnpm` is the chosen package manager, the default preset will be used.
		#[arg(short, long, value_name = "PRESET_ID")]
		pnpm: Option<String>,

		/// The id of the package preset to use for the root package. If unset, the default preset is used, along with the values set via cli flags.
		#[arg(short, long, value_name = "PRESET_ID")]
		root_package: Option<String>,

		#[command(flatten)]
		root_package_overrides: Option<TsPackagePreset>,

		/// Installs the dependencies with the chosen package manager
		#[arg(short, long)]
		install: bool,
	},

	/// Generates a new typescript package
	Package {
		/// The root directory for the new package. Defaults to the package name.
		dir: Option<PathBuf>,

		/// The package preset to use. If unset, the default preset is used, along with the values set via cli flags
		#[arg(short, long, value_name = "ID")]
		preset: Option<String>,

		/// An optional list of tsconfig files where the new tsconfig file will be added as a reference.
		#[arg(short, long)]
		update_tsconfig: Vec<PathBuf>,

		/// Installs the dependencies with the chosen package manager
		#[arg(short, long)]
		install: bool,

		/// The vitest preset to use. It can be set to `default` to use the default preset.
		#[arg(long, value_name = "ID")]
		vitest: Option<String>,

		#[command(flatten)]
		package_config: Option<TsPackagePreset>,
	},

	/// Creates a barrel file
	Barrel {
		#[command(flatten)]
		args: TsBarrelArgs,
	},

	/// Generates a `tsconfig.json` file from a preset.
	Config {
		/// The preset id
		preset: String,

		/// The output path of the generated file [default: `tsconfig.json`]
		output: Option<PathBuf>,
	},
}

impl TsCommands {
	pub(crate) async fn execute(
		self,
		mut config: Config,
		cli_vars: &IndexMap<String, Value>,
	) -> Result<(), AppError> {
		let overwrite = config.can_overwrite();
		let typescript = config.typescript.get_or_insert_default();

		match self {
			Self::Config { output, preset } => {
				let content = typescript.get_tsconfig_preset(&preset)?.config;

				let output = output.unwrap_or_else(|| "tsconfig.json".into());

				create_parent_dirs(&output)?;

				serialize_json(&content, &output, overwrite)?;
			}
			Self::Barrel { args } => {
				args.create_ts_barrel(overwrite)?;
			}
			Self::Monorepo {
				install,
				root_package_overrides,
				root_package,
				dir: out_dir,
				pnpm,
			} => {
				let mut root_package = if let Some(id) = root_package {
					typescript.get_package_preset(&id)?
				} else {
					let mut package = TsPackagePreset::default();
					package.oxlint = Some(OxlintPresetRef::Bool(true));
					package
				};

				if let Some(overrides) = root_package_overrides {
					root_package.merge(overrides);
				}

				let package_manager = *typescript.package_manager.get_or_insert_default();

				let pnpm_config = if let Some(id) = pnpm {
					Some(typescript.get_pnpm_preset(&id)?.config)
				} else if package_manager.is_pnpm() {
					Some(PnpmWorkspace::default())
				} else {
					None
				};

				config
					.create_ts_package(TsPackageSetup {
						data: TsPackagePresetRef::Preset(root_package),
						pkg_root: &out_dir,
						tsconfig_files_to_update: vec![],
						cli_vars,
						package_type: PackageType::MonorepoRoot { pnpm: pnpm_config },
						install,
					})
					.await?;
			}
			Self::Package {
				preset,
				package_config,
				update_tsconfig,
				dir,
				vitest,
				install,
			} => {
				let mut package = if let Some(id) = preset {
					typescript.get_package_preset(&id)?
				} else {
					TsPackagePreset::default()
				};

				if let Some(overrides) = package_config {
					package.merge(overrides);
				}

				if let Some(vitest) = vitest {
					package.vitest = Some(VitestPresetRef::PresetId(vitest))
				}

				let package_dir = dir.unwrap_or_else(|| {
					package
						.name
						.as_deref()
						.unwrap_or("new_package")
						.into()
				});

				config
					.create_ts_package(TsPackageSetup {
						data: TsPackagePresetRef::Preset(package),
						pkg_root: &package_dir,
						tsconfig_files_to_update: update_tsconfig,
						cli_vars,
						package_type: PackageType::Normal,
						install,
					})
					.await?;
			}
		}

		Ok(())
	}
}
