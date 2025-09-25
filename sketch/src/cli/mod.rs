#[cfg(test)]
mod cli_tests;

mod config_discovery;
mod ts_cmds;

mod cli_elements;
pub(crate) mod parsers;

use std::{fmt::Debug, fs::read_to_string, path::PathBuf};

use clap::{Args, Parser, Subcommand};
use parsers::parse_serializable_key_value_pair;
use serde_json::Value;
use Commands::*;

use crate::{
  cli::{
    config_discovery::get_config_from_cli,
    ts_cmds::{handle_ts_commands, TsCommands},
  },
  custom_templating::{
    TemplateData, TemplateOutput, TemplateOutputKind, TemplatingPreset, TemplatingPresetReference,
  },
  fs::{
    create_all_dirs, create_parent_dirs, get_cwd, get_extension, serialize_json, serialize_toml,
    serialize_yaml,
  },
  init_repo::{gitignore::GitIgnoreSetting, pre_commit::PreCommitSetting, RepoPreset},
  ts::TypescriptConfig,
  Config, *,
};

pub async fn main_entrypoint() -> Result<(), GenError> {
  execute_cli(Cli::parse()).await
}

async fn execute_cli(cli: Cli) -> Result<(), GenError> {
  let command = cli.command.clone();
  let cli_vars = cli.templates_vars.clone();

  let config = get_config_from_cli(cli).await?;

  let debug = config.debug;
  let overwrite = !config.no_overwrite;

  if debug {
    log_debug("Config", &config);
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

      let output = output.unwrap_or_else(|| ".pre-commit-config.yaml".into());

      create_parent_dirs(&output)?;

      serialize_yaml(&content, &output, overwrite)?;
    }
    Repo {
      remote,
      input,
      preset,
      dir,
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

      if let Some(presets) = input.with_templ_preset {
        let templates_list = preset.with_templates.get_or_insert_default();

        for id in presets {
          templates_list.push(TemplatingPresetReference::Preset {
            id: id,
            context: Default::default(),
          });
        }
      }

      if let Some(templates) = input.with_templates {
        let templates_list = preset.with_templates.get_or_insert_default();

        for template in templates {
          templates_list.push(TemplatingPresetReference::Definition(
            TemplatingPreset::Single(template),
          ));
        }
      }

      let out_dir = dir.unwrap_or_else(|| get_cwd());

      create_all_dirs(&out_dir)?;

      config.init_repo(preset, remote.as_deref(), &out_dir, cli_vars)?;
    }

    RenderPreset { id, out_dir } => {
      let out_dir = out_dir.unwrap_or_else(|| get_cwd());

      config.generate_templates(
        &out_dir,
        vec![TemplatingPresetReference::Preset {
          id,
          context: Default::default(),
        }],
        cli_vars,
      )?;
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
        TemplateOutputKind::Stdout
      } else {
        TemplateOutputKind::Path(
          output
            .output_path
            .expect("At least one must be set between output_path and --stdout"),
        )
      };

      let template = TemplateOutput {
        output,
        template: template_data,
        context: Default::default(),
      };

      if debug {
        log_debug("Template", &template);
      }

      config.generate_templates(
        get_cwd(),
        vec![TemplatingPresetReference::Definition(
          TemplatingPreset::Collection {
            templates: vec![template],
            context: Default::default(),
          },
        )],
        cli_vars,
      )?;
    }
    New { output } => {
      let output_path = output.unwrap_or_else(|| PathBuf::from("sketch.yaml"));

      create_parent_dirs(&output_path)?;

      let format = get_extension(&output_path).to_string_lossy();

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

      let cwd = cwd.unwrap_or_else(|| get_cwd());

      let shell = config.shell.clone();
      config.execute_command(shell.as_deref(), &cwd, command, cli_vars)?;
    }
    Ts { command, .. } => {
      handle_ts_commands(config, command, cli_vars).await?;
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

  /// Sets a variable (as key=value) to use in templates. Overrides global and local variables. Values must be in valid JSON
  #[arg(long = "set", short = 's', value_parser = parse_serializable_key_value_pair, value_name = "KEY=VALUE")]
  pub templates_vars: Option<Vec<(String, Value)>>,
}

#[derive(Args, Debug, Clone)]
#[group(required = true, multiple = false)]
pub struct RenderingOutput {
  /// The output path for the generated file
  #[arg(requires = "input")]
  output_path: Option<PathBuf>,

  /// Prints the result to stdout
  #[arg(long, requires = "input")]
  stdout: bool,
}

#[derive(Args, Debug, Clone)]
pub struct RepoConfigInput {
  /// Do not generate a pre-commit config
  #[arg(long, group = "pre-commit")]
  no_pre_commit: bool,

  /// Selects a pre-commit preset
  #[arg(long, group = "pre-commit")]
  pre_commit: Option<String>,

  /// Selects a gitignore preset
  #[arg(long)]
  gitignore: Option<String>,

  /// One or many individual templates to render in the new repo
  #[arg(
    short,
    long = "with-template",
    value_name = "id=TEMPLATE_ID,output=PATH", value_parser = TemplateOutput::from_cli
  )]
  with_templates: Option<Vec<TemplateOutput>>,

  /// One or many templating presets to render in the new repo
  #[arg(short = 't', value_name = "ID")]
  with_templ_preset: Option<Vec<String>>,
}

/// The cli commands.
#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
  /// Generates a `pre-commit` config file from a preset.
  PreCommit {
    /// The preset id
    preset: String,

    /// The output path of the created file [default: `.pre-commit-config.yaml`]
    output: Option<PathBuf>,
  },

  /// Executes typescript-specific commands.
  Ts {
    #[command(flatten)]
    typescript_overrides: Option<TypescriptConfig>,

    #[command(subcommand)]
    command: TsCommands,
  },

  /// Creates a new git repo from a preset.
  Repo {
    /// The directory where the new repo should be generated. [default: `.`]
    dir: Option<PathBuf>,

    /// Selects a git preset from a configuration file.
    #[arg(short, long)]
    preset: Option<String>,

    #[command(flatten)]
    input: RepoConfigInput,

    /// The link of the git remote to use for the new repo.
    #[arg(short, long)]
    remote: Option<String>,
  },

  /// Generates a new config file.
  New {
    /// The output file [default: sketch.yaml]
    output: Option<PathBuf>,
  },

  /// Renders a single template to a file or to stdout
  Render {
    #[command(flatten)]
    output: RenderingOutput,

    /// The path to the template file
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

    /// The base path to join to relative output paths. [default: `.`]
    out_dir: Option<PathBuf>,
  },

  /// Renders a template and executes it as a shell command
  Exec {
    /// The literal definition for the template (incompatible with `--file` or `--template`)
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
