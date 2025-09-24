#[cfg(test)]
mod cli_tests;

mod ts_cmds;

mod cli_elements;
pub(crate) mod parsers;

use std::{
  env,
  fs::{exists, read_dir, read_to_string},
  path::PathBuf,
};

use clap::{Args, Parser, Subcommand};
use merge::Merge;
use parsers::parse_serializable_key_value_pair;
use serde_json::Value;
use Commands::*;

use crate::{
  cli::ts_cmds::{handle_ts_commands, TsCommands},
  custom_templating::{TemplateData, TemplateOutput},
  fs::{create_all_dirs, get_cwd, get_extension, serialize_json, serialize_toml, serialize_yaml},
  init_repo::{gitignore::GitIgnoreSetting, pre_commit::PreCommitSetting, RepoPreset},
  ts::TypescriptConfig,
  Config, *,
};

pub async fn main_entrypoint() -> Result<(), GenError> {
  execute_cli(Cli::parse()).await
}

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

  if !cli.ignore_config {
    let config_path = if let Some(config_file) = get_config_file_path(cli.config) {
      Some(config_file)
    } else if let Some(config_from_xdg) = get_config_from_xdg() {
      Some(config_from_xdg)
    } else {
      None
    };

    if let Some(config_path) = config_path {
      if config.debug {
        eprintln!("Found config file `{}`", config_path.display());
      }
      config.merge(Config::from_file(&config_path)?);
    }
  } else if config.debug {
    eprintln!("`ignore_config` detected");
  }

  if let Some(overrides) = cli.overrides {
    config.merge(overrides);
  }

  if let Some(vars) = cli.templates_vars {
    config.vars.extend(vars);
  }

  match cli.command {
    Ts {
      typescript_overrides,
      ..
    } => {
      let typescript = config.typescript.get_or_insert_default();

      if let Some(typescript_overrides) = typescript_overrides {
        typescript.merge(typescript_overrides);
      }
    }
    _ => {}
  };

  Ok(config)
}

async fn execute_cli(cli: Cli) -> Result<(), GenError> {
  let is_dry_run = cli.dry_run;
  let command = cli.command.clone();

  let root_dir = cli.out_dir.clone().unwrap_or_else(|| get_cwd());
  create_all_dirs(&root_dir)?;

  let config = get_config_from_cli(cli).await?;

  let overwrite = !config.no_overwrite;

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
    Commands::PreCommit { output, preset } => {
      let content = config
        .pre_commit_presets
        .get(&preset)
        .ok_or(GenError::PresetNotFound {
          kind: Preset::PreCommit,
          name: preset.clone(),
        })?
        .clone()
        .process_data(preset.as_str(), &config.pre_commit_presets)?;

      let output = root_dir.join(output.unwrap_or_else(|| ".pre-commit-config.yaml".into()));

      serialize_yaml(&content, &output, overwrite)?;
    }
    Repo {
      remote,
      input,
      preset,
    } => {
      let mut preset = if let Some(id) = preset {
        config
          .git_presets
          .get(&id)
          .ok_or(GenError::PresetNotFound {
            kind: Preset::Repo,
            name: id.clone(),
          })?
          .clone()
      } else {
        RepoPreset::default()
      };

      if input.no_pre_commit {
        preset.pre_commit = PreCommitSetting::Bool(false)
      } else if let Some(preset_id) = input.pre_commit {
        preset.pre_commit = PreCommitSetting::Id(preset_id);
      };

      if let Some(gitignore) = input.gitignore {
        preset.gitignore = Some(GitIgnoreSetting::Id(gitignore));
      }

      if let Some(templates) = input.with_templates {
        preset
          .with_templates
          .get_or_insert_default()
          .extend(templates);
      }

      exit_if_dry_run!();

      config.init_repo(preset, remote.as_deref(), &root_dir)?;
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

      let new_config = Config::default();

      match format.as_ref() {
        "yaml" => serialize_yaml(&new_config, &output_path, overwrite)?,
        "toml" => {
          serialize_toml(&new_config, &output_path, overwrite)?;
        }
        "json" => serialize_json(&new_config, &output_path, overwrite)?,
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
      handle_ts_commands(config, command, root_dir).await?;
    }
  }
  Ok(())
}

