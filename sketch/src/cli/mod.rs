#[cfg(test)]
mod cli_tests;

mod config_discovery;
mod ts_cmds;

mod cli_elements;
pub(crate) mod parsers;

use std::{fmt::Debug, fs::read_to_string, path::PathBuf};

use clap::{Args, Parser, Subcommand};
use indexmap::IndexMap;
use parsers::parse_serializable_key_value_pair;
use serde_json::Value;
use Commands::*;

use crate::{
  cli::{
    cli_elements::TemplateRef,
    config_discovery::get_config_from_cli,
    ts_cmds::{handle_ts_commands, TsCommands},
  },
  custom_templating::{
    PresetElement, TemplateData, TemplateOutput, TemplateOutputKind, TemplatingPreset,
    TemplatingPresetReference,
  },
  exec::Hook,
  fs::{
    create_all_dirs, create_parent_dirs, get_cwd, get_extension, serialize_json, serialize_toml,
    serialize_yaml, write_file,
  },
  git_workflow::WorkflowReference,
  init_repo::{gitignore::GitIgnoreSetting, pre_commit::PreCommitSetting, RepoPreset},
  licenses::License,
  serde_utils::deserialize_map,
  ts::TypescriptConfig,
  Config, *,
};

pub async fn main_entrypoint() -> Result<(), GenError> {
  execute_cli(Cli::parse()).await
}

