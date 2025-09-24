use std::path::PathBuf;

use clap::Subcommand;
use merge::Merge;

use crate::{
  exec::launch_command,
  fs::{create_parent_dirs, get_cwd, serialize_json},
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

      let output = output.unwrap_or_else(|| "tsconfig.json".into());
      create_parent_dirs(&output)?;

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

      let output = output.unwrap_or_else(|| ".oxlintrc.json".into());
      create_parent_dirs(&output)?;

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

      let output = output.unwrap_or_else(|| "package.json".into());
      create_parent_dirs(&output)?;

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

      if let Some(id) = oxlint {
        root_package.oxlint = if id == "default" {
          Some(OxlintConfigSetting::Bool(true))
        } else {
          Some(OxlintConfigSetting::Id(id))
        };
      }

      let package_manager = typescript.package_manager.get_or_insert_default().clone();

      config.create_ts_monorepo(root_package, &get_cwd()).await?;

      if install {
        launch_command(
          package_manager.to_string().as_str(),
          &["install"],
          &get_cwd(),
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
      update_tsconfig,
      dir,
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

      if no_vitest {
        package.vitest = VitestConfigKind::Bool(false);
      }

      if let Some(id) = oxlint {
        package.oxlint = if id == "default" {
          Some(OxlintConfigSetting::Bool(true))
        } else {
          Some(OxlintConfigSetting::Id(id))
        };
      }

      if config.debug {
        eprintln!("DEBUG:");
        eprintln!("  package: {:#?}", package);
      }

      let package_dir =
        dir.unwrap_or_else(|| package.name.as_deref().unwrap_or("new_package").into());

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
          package_dir,
          update_tsconfig,
        )
        .await?;
    }
  }

  Ok(())
}

#[derive(Subcommand, Debug, Clone)]
pub enum TsCommands {
  /// Generates a `package.json` file from a preset.
  PackageJson {
    /// The output path of the generated file [default: `package.json`]
    output: Option<PathBuf>,

    /// The preset id
    #[arg(short, long, value_name = "ID")]
    preset: String,
  },

  /// Generates a `tsconfig.json` file from a preset.
  TsConfig {
    /// The output path of the generated file [default: `tsconfig.json`]
    output: Option<PathBuf>,

    /// The preset id
    #[arg(short, long, value_name = "ID")]
    preset: String,
  },

  /// Generates a `.oxlintrc.json` file from a preset.
  OxlintConfig {
    /// The output path of the generated file [default: `.oxlintrc.json`]
    output: Option<PathBuf>,

    /// The preset id
    #[arg(short, long, value_name = "ID")]
    preset: String,
  },

  /// Generates a new typescript monorepo
  Monorepo {
    /// The id of the package preset to use for the root package. If unset, the default preset is used, along with the values set via cli flags.
    #[arg(short, long, value_name = "ID")]
    root_package: Option<String>,

    #[command(flatten)]
    root_package_overrides: Option<PackageConfig>,

    /// The oxlint preset to use. It can be set to `default` to use the default preset.
    #[arg(long, value_name = "ID")]
    oxlint: Option<String>,

    /// Installs the dependencies at the root after creation.
    #[arg(short, long)]
    install: bool,
  },

  /// Generates a new typescript package
  Package {
    /// The root directory for the new package. Defaults to the package name, if that is set.
    dir: Option<PathBuf>,

    /// The package preset to use
    #[arg(short, long, value_name = "ID")]
    preset: Option<String>,

    /// An optional list of tsconfig paths where the new tsconfig file will be added as a reference.
    #[arg(short, long)]
    update_tsconfig: Option<Vec<PathBuf>>,

    /// Does not set up vitest for this package
    #[arg(long)]
    no_vitest: bool,

    /// The oxlint preset to use. It can be set to `default` to use the default preset.
    #[arg(long, value_name = "ID")]
    oxlint: Option<String>,

    /// Installs the dependencies with the chosen package manager
    #[arg(short, long)]
    install: bool,

    #[command(flatten)]
    package_config: Option<PackageConfig>,
  },
}
