#[cfg(test)]
mod cli_tests;

mod cli_elements;
pub(crate) mod parsers;

use std::{
  env,
  fs::{exists, read_dir, read_to_string},
  io::{self, Write},
  path::PathBuf,
};

use clap::{Args, Parser, Subcommand};
use cli_elements::*;
use merge::Merge;
use parsers::parse_serializable_key_value_pair;
use serde_json::Value;
use Commands::*;

use crate::{
  commands::launch_command,
  custom_templating::{TemplateData, TemplateOutput},
  paths::get_cwd,
  ts::{
    package::{PackageConfig, PackageData, RootPackage},
    vitest::VitestConfigKind,
    OxlintConfig, TypescriptConfig,
  },
  Config, *,
};

fn get_config_file_path(cli_arg: Option<PathBuf>) -> Option<PathBuf> {
  if let Some(cli_arg) = cli_arg {
    Some(cli_arg)
  } else if exists("sketch.yaml").is_ok_and(|exists| exists) {
    Some(PathBuf::from("sketch.yaml"))
  } else if exists("sketch.toml").is_ok_and(|exists| exists) {
    Some(PathBuf::from("sketch.toml"))
  } else if exists("sketch.json").is_ok_and(|exists| exists) {
    Some(PathBuf::from("sketch.json"))
  } else {
    None
  }
}

fn get_config_from_xdg() -> Option<PathBuf> {
  if let Ok(xdg_config) = env::var("XDG_CONFIG_HOME") {
    let config_dir = PathBuf::from(xdg_config).join("sketch");

    if config_dir.is_dir() {
      if let Ok(dir_contents) = read_dir(&config_dir) {
        for item in dir_contents {
          if let Ok(item) = item {
            if item.file_name() == "sketch.toml"
              || item.file_name() == "sketch.yaml"
              || item.file_name() == "sketch.json"
            {
              return Some(item.path());
            }
          }
        }
      }
    }
  }
  None
}

