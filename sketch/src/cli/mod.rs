#![allow(clippy::large_enum_variant)]
use std::{
  fs::read_to_string,
  io::{self, Write},
  path::PathBuf,
  str::FromStr,
};

mod cli_elements;

use cli_elements::*;
use Commands::*;

use crate::{
  commands::launch_command,
  package::PackageDataKind,
  tera::{TemplateData, TemplateOutput},
};

pub(crate) mod parsers;

use std::env::current_dir;

use clap::{error::ErrorKind, CommandFactory, Parser, Subcommand};
use merge::Merge;
use parsers::parse_serializable_key_value_pair;
use serde_json::Value;

use crate::{
  moon::{MoonConfigKind, MoonDotYmlKind},
  package::{vitest::VitestConfigKind, PackageConfig},
  Config, *,
};

pub async fn start_cli() -> Result<(), GenError> {
  let cli = Cli::parse();

  let mut config = if let Some(config_path) = cli.config.as_deref() {
    match Config::from_file(config_path) {
      Ok(conf) => conf,
      Err(e) => {
        let mut cmd = Cli::command();
        cmd.error(ErrorKind::InvalidValue, format!("{}", e)).exit();
      }
    }
  } else {
    Config::default()
  };

  if let Some(overrides) = cli.overrides {
    config.merge(overrides);
  }

  if let Some(vars) = cli.templates_vars {
    config.global_templates_vars.extend(vars);
  }

  macro_rules! exit_if_dry_run {
    () => {
      if cli.dry_run {
        println!("Aborting due to dry run...");
        return Ok(());
      }
    };
  }

  if cli.no_overwrite {
    config.overwrite = false;
  }

  match cli.command {
    Init {
      no_pre_commit,
      remote,
    } => {
      if no_pre_commit {
        config.pre_commit = PreCommitSetting::Bool(false);
      }

      config.init_repo(remote.as_deref())?;
    }

    RenderPreset { id } => {
      let preset = config
        .templating_presets
        .get(&id)
        .ok_or(GenError::PresetNotFound {
          kind: Preset::Templating,
          name: id.clone(),
        })?
        .clone();

      let base_path = config.root_dir.clone();
      config.generate_templates(base_path.as_deref().unwrap_or("."), preset)?;
    }

    Render {
      content,
      output,
      id,
      ..
    } => {
      let template_data = if let Some(id) = id {
        TemplateData::Id(id)
      } else if let Some(content) = content {
        TemplateData::Content {
          name: "template_from_cli".to_string(),
          content,
        }
      } else {
        panic!("Missing id or content for template generation");
      };

      let template = TemplateOutput {
        output,
        context: Default::default(),
        template: template_data,
      };

      if config.debug {
        println!("DEBUG: {:#?}", template);
      }

      exit_if_dry_run!();

      config.generate_templates(
        &current_dir()
          .expect("Could not get the cwd")
          .to_string_lossy(),
        vec![template],
      )?;
    }
    New { output } => {
      let output_path = output.unwrap_or_else(|| PathBuf::from("sketch.yaml"));

      if let Some(parent_dir) = output_path.parent() {
        create_dir_all(parent_dir).map_err(|e| GenError::DirCreation {
          path: parent_dir.to_path_buf(),
          source: e,
        })?;
      }

      let format = <ConfigFormat as FromStr>::from_str(
        &output_path
          .extension()
          .unwrap_or_else(|| panic!("File {} has no extension.", output_path.display()))
          .to_string_lossy(),
      )?;

      let mut output_file = if config.overwrite {
        File::create(&output_path).map_err(|e| GenError::FileCreation {
          path: output_path.clone(),
          source: e,
        })?
      } else {
        File::create_new(&output_path).map_err(|e| match e.kind() {
          io::ErrorKind::AlreadyExists => GenError::FileExists {
            path: output_path.clone(),
          },
          _ => GenError::WriteError {
            path: output_path.clone(),
            source: e,
          },
        })?
      };

      match format {
        ConfigFormat::Yaml => serde_yaml_ng::to_writer(output_file, &config).map_err(|e| {
          GenError::SerializationError {
            target: "the new config file".to_string(),
            error: e.to_string(),
          }
        })?,
        ConfigFormat::Toml => {
          let content =
            toml::to_string_pretty(&config).map_err(|e| GenError::SerializationError {
              target: "the new config file".to_string(),
              error: e.to_string(),
            })?;

          output_file
            .write_all(&content.into_bytes())
            .map_err(|e| GenError::WriteError {
              path: output_path.clone(),
              source: e,
            })?;
        }
        ConfigFormat::Json => serde_json::to_writer_pretty(output_file, &config).map_err(|e| {
          GenError::SerializationError {
            target: "the new config file".to_string(),
            error: e.to_string(),
          }
        })?,
      };
    }
    Command { command, file, cwd } => {
      let command = if let Some(literal) = command {
        literal
      } else if let Some(file_path) = file {
        read_to_string(&file_path).map_err(|e| GenError::ReadError {
          path: file_path,
          source: e,
        })?
      } else {
        panic!("At least one between command and file must be set.")
      };

      exit_if_dry_run!();

      let shell = config.shell.clone();
      config.execute_command(shell.as_deref(), cwd, &command)?;
    }
    Ts {
      command,
      typescript_overrides,
      shared_out_dir,
      no_shared_out_dir,
      no_convert_latest,
      no_catalog,
    } => {
      let typescript = config.typescript.get_or_insert_default();

      if let Some(typescript_overrides) = typescript_overrides {
        typescript.merge(typescript_overrides);
      }

      if no_convert_latest {
        typescript.convert_latest_to_range = false;
      }

      if no_catalog {
        typescript.catalog = false;
      }

      if no_shared_out_dir {
        typescript.shared_out_dir = SharedOutDir::Bool(false);
      }

      if let Some(shared_out_dir) = shared_out_dir {
        typescript.shared_out_dir = SharedOutDir::Name(shared_out_dir);
      }

      match command {
        TsCommands::Monorepo {
          moonrepo,
          no_oxlint,
          root_package_overrides,
          ..
        } => {
          let root_package = typescript.root_package.get_or_insert_default();

          if let Some(root_package_overrides) = root_package_overrides {
            root_package.merge(root_package_overrides);
          }

          if no_oxlint {
            root_package.oxlint = Some(OxlintConfig::Bool(false));
          }

          if moonrepo {
            root_package.moonrepo = Some(MoonConfigKind::Bool(true));
          }

          exit_if_dry_run!();
        }
        TsCommands::Package {
          package_config,
          kind,
          preset,
          moonrepo,
          no_vitest,
          oxlint,
          no_update_root_tsconfig,
          install,
          name,
        } => {
          let package_manager = typescript.package_manager.unwrap_or_default();

          let package = if let Some(preset) = preset {
            typescript
              .package_presets
              .get_mut(&preset)
              .ok_or(GenError::PresetNotFound {
                kind: Preset::Package,
                name: preset.clone(),
              })?
          } else {
            &mut package_config.unwrap_or_default()
          };

          let package_dir = package.dir.clone();

          if let Some(kind) = kind {
            package.kind = Some(kind.into());
          }

          if moonrepo {
            package.moonrepo = Some(MoonDotYmlKind::Bool(true));
          }

          if no_vitest {
            package.vitest = VitestConfigKind::Boolean(false);
          }

          if oxlint {
            package.oxlint = Some(OxlintConfig::Bool(true));
          }

          if no_update_root_tsconfig {
            package.update_root_tsconfig = false;
          }

          let package = package.clone();

          if config.debug {
            println!("DEBUG: Config {:#?}", config);
            println!("DEBUG: Package {:#?}", package);
          }

          exit_if_dry_run!();

          if install {
            launch_command(
              None,
              &[package_manager.to_string().as_str(), "install"],
              package_dir.as_deref().unwrap_or("."),
              Some("Could not install dependencies"),
            )?;
          }

          config
            .build_package(package::PackageData {
              name,
              kind: PackageDataKind::Config(package),
            })
            .await?;
        }
      }
    }
  };

  Ok(())
}