async fn execute_cli(cli: Cli) -> Result<(), GenError> {
  let mut config = get_config_from_cli(cli.overrides.unwrap_or_default(), &cli.command).await?;

  let command = cli.command;
  let mut cli_vars: IndexMap<String, Value> = IndexMap::new();

  if let Some(cli_overrides) = cli.vars_overrides {
    for (name, value) in cli_overrides {
      cli_vars.insert(name, value);
    }
  }

  for file in cli.vars_files {
    let vars = deserialize_map(&file)?;
    config.vars.extend(vars);
  }

  let overwrite = config.can_overwrite();

  if cli.print_config {
    println!("Full parsed config:");
    println!("{config:?}");
  }

  match command {
    Commands::GhWorkflow { preset, file, dir } => {
      let data = config
        .github
        .workflow_presets
        .get(&preset)
        .ok_or(GenError::PresetNotFound {
          kind: Preset::GithubWorkflow,
          name: preset.clone(),
        })?
        .clone()
        .process_data(&preset, &config.github)?;

      let workflows_dir = dir.unwrap_or_else(|| PathBuf::from(".github/workflows"));

      let output = workflows_dir.join(file);

      serialize_yaml(&data, &output, overwrite)?;
    }
    Commands::License { license, output } => {
      let output = output.unwrap_or_else(|| "LICENSE".into());

      write_file(&output, license.get_content(), overwrite)?;
    }
    Commands::PnpmWorkspace { output, preset } => {
      let typescript = config.typescript.unwrap_or_default();

      let content = typescript
        .pnpm_presets
        .get(&preset)
        .ok_or(GenError::PresetNotFound {
          kind: Preset::PnpmWorkspace,
          name: preset.clone(),
        })?
        .clone()
        .process_data(preset.as_str(), &typescript.pnpm_presets)?;

      let output = output.unwrap_or_else(|| "pnpm-workspace.yaml".into());

      create_parent_dirs(&output)?;

      serialize_yaml(&content, &output, overwrite)?;
    }
    Commands::TsConfig { output, preset } => {
      let typescript = config.typescript.unwrap_or_default();

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
    Commands::Oxlint { output, preset } => {
      let typescript = config.typescript.unwrap_or_default();

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
    Commands::PackageJson { output, preset } => {
      let typescript = config.typescript.unwrap_or_default();

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

    Commands::CargoToml { output, preset } => {
      let content = config
        .cargo_toml_presets
        .get(&preset)
        .ok_or(GenError::PresetNotFound {
          kind: Preset::CargoToml,
          name: preset.clone(),
        })?
        .clone()
        .process_data(preset.as_str(), &config.cargo_toml_presets)?;

      let output = output.unwrap_or_else(|| "Cargo.toml".into());

      create_parent_dirs(&output)?;

      serialize_toml(&content, &output, overwrite)?;
    }

    Commands::DockerCompose { output, preset } => {
      let docker_config = config.docker.unwrap_or_default();
      let compose_presets = docker_config.compose_presets;

      let content = compose_presets
        .get(&preset)
        .ok_or(GenError::PresetNotFound {
          kind: Preset::DockerCompose,
          name: preset.clone(),
        })?
        .clone()
        .process_data(
          preset.as_str(),
          &compose_presets,
          &docker_config.service_presets,
        )?;

      let output = output.unwrap_or_else(|| "compose.yaml".into());

      create_parent_dirs(&output)?;

      serialize_yaml(&content, &output, overwrite)?;
    }
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
      input:
        RepoConfigInput {
          no_pre_commit,
          pre_commit,
          gitignore,
          license,
          with_templates,
          workflows,
        },
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

      if no_pre_commit {
        preset.pre_commit = PreCommitSetting::Bool(false)
      } else if let Some(preset_id) = pre_commit {
        preset.pre_commit = PreCommitSetting::Id(preset_id);
      };

      if let Some(gitignore) = gitignore {
        preset.gitignore = Some(GitIgnoreSetting::Id(gitignore));
      }

      if let Some(template_refs) = with_templates {
        let templates_list = preset.with_templates.get_or_insert_default();

        let mut single_templates: Vec<PresetElement> = Vec::new();

        for template in template_refs {
          match template {
            TemplateRef::PresetId(id) => templates_list.push(TemplatingPresetReference::Preset {
              id,
              context: Default::default(),
            }),
            TemplateRef::Template(def) => single_templates.push(PresetElement::Template(def)),
          };
        }

        if !single_templates.is_empty() {
          templates_list.push(TemplatingPresetReference::Definition(TemplatingPreset {
            templates: single_templates,
            ..Default::default()
          }));
        }
      }

      if let Some(license) = license {
        preset.license = Some(license);
      }

      if let Some(workflows) = workflows {
        preset.workflows.extend(workflows);
      }

      let out_dir = dir.unwrap_or_else(|| get_cwd());

      create_all_dirs(&out_dir)?;

      config.init_repo(preset, remote.as_deref(), &out_dir, &cli_vars)?;
    }

    RenderPreset { id, out_dir } => {
      let out_dir = out_dir.unwrap_or_else(|| get_cwd());

      config.generate_templates(
        &out_dir,
        vec![TemplatingPresetReference::Preset {
          id,
          context: Default::default(),
        }],
        &cli_vars,
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

      config.generate_templates(
        get_cwd(),
        vec![TemplatingPresetReference::Definition(TemplatingPreset {
          templates: vec![PresetElement::Template(template)],
          ..Default::default()
        })],
        &cli_vars,
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
      shell,
      print_cmd,
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

      let shell = if let Some(cli_flag) = shell {
        Some(cli_flag)
      } else {
        config.shell.clone()
      };

      config.execute_command(
        shell.as_deref(),
        &cwd,
        vec![Hook {
          command,
          context: Default::default(),
        }],
        &cli_vars,
        print_cmd,
      )?;
    }
    Ts { command, .. } => {
      handle_ts_commands(config, command, &cli_vars).await?;
    }
  }
  Ok(())
}

#[derive(Args, Debug, Clone, Default)]
pub struct ConfigOverrides {
  /// The path to the templates directory.
  #[arg(long, value_name = "DIR")]
  pub templates_dir: Option<PathBuf>,

  /// Do not overwrite existing files.
  #[arg(long)]
  pub no_overwrite: bool,

  /// Sets a custom config file. Any file named `sketch.{yaml,json,toml}` in the cwd or in `XDG_CONFIG_HOME/sketch` will be detected automatically. If no file is found, the default settings are used
  #[arg(short, long, value_name = "FILE", group = "config-file")]
  pub config: Option<PathBuf>,

  /// Ignores any automatically detected config files, uses cli instructions only
  #[arg(long, group = "config-file")]
  pub ignore_config: bool,
}

#[derive(Parser, Debug, Clone)]
#[command(name = "sketch")]
#[command(version, about, long_about = None)]
pub struct Cli {
  /// Prints the full parsed config
  #[arg(long)]
  pub print_config: bool,

  #[command(subcommand)]
  pub command: Commands,

  #[command(flatten)]
  pub overrides: Option<ConfigOverrides>,

  /// Sets a variable (as key=value) to use in templates. Overrides global and local variables. Values must be in valid JSON
  #[arg(long = "set", short = 's', value_parser = parse_serializable_key_value_pair, value_name = "KEY=VALUE")]
  pub vars_overrides: Option<Vec<(String, Value)>>,

  /// One or more paths to json, yaml or toml files to extract template variables from, in the given order.
  #[arg(long = "vars-file")]
  pub vars_files: Vec<PathBuf>,
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

  /// A license file to generate for the new repo.
  license: Option<License>,

  /// One or many individual templates or templating presets to render in the new repo
  #[arg(
    short,
    long = "with-template",
    value_name = "PRESET_ID|id=TEMPLATE_ID,output=PATH"
  )]
  with_templates: Option<Vec<TemplateRef>>,

  /// One or many workflow presets to use for the new repo. The file path will be joined to `.github/workflows`
  #[arg(
    long = "workflow",
    value_name = "id=PRESET_ID,file=PATH",
    value_parser = WorkflowReference::from_cli
  )]
  workflows: Option<Vec<WorkflowReference>>,
}

