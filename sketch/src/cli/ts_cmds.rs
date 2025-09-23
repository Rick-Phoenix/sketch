use std::path::PathBuf;

use clap::Subcommand;
use merge::Merge;

use crate::{
  cli::cli_elements::PackageKindFlag,
  exec::launch_command,
  fs::{get_cwd, serialize_json},
  ts::{
    oxlint::OxlintConfigSetting,
    package::{PackageConfig, PackageData},
    vitest::VitestConfigKind,
  },
  Config, GenError, *,
};

pub(crate) async fn handle_ts_commands(
  mut config: Config,
  command: TsCommands,
  out_dir: PathBuf,
) -> Result<(), GenError> {
  let overwrite = config.can_overwrite();
  let typescript = config.typescript.get_or_insert_default();

  match command {
    TsCommands::TsConfig { output, preset } => {
      let content = typescript
        .ts_config_presets
        .get(&preset)
        .ok_or(GenError::PresetNotFound {
          kind: Preset::TsConfig,
          name: preset.clone(),
        })?
        .clone()
        .process_data(preset.as_str(), &typescript.ts_config_presets)?;

      let output = out_dir.join(output.unwrap_or_else(|| "tsconfig.json".into()));

      serialize_json(&content, &output, overwrite)?;
    }
    TsCommands::OxlintConfig { output, preset } => {
      let content = typescript
        .oxlint_presets
        .get(&preset)
        .ok_or(GenError::PresetNotFound {
          kind: Preset::Oxlint,
          name: preset.clone(),
        })?
        .clone()
        .process_data(preset.as_str(), &typescript.oxlint_presets)?;

      let output = out_dir.join(output.unwrap_or_else(|| ".oxlintrc.json".into()));

      serialize_json(&content, &output, overwrite)?;
    }
    TsCommands::PackageJson { output, preset } => {
      let content = typescript
        .package_json_presets
        .get(&preset)
        .ok_or(GenError::PresetNotFound {
          kind: Preset::PackageJson,
          name: preset.clone(),
        })?
        .clone()
        .process_data(
          preset.as_str(),
          &typescript.package_json_presets,
          &typescript.people,
        )?;

      let output = out_dir.join(output.unwrap_or_else(|| "package.json".into()));

      serialize_json(&content, &output, overwrite)?;
    }
    TsCommands::Monorepo {
      install,
      root_package_overrides,
      root_package,
      oxlint,
      ..
    } => {
      let mut root_package = if let Some(id) = root_package {
        typescript
          .package_presets
          .get(&id)
          .ok_or(GenError::PresetNotFound {
            kind: Preset::TsPackage,
            name: id,
          })?
          .clone()
      } else {
        let mut package = PackageConfig::default();
        package.oxlint = Some(OxlintConfigSetting::Bool(true));
        package.vitest = VitestConfigKind::Bool(false);
        package.name = Some("root".to_string());
        package
      };

      if let Some(overrides) = root_package_overrides {
        root_package.merge(overrides);
      }

      if oxlint
        && root_package
          .oxlint
          .as_ref()
          .is_none_or(|ox| !ox.is_enabled())
      {
        root_package.oxlint = Some(OxlintConfigSetting::Bool(true));
      }

      let package_manager = typescript.package_manager.get_or_insert_default().clone();

      config.create_ts_monorepo(root_package, &out_dir).await?;

      if install {
        launch_command(
          package_manager.to_string().as_str(),
          &["install"],
          &out_dir,
          Some("Could not install dependencies"),
        )?;
      }
    }
    TsCommands::Package {
      preset,
      package_config,
      install,
      no_vitest,
      oxlint,
      kind,
      update_root_tsconfig,
    } => {
      let mut package = if let Some(preset) = preset {
        typescript
          .package_presets
          .get(&preset)
          .ok_or(GenError::PresetNotFound {
            kind: Preset::TsPackage,
            name: preset.clone(),
          })?
          .clone()
      } else {
        PackageConfig::default()
      };

      if let Some(overrides) = package_config {
        package.merge(overrides);
      }

      if let Some(kind) = kind {
        package.kind = Some(kind.into());
      }

      if no_vitest {
        package.vitest = VitestConfigKind::Bool(false);
      }

      if oxlint && package.oxlint.is_none() {
        package.oxlint = Some(OxlintConfigSetting::Bool(true));
      }

      if config.debug {
        eprintln!("DEBUG:");
        eprintln!("  package: {:#?}", package);
      }

      let package_dir = package.dir.get_or_insert_with(|| get_cwd()).clone();

      if install {
        let package_manager = typescript.package_manager.get_or_insert_default().clone();

        launch_command(
          package_manager.to_string().as_str(),
          &["install"],
          &package_dir,
          Some("Could not install dependencies"),
        )?;
      }

      config
        .build_package(
          PackageData::Config(package.clone()),
          update_root_tsconfig,
          out_dir,
        )
        .await?;
    }
  }

  Ok(())
}

#[derive(Subcommand, Debug, Clone)]
pub enum TsCommands {
  PackageJson {
    /// The output path of the created file [default: `package.json`]
    output: Option<PathBuf>,

    /// The preset id
    #[arg(short, long, value_name = "ID")]
    preset: String,
  },

  TsConfig {
    /// The output path of the created file [default: `tsconfig.json`]
    output: Option<PathBuf>,

    /// The preset id
    #[arg(short, long, value_name = "ID")]
    preset: String,
  },

  OxlintConfig {
    /// The output path of the created file [default: `.oxlintrc.json`]
    output: Option<PathBuf>,

    /// The preset id
    #[arg(short, long, value_name = "ID")]
    preset: String,
  },

  /// Generates a new typescript monorepo inside the `out_dir`
  Monorepo {
    /// The id of the package preset to use for the root package.
    #[arg(short, long, value_name = "ID")]
    root_package: Option<String>,

    #[command(flatten)]
    root_package_overrides: Option<PackageConfig>,

    /// Generate a basic oxlint config at the root.
    #[arg(long)]
    oxlint: bool,

    /// Installs the dependencies at the root after creation.
    #[arg(short, long)]
    install: bool,
  },

  /// Generates a new typescript package
  Package {
    /// The package preset to use
    #[arg(short, long)]
    preset: Option<String>,

    /// Whether the tsconfig file at the workspace root
    /// should receive a reference to the new package
    #[arg(long)]
    update_root_tsconfig: bool,

    /// Does not set up vitest for this package
    #[arg(long)]
    no_vitest: bool,

    /// If an oxlint config is not defined or enabled, this will generate one with the default values.
    #[arg(long)]
    oxlint: bool,

    /// Installs the dependencies with the chosen package manager
    #[arg(short, long)]
    install: bool,

    #[command(flatten)]
    kind: Option<PackageKindFlag>,

    #[command(flatten)]
    package_config: Option<PackageConfig>,
  },
}