#[derive(Parser)]
#[command(name = "sketch")]
#[command(version, about, long_about = None)]
struct Cli {
  /// Sets a custom config file.
  #[arg(short, long, value_name = "FILE")]
  pub config: Option<PathBuf>,

  #[command(subcommand)]
  pub command: Commands,

  #[command(flatten)]
  pub overrides: Option<Config>,

  /// Aborts before writing any content to disk.
  #[arg(long)]
  pub dry_run: bool,

  /// Exits with error if a file being created already exists.
  #[arg(long)]
  pub(crate) no_overwrite: bool,

  /// Set a variable (as key=value) to use in templates. Overrides global and local variables.
  #[arg(long = "set", short = 's', value_parser = parse_serializable_key_value_pair, value_name = "KEY=VALUE")]
  pub templates_vars: Option<Vec<(String, Value)>>,
}

#[derive(Subcommand)]
enum Commands {
  /// Launches typescript-specific commands.
  Ts {
    #[command(flatten)]
    typescript_overrides: Option<TypescriptConfig>,

    #[command(subcommand)]
    command: TsCommands,

    /// The path to the shared out_dir for TS packages.
    #[arg(long, conflicts_with = "no_shared_out_dir")]
    shared_out_dir: Option<String>,

    /// Does not use a shared out_dir for TS packages.
    #[arg(long, default_value_t = false)]
    no_shared_out_dir: bool,

    /// Does not convert 'latest' to a version range.
    #[arg(long)]
    no_convert_latest: bool,

    /// Does not use the catalog for default dependencies.
    #[arg(long)]
    no_catalog: bool,
  },