/// The cli commands.
#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
  /// Generates a new config file.
  New {
    /// The output file [default: sketch.yaml]
    output: Option<PathBuf>,
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

  /// Renders a templating preset
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

    /// Prints the rendered command to stdout before executing it
    #[arg(long)]
    print_cmd: bool,

    /// The shell to use for commands [default: `cmd.exe` on windows and `sh` elsewhere].
    #[arg(short, long)]
    shell: Option<String>,

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

  /// Generates a Github workflow.
  GhWorkflow {
    /// The preset id
    preset: String,

    /// The name of the workflow's file, to join to the workflows directory
    file: PathBuf,

    /// The path to the workflows dir [default: `.github/workflows`]
    #[arg(short, long)]
    dir: Option<PathBuf>,
  },

  /// Generates a Docker Compose file from a preset.
  DockerCompose {
    /// The preset id
    preset: String,

    /// The output path of the created file [default: `compose.yaml`]
    output: Option<PathBuf>,
  },

  /// Generates a `pre-commit` config file from a preset.
  PreCommit {
    /// The preset id
    preset: String,

    /// The output path of the created file [default: `.pre-commit-config.yaml`]
    output: Option<PathBuf>,
  },

  /// Generates a `Cargo.toml` file from a preset.
  CargoToml {
    /// The preset id
    preset: String,

    /// The output path of the created file [default: `Cargo.toml`]
    output: Option<PathBuf>,
  },

  /// Executes typescript-specific commands.
  Ts {
    #[command(flatten)]
    typescript_overrides: Option<TypescriptConfig>,

    #[command(subcommand)]
    command: TsCommands,
  },

  /// Generates a `package.json` file from a preset.
  PackageJson {
    /// The preset id
    preset: String,

    /// The output path of the generated file [default: `package.json`]
    output: Option<PathBuf>,
  },

  /// Generates a `tsconfig.json` file from a preset.
  TsConfig {
    /// The preset id
    preset: String,

    /// The output path of the generated file [default: `tsconfig.json`]
    output: Option<PathBuf>,
  },

  /// Generates a `.oxlintrc.json` file from a preset.
  Oxlint {
    /// The preset id
    preset: String,

    /// The output path of the generated file [default: `.oxlintrc.json`]
    output: Option<PathBuf>,
  },

  /// Generates a `pnpm-workspace.yaml` file from a preset.
  PnpmWorkspace {
    /// The preset id
    preset: String,

    /// The output path of the generated file [default: `pnpm-workspace.yaml`]
    output: Option<PathBuf>,
  },

  License {
    #[arg(value_enum)]
    license: License,

    /// The path of the output file [default: `LICENSE`]
    #[arg(short, long)]
    output: Option<PathBuf>,
  },
}
