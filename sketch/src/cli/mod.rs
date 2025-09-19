#[cfg(test)]
mod cli_tests;

mod cli_elements;
pub(crate) mod parsers;

use std::{
  env,
  fs::{exists, read_dir, read_to_string},
  path::PathBuf,
};

use clap::{Args, Parser, Subcommand};
use cli_elements::*;
use merge::Merge;
use parsers::parse_serializable_key_value_pair;
use serde_json::Value;
use Commands::*;

use crate::{
  custom_templating::{TemplateData, TemplateOutput},
  exec::launch_command,
  fs::{create_all_dirs, get_cwd, get_extension, serialize_json, serialize_toml, serialize_yaml},
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
  let xdg_config = if let Ok(env_val) = env::var("XDG_CONFIG_HOME") {
    Some(PathBuf::from(env_val))
  } else if let Some(home) = env::home_dir() {
    Some(home.join(".config"))
  } else {
    None
  };

  if let Some(xdg_config) = xdg_config {
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
    let config_path = if let Some(config_file) = get_config_file_path(cli.config) {
      Some(config_file)
    } else if let Some(config_from_xdg) = get_config_from_xdg() {
      Some(config_from_xdg)
    } else {
      None
    };

    if let Some(config_path) = config_path {
      eprintln!("Found config file `{}`", config_path.display());
      config.merge(Config::from_file(&config_path)?);
    }
  } else if config.debug {
    eprintln!("`ignore_config_file` detected");
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
  let is_dry_run = cli.dry_run;
  let command = cli.command.clone();

  let mut config = get_config_from_cli(cli).await?;
  let root_dir = config.out_dir.clone().unwrap_or_else(|| get_cwd());

  macro_rules! exit_if_dry_run {
    () => {
      if is_dry_run {
        eprintln!("Aborting due to dry run...");
        return Ok(());
      }
    };
  }

  if config.debug {
    eprintln!("DEBUG:");
    eprintln!("  config: {:#?}", config);
  }

  match command {
    Init { remote, .. } => {
      if config.debug {
        eprintln!("DEBUG:");
        eprintln!("  remote: {:?}", remote);
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
          name: "__from_cli".to_string(),
          content,
        }
      } else if let Some(file) = file {
        let file_content = read_to_string(&file).map_err(|e| GenError::ReadError {
          path: file.clone(),
          source: e,
        })?;

        TemplateData::Content {
          name: format!("__custom_file_{}", file.display()),
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
        template: template_data,
        context: Default::default(),
      };

      exit_if_dry_run!();

      config.generate_templates(root_dir, vec![template])?;
    }
    New { output } => {
      let output_path = get_cwd().join(output.unwrap_or_else(|| PathBuf::from("sketch.yaml")));

      if let Some(parent_dir) = output_path.parent() {
        create_all_dirs(&parent_dir)?;
      }

      let format = get_extension(&output_path).to_string_lossy();

      exit_if_dry_run!();

      if output_path.exists() && config.no_overwrite {
        return Err(GenError::Custom(format!(
          "File `{}` already exists and overwriting is disabled",
          output_path.display()
        )));
      }

      match format.as_ref() {
        "yaml" => serialize_yaml(&config, &output_path)?,
        "toml" => {
          serialize_toml(&config, &output_path)?;
        }
        "json" => serialize_json(&config, &output_path)?,
        _ => {
          return Err(GenError::Custom(format!(
            "Invalid config format. Allowed formats are: yaml, toml, json"
          )))
        }
      };
    }
    Exec {
      cmd: command,
      file,
      template,
      cwd,
    } => {
      let command = if let Some(literal) = command {
        TemplateData::Content {
          name: "__from_cli".to_string(),
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
          name: format!("__custom_file_{}", file_path.display()),
          content,
        }
      } else {
        panic!("At least one between command and file must be set.")
      };

      exit_if_dry_run!();

      let cwd = cwd.unwrap_or_else(|| get_cwd());

      let shell = config.shell.clone();
      config.execute_command(shell.as_deref(), &cwd, command)?;
    }
    Ts { command, .. } => {
      let typescript = config.typescript.get_or_insert_default();

      match command {
        TsCommands::Monorepo { install, .. } => {
          exit_if_dry_run!();

          let package_manager = typescript.package_manager.get_or_insert_default().clone();

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

          if config.debug {
            eprintln!("DEBUG:");
            eprintln!("  package: {:#?}", package);
          }

          exit_if_dry_run!();

          let package_dir = package.dir.get_or_insert_with(|| get_cwd()).clone();

          if install {
            let package_manager = typescript.package_manager.get_or_insert_default().clone();

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

#[derive(Parser, Debug, Clone)]
#[command(name = "sketch")]
#[command(version, about, long_about = None)]
pub struct Cli {
  /// Sets a custom config file. Any file names `sketch.{yaml,json,toml}` in the cwd or in `XDG_CONFIG_HOME/sketch` will be detected automatically. If no file is found, the default settings are used
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

  /// Prints the result to stdout
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

  /// Creates a new git repo with a generated gitignore file and, optionally, it sets up the git remote and the pre-commit config.
  Init {
    /// Does not generate a pre-commit config.
    #[arg(long)]
    no_pre_commit: bool,

    /// The link to the git remote to use.
    #[arg(long)]
    remote: Option<String>,
  },

  /// Generates a new config file with some optional initial values defined via cli flags.
  New {
    /// The output file. Must be an absolute path or a path relative from the cwd [default: sketch.yaml]
    output: Option<PathBuf>,
  },

  /// Renders a single template to a file or to stdout
  Render {
    #[command(flatten)]
    output: RenderingOutput,

    /// The path to the template file, as an absolute path or relative to the cwd
    #[arg(short, long, group = "input")]
    file: Option<PathBuf>,

    /// The id of the template to use (a name for config-defined templates, or a relative path for a file inside `templates_dir`)
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

  /// Renders a template and executes it as a shell command
  Exec {
    /// The literal definition for the template
    #[arg(group = "input")]
    cmd: Option<String>,

    /// The cwd for the command to execute. [default: `.`]
    #[arg(long)]
    cwd: Option<PathBuf>,

    /// The path to the command's template file, as an absolute path or relative to the cwd
    #[arg(short, long, group = "input")]
    file: Option<PathBuf>,

    /// The id of the template to use (a name for config-defined templates, or a relative path to a file inside `templates_dir`)
    #[arg(short, long, group = "input")]
    template: Option<String>,
  },
}

#[derive(Subcommand, Debug, Clone)]
pub enum TsCommands {
  /// Generates a new typescript monorepo inside the `out_dir`
  Monorepo {
    #[command(flatten)]
    root_package_overrides: Option<RootPackage>,

    /// Does not generate an oxlint config at the root.
    #[arg(long)]
    no_oxlint: bool,

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