async fn get_config_from_cli(cli: Cli) -> Result<Config, GenError> {
  let mut config = Config::default();

  if !cli.ignore_config_file {
    let config_file = get_config_file_path(cli.config);

    if let Some(config_path) = config_file {
      config.merge(Config::from_file(&config_path)?);
    } else if let Some(config_from_xdg) = get_config_from_xdg() {
      config.merge(Config::from_file(&config_from_xdg)?);
    }
  }

  if let Some(overrides) = cli.overrides {
    config.merge(overrides);
  }

  if let Some(vars) = cli.templates_vars {
    config.vars.extend(vars);
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
    Exec { .. } => {}
    Ts {
      command,
      typescript_overrides,
    } => {
      let typescript = config.typescript.get_or_insert_default();

      if let Some(typescript_overrides) = typescript_overrides {
        typescript.merge(typescript_overrides);
      }

      match command {
        TsCommands::Monorepo {
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
        }
        TsCommands::Package { .. } => {}
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
  let root_dir = config.out_dir.clone().unwrap_or_else(|| get_cwd());

  macro_rules! exit_if_dry_run {
    () => {
      if cli.dry_run {
        eprintln!("Aborting due to dry run...");
        return Ok(());
      }
    };
  }

  if config.debug {
    eprintln!("DEBUG:");
    eprintln!("  config: {:#?}", config);
  }

  match cli.command {
    Init {
      remote,
      no_pre_commit,
    } => {
      if config.debug {
        eprintln!("DEBUG:");
        eprintln!("  remote: {:?}", remote);
        eprintln!("  no_pre_commit: {}", no_pre_commit);
      }

      exit_if_dry_run!();

      config.init_repo(remote.as_deref())?;
    }

    RenderPreset { id } => {
      let preset = config
        .templating_presets
        .get(&id)
        .ok_or(GenError::PresetNotFound {
          kind: Preset::Templates,
          name: id.clone(),
        })?
        .clone();

      if config.debug {
        eprintln!("DEBUG:");
        eprintln!("  preset: {:#?}", preset);
      }

      exit_if_dry_run!();

      config.generate_templates(root_dir, preset)?;
    }

    Render {
      content,
      output,
      id,
      file,
    } => {
      let template_data = if let Some(id) = id {
        TemplateData::Id(id)
      } else if let Some(content) = content {
        TemplateData::Content {
          name: "template_from_cli".to_string(),
          content,
        }
      } else if let Some(file) = file {
        let file_content = read_to_string(&file).map_err(|e| GenError::ReadError {
          path: file.clone(),
          source: e,
        })?;

        TemplateData::Content {
          name: "template_from_cli".to_string(),
          content: file_content,
        }
      } else {
        panic!("Missing id or content for template generation");
      };

      let output = if output.stdout {
        "__stdout".to_string()
      } else {
        output
          .output_path
          .expect("At least one must be set between output_path and --stdout")
      };

      let template = TemplateOutput {
        output,
        context: Default::default(),
        template: template_data,
      };

      if config.debug {
        eprintln!("DEBUG:");
        eprintln!("  template: {:#?}", template);
      }

      exit_if_dry_run!();

      config.generate_templates(root_dir, vec![template])?;
    }
    New { output } => {
      let output_path = output.unwrap_or_else(|| PathBuf::from("sketch.yaml"));

      if let Some(parent_dir) = output_path.parent() {
        create_dir_all(parent_dir).map_err(|e| GenError::DirCreation {
          path: parent_dir.to_path_buf(),
          source: e,
        })?;
      }

      let format = &output_path
        .extension()
        .unwrap_or_else(|| panic!("Output file {} has no extension.", output_path.display()))
        .to_string_lossy();

      if config.debug {
        eprintln!("DEBUG:");
        eprintln!("  output path: {}", output_path.display());
      }

      exit_if_dry_run!();

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

      match format.as_ref() {
        "yaml" => serde_yaml_ng::to_writer(output_file, &config).map_err(|e| {
          GenError::SerializationError {
            target: "the new config file".to_string(),
            error: e.to_string(),
          }
        })?,
        "toml" => {
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
        "json" => serde_json::to_writer_pretty(output_file, &config).map_err(|e| {
          GenError::SerializationError {
            target: "the new config file".to_string(),
            error: e.to_string(),
          }
        })?,
        _ => return Err(GenError::InvalidConfigFormat { file: output_path }),
      };
    }
    Exec {
      cmd: command,
      file,
      template,
    } => {
      let command = if let Some(literal) = command {
        TemplateData::Content {
          name: "__command".to_string(),
          content: literal,
        }
      } else if let Some(id) = template {
        TemplateData::Id(id)
      } else if let Some(file_path) = file {
        let content = read_to_string(&file_path).map_err(|e| GenError::ReadError {
          path: file_path.clone(),
          source: e,
        })?;

        TemplateData::Content {
          name: format!("__{}", file_path.display()),
          content,
        }
      } else {
        panic!("At least one between command and file must be set.")
      };

      exit_if_dry_run!();

      let shell = config.shell.clone();
      config.execute_command(shell.as_deref(), &root_dir, command)?;
    }
    Ts { command, .. } => {
      let typescript = config.typescript.get_or_insert_default();

      match command {
        TsCommands::Monorepo { install, .. } => {
          exit_if_dry_run!();

          let package_manager = typescript.package_manager.unwrap_or_default();

          config.create_ts_monorepo().await?;

          if install {
            launch_command(
              None,
              &[package_manager.to_string().as_str(), "install"],
              &root_dir,
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

          if oxlint {
            package.oxlint = Some(OxlintConfig::Bool(true));
          }

          exit_if_dry_run!();

          let package_dir = package.dir.clone().unwrap_or_else(|| get_cwd());

          if install {
            let package_manager = typescript.package_manager.unwrap_or_default();

            launch_command(
              None,
              &[package_manager.to_string().as_str(), "install"],
              &package_dir,
              Some("Could not install dependencies"),
            )?;
          }

          config
            .build_package(PackageData::Config(package.clone()), update_root_tsconfig)
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
pub struct Cli {
  /// Sets a custom config file.
  #[arg(short, long, value_name = "FILE", group = "config-file")]
  pub config: Option<PathBuf>,

  /// Ignores any config files, uses cli instructions only
  #[arg(long, group = "config-file")]
  pub ignore_config_file: bool,

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

#[derive(Args, Debug, Clone)]
#[group(required = true, multiple = false)]
pub struct RenderingOutput {
  /// The output file (relative from the cwd)
  #[arg(requires = "input")]
  output_path: Option<String>,

  /// Output the result to stdout
  #[arg(long, requires = "input")]
  stdout: bool,
}

/// The cli commands.
#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
  /// Launches typescript-specific commands.
  Ts {
    #[command(flatten)]
    typescript_overrides: Option<TypescriptConfig>,

    #[command(subcommand)]
    command: TsCommands,
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

  /// Renders a single template to a file or to stdout
  Render {
    #[command(flatten)]
    output: RenderingOutput,

    /// The path to the template file, from the cwd
    #[arg(short, long, group = "input")]
    file: Option<PathBuf>,

    /// The id of the template to use
    #[arg(short, long, group = "input")]
    id: Option<String>,

    /// The literal definition for the template
    #[arg(short, long, group = "input")]
    content: Option<String>,
  },

  /// Renders a templating preset defined in the configuration file
  RenderPreset {
    /// The id of the preset.
    id: String,
  },

  /// Renders a template and launches it as a command
  Exec {
    #[arg(group = "input")]
    /// The literal definition for the command's template
    cmd: Option<String>,

    /// The path to the command's template file
    #[arg(short, long, group = "input")]
    file: Option<PathBuf>,

    /// The id of the template to use
    #[arg(short, long, group = "input")]
    template: Option<String>,
  },
}

#[derive(Subcommand, Debug, Clone)]
pub enum TsCommands {
  /// Generates a new typescript monorepo
  Monorepo {
    #[command(flatten)]
    root_package_overrides: Option<RootPackage>,

    /// Does not generate an oxlint config at the root.
    #[arg(long)]
    no_oxlint: bool,

    /// Install the dependencies at the root after creation.
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

    /// Sets up an oxlint config file for this package
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