  /// Creates a new git repo with a gitignore file. Optionally, it sets up the git remote and the pre-commit config.
  Init {
    /// Does not generate a pre-commit config.
    #[arg(long)]
    no_pre_commit: bool,

    /// The link to the git remote to use.
    #[arg(long)]
    remote: Option<String>,
  },

  /// Generates a new config file with some optional initial values defined via the cli flags.
  New {
    /// The output file [default: sketch.yaml]
    output: Option<PathBuf>,
  },

  /// Generates a single file from a template.
  Render {
    /// The output file (relative from the cwd)
    #[arg(requires = "input")]
    output: String,
    /// The id of the preset to select (cannot be used with the --content flag)
    #[arg(short, long, group = "input")]
    id: Option<String>,
    /// The literal definition for the template (cannot be used with the --id flag)
    #[arg(short, long, group = "input")]
    content: Option<String>,
  },

  /// Generates content from a templating preset, with predefined content, output and context.
  RenderPreset {
    /// The id of the preset.
    id: String,
  },

  /// Renders a template (from text or file) and launches it as a command
  Command {
    /// The literal definition for the command's template (cannot be used with the --file flag)
    command: Option<String>,

    /// The path to the command's template file
    #[arg(short, long, conflicts_with = "command")]
    file: Option<PathBuf>,

    /// The cwd for the command to execute
    #[arg(long)]
    cwd: Option<PathBuf>,
  },
}

#[derive(Subcommand)]
enum TsCommands {
  /// Generates a new typescript monorepo
  Monorepo {
    #[command(flatten)]
    root_package_overrides: Option<RootPackage>,

    /// Does not generate an oxlint config at the root.
    #[arg(long)]
    no_oxlint: bool,

    /// Generate setup for moonrepo
    #[arg(long)]
    moonrepo: bool,
  },

  /// Generates a new typescript package
  Package {
    /// The name of the new package
    name: Option<String>,
    /// The package preset to use
    #[arg(long)]
    preset: Option<String>,

    /// Sets up a basic moon.yml file
    #[arg(long)]
    moonrepo: bool,

    /// Does not set up vitest for this package
    #[arg(long)]
    no_vitest: bool,

    /// Sets up an oxlint config file for this package
    #[arg(long)]
    oxlint: bool,

    /// Does not update the root tsconfig with a reference to the new tsconfig file
    #[arg(long)]
    no_update_root_tsconfig: bool,

    /// Installs the dependencies with the chosen package manager
    #[arg(short, long)]
    install: bool,

    #[command(flatten)]
    kind: Option<PackageKindFlag>,

    #[command(flatten)]
    package_config: Option<PackageConfig>,
  },
}

#[test]
fn verify_cli() {
  use clap::CommandFactory;
  Cli::command().debug_assert();
}
