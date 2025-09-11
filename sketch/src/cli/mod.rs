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
  paths::get_cwd,
  tera::{TemplateData, TemplateOutput},
};

pub(crate) mod parsers;

use clap::{error::ErrorKind, CommandFactory, Parser, Subcommand};
use merge::Merge;
use parsers::parse_serializable_key_value_pair;
use serde_json::Value;

use crate::{
  moon::{MoonConfigKind, MoonDotYmlKind},
  package::{vitest::VitestConfigKind, PackageConfig},
  Config, *,
};

async fn get_config_from_cli(cli: Cli) -> Result<Config, GenError> {
  let mut config = if let Some(config_path) = cli.config {
    let conf = match Config::from_file(&config_path) {
      Ok(conf) => conf,
      Err(e) => {
        let mut cmd = Cli::command();
        cmd.error(ErrorKind::InvalidValue, format!("{}", e)).exit();
      }
    };

    conf
  } else {
    Config::default()
  };

  if let Some(overrides) = cli.overrides {
    config.merge(overrides);
  }

  if let Some(vars) = cli.templates_vars {
    config.global_templates_vars.extend(vars);
  }

  match cli.command {
    Init { no_pre_commit, .. } => {
      if no_pre_commit {
        config.pre_commit = PreCommitSetting::Bool(false);
      }
    }

    RenderPreset { .. } => {}

    Render { .. } => {}
    New { .. } => {}
    Command { .. } => {}
    Ts {
      command,
      typescript_overrides,
      shared_out_dir,
      no_shared_out_dir,
    } => {
      let typescript = config.typescript.get_or_insert_default();

      if let Some(typescript_overrides) = typescript_overrides {
        typescript.merge(typescript_overrides);
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
        }
        TsCommands::Package {
          package_config,
          kind,
          preset,
          moonrepo,
          no_vitest,
          oxlint,
          no_update_root_tsconfig,
          ..
        } => {
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
        }
      }
    }
  };

  Ok(config)
}

pub async fn main_entrypoint() -> Result<(), GenError> {
  execute_cli(Cli::parse()).await
}

async fn execute_cli(cli: Cli) -> Result<(), GenError> {
  let mut config = get_config_from_cli(cli.clone()).await?;

  macro_rules! exit_if_dry_run {
    () => {
      if cli.dry_run {
        println!("Aborting due to dry run...");
        return Ok(());
      }
    };
  }

  if config.debug {
    println!("DEBUG:");
    println!("  config: {:#?}", config);
  }

  match cli.command {
    Init {
      remote,
      no_pre_commit,
    } => {
      if config.debug {
        println!("DEBUG:");
        println!("  remote: {:?}", remote);
        println!("  no_pre_commit: {}", no_pre_commit);
      }

      exit_if_dry_run!();

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

      if config.debug {
        println!("DEBUG:");
        println!("  preset: {:#?}", preset);
      }

      exit_if_dry_run!();

      config.generate_templates(get_cwd(), preset)?;
    }

    Render {
      content,
      output,
      id,
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
        println!("DEBUG:");
        println!("  template: {:#?}", template);
      }

      exit_if_dry_run!();

      config.generate_templates(get_cwd(), vec![template])?;
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
          .unwrap_or_else(|| panic!("Output file {} has no extension.", output_path.display()))
          .to_string_lossy(),
      )?;

      if config.debug {
        println!("DEBUG:");
        println!("  output path: {}", output_path.display());
      }

      let mut output_file = if config.no_overwrite {
        File::create_new(&output_path).map_err(|e| match e.kind() {
          io::ErrorKind::AlreadyExists => GenError::FileExists {
            path: output_path.clone(),
          },
          _ => GenError::WriteError {
            path: output_path.clone(),
            source: e,
          },
        })?
      } else {
        File::create(&output_path).map_err(|e| GenError::FileCreation {
          path: output_path.clone(),
          source: e,
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
        if config.debug {
          println!("DEBUG:");
          println!("  command: {}", literal);
        }
        literal
      } else if let Some(file_path) = file {
        if config.debug {
          println!("DEBUG:");
          println!("  file: {}", file_path.display());
        }
        read_to_string(&file_path).map_err(|e| GenError::ReadError {
          path: file_path,
          source: e,
        })?
      } else {
        panic!("At least one between command and file must be set.")
      };

      let shell = config.shell.clone();
      config.execute_command(shell.as_deref(), cwd, &command)?;
    }
    Ts { command, .. } => {
      let typescript = config.typescript.get_or_insert_default();

      match command {
        TsCommands::Monorepo { .. } => {
          config.create_ts_monorepo().await?;
        }
        TsCommands::Package {
          preset,
          package_config,
          install,
          name,
          ..
        } => {
          let package_manager = typescript.package_manager.unwrap_or_default();

          let package = if let Some(preset) = preset {
            typescript
              .package_presets
              .shift_remove(&preset)
              .ok_or(GenError::PresetNotFound {
                kind: Preset::Package,
                name: preset.clone(),
              })?
          } else {
            package_config.unwrap_or_default()
          };

          if config.debug {
            println!("DEBUG:");
            println!("  package {:#?}", package);
          }

          exit_if_dry_run!();

          let package_dir = package.dir.clone().unwrap_or_else(|| get_cwd());

          if install {
            launch_command(
              None,
              &[package_manager.to_string().as_str(), "install"],
              &package_dir,
              Some("Could not install dependencies"),
            )?;
          }

          config
            .build_package(package::PackageData {
              name,
              kind: PackageDataKind::Config(package.clone()),
            })
            .await?;
        }
      }
    }
  }
  Ok(())
}

/// The struct defining the cli for this crate.
#[derive(Parser, Debug, Clone)]
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

  /// Set a variable (as key=value) to use in templates. Overrides global and local variables.
  #[arg(long = "set", short = 's', value_parser = parse_serializable_key_value_pair, value_name = "KEY=VALUE")]
  pub templates_vars: Option<Vec<(String, Value)>>,
}

/// The cli commands.
#[derive(Subcommand, Debug, Clone)]
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

#[derive(Subcommand, Debug, Clone)]
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

#[cfg(test)]
mod test {
  use clap::Parser;

  use crate::cli::{execute_cli, Cli};

  #[tokio::test]
  async fn cli_root_dir() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::try_parse_from([
      "sketch",
      "--debug",
      "-c",
      "tests/paths_resolution/root_dir_resolution.toml",
      "ts",
      "--no-catalog",
      "package",
      "package1",
    ])?;

    println!("{:#?}", cli);

    execute_cli(cli).await?;

    Ok(())
  }
  #[test]
  fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert();
  }
}