#[derive(Parser, Debug, Clone)]
#[command(name = "sketch")]
#[command(version, about, long_about = None)]
pub struct Cli {
  /// Sets a custom config file. Any file named `sketch.{yaml,json,toml}` in the cwd or in `XDG_CONFIG_HOME/sketch` will be detected automatically. If no file is found, the default settings are used
  #[arg(short, long, value_name = "FILE", group = "config-file")]
  pub config: Option<PathBuf>,

  /// Ignores any automatically detected config files, uses cli instructions only
  #[arg(long, group = "config-file")]
  pub ignore_config: bool,

  #[command(subcommand)]
  pub command: Commands,

  #[command(flatten)]
  pub overrides: Option<Config>,

  /// The base path for the output files.
  #[arg(short, long, value_name = "PATH")]
  pub out_dir: Option<PathBuf>,

  /// Aborts before writing any content to disk.
  #[arg(long)]
  pub dry_run: bool,

  /// Sets a variable (as key=value) to use in templates. Overrides global and local variables.
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

#[derive(Args, Debug, Clone)]
pub struct RepoConfigInput {
  /// Does not generate a pre-commit config. It overrides the value in the git preset if one is being used.
  #[arg(long, group = "pre-commit")]
  no_pre_commit: bool,

  /// Selects a pre-commit preset. It overrides the value in the git preset if one is being used.
  #[arg(long, group = "pre-commit")]
  pre_commit: Option<String>,

  /// Selects a gitignore preset. It overrides the value in the git preset if one is being used.
  #[arg(long)]
  gitignore: Option<String>,

  /// One or many templates to render in the new repo's root. If a preset is being used, the list is extended and not replaced.
  #[arg(short = 't', long = "with-template", value_parser = TemplateOutput::from_cli, value_name = "id=TEMPLATE_ID,output=PATH")]
  with_templates: Option<Vec<TemplateOutput>>,
}

/// The cli commands.
#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
  /// Generates a `pre-commit` config file from a preset.
  PreCommit {
    /// The output path of the created file [default: `.pre-commit-config.yaml`]
    output: Option<PathBuf>,

    /// The preset id
    #[arg(short, long, value_name = "ID")]
    preset: String,
  },

  /// Launches typescript-specific commands.
  Ts {
    #[command(flatten)]
    typescript_overrides: Option<TypescriptConfig>,

    #[command(subcommand)]
    command: TsCommands,
  },

  /// Creates a new git repo.
  Repo {
    /// Selects a git preset from a configuration file.
    #[arg(short, long)]
    preset: Option<String>,

    #[command(flatten)]
    input: RepoConfigInput,

    /// The link of the git remote to use for the new repo.
    #[arg(long)]
    remote: Option<String>,
  },

  /// Generates a new config file.
  New {
    /// The output file. Must be an absolute path or a path relative to the cwd [default: sketch.yaml]
    output: Option<PathBuf>,
  },

  /// Renders a single template to a file or to stdout
  Render {
    #[command(flatten)]
    output: RenderingOutput,

    /// The path to the template file, as an absolute path or relative to the cwd
    #[arg(short, long, group = "input")]
    file: Option<PathBuf>,

    /// The id of the template to use (a name for config-defined templates, or a relative path to a file from `templates_dir`)
    #[arg(short, long, group = "input")]
    id: Option<String>,

    /// The literal definition for the template
    #[arg(short, long, group = "input")]
    content: Option<String>,
  },

  /// Renders a templating preset defined in a configuration file
  RenderPreset {
    /// The id of the preset.
    id: String,
  },

  /// Renders a template and executes it as a shell command
  Exec {
    /// The literal definition for the template
    #[arg(group = "input")]
    cmd: Option<String>,

    /// The cwd for the command to execute [default: `.`]
    #[arg(long)]
    cwd: Option<PathBuf>,

    /// The path to the command's template file, as an absolute path or relative to the cwd
    #[arg(short, long, group = "input")]
    file: Option<PathBuf>,

    /// The id of the template to use (a name for config-defined templates, or a relative path to a file from `templates_dir`)
    #[arg(short, long, group = "input")]
    template: Option<String>,
  },
}
