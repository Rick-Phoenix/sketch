#[cfg(test)]
mod cli_tests;

mod config_discovery;

use config_discovery::*;

mod ts_cmds;
use ts_cmds::*;

mod rust_cmds;
use rust_cmds::*;

mod exec_cmd;
use exec_cmd::*;

pub(crate) mod parsers;

use clap::Subcommand;

use crate::{
	docker::{ComposeFilePreset, ServiceFromCli, ServicePresetRef},
	exec::Hook,
	init_repo::RepoPreset,
	licenses::License,
	rust::{CargoTomlPresetRef, CratePreset},
	ts::TypescriptConfig,
	*,
};

pub async fn main_entrypoint() -> Result<(), AppError> {
	Cli::parse().execute().await
}

impl Cli {
	async fn execute(self) -> Result<(), AppError> {
		let mut config = get_config_from_cli(self.overrides.unwrap_or_default(), &self.command)?;

		let command = self.command;
		let cli_vars: IndexMap<String, Value> = self.vars_overrides.into_iter().collect();

		for file in self.vars_files {
			let vars = deserialize_vars_file(&file)?;
			config.vars.extend(vars);
		}

		let overwrite = config.can_overwrite();

		if self.print_config {
			println!("Full parsed config:");
			println!("{config:?}");
		}

		match command {
			#[cfg(feature = "schemars")]
			Commands::JsonSchema { output } => {
				Config::generate_json_schema(&output)?;
			}
			Commands::Rust { command } => {
				command.execute(&config)?;
			}
			Commands::Gitignore { preset, output } => {
				let data = config.get_gitignore_preset(&preset)?;

				write_file(
					&output.unwrap_or_else(|| PathBuf::from(".gitignore")),
					&data.content.to_string(),
					overwrite,
				)?;
			}
			Commands::GhWorkflow { preset, output } => {
				let data = config.github.get_workflow(&preset)?;

				create_parent_dirs(&output)?;

				serialize_yaml(&data, &output, overwrite)?;
			}
			Commands::License { license, output } => {
				let output = output.unwrap_or_else(|| "LICENSE".into());

				write_file(&output, license.get_content(), overwrite)?;
			}
			Commands::PnpmWorkspace { output, preset } => {
				let typescript = config.typescript.unwrap_or_default();

				let content = typescript.get_pnpm_preset(&preset)?.config;

				let output = output.unwrap_or_else(|| "pnpm-workspace.yaml".into());

				create_parent_dirs(&output)?;

				serialize_yaml(&content, &output, overwrite)?;
			}
			Commands::Oxlint { output, preset } => {
				let typescript = config.typescript.unwrap_or_default();

				let content = typescript.get_oxlint_preset(&preset)?.config;

				let output = output.unwrap_or_else(|| ".oxlintrc.json".into());
				create_parent_dirs(&output)?;

				serialize_json(&content, &output, overwrite)?;
			}
			Commands::PackageJson { output, preset } => {
				let typescript = config.typescript.unwrap_or_default();

				let content = typescript.get_package_json(&preset)?;

				let output = output.unwrap_or_else(|| "package.json".into());
				create_parent_dirs(&output)?;

				serialize_json(&content, &output, overwrite)?;
			}

			Commands::DockerCompose {
				output,
				preset,
				services,
			} => {
				let docker_config = config.docker.unwrap_or_default();

				let mut file_preset = if let Some(id) = preset.as_ref() {
					docker_config.get_file_preset(id)?
				} else {
					ComposeFilePreset::default()
				};

				for service in services {
					let service_name = service
						.name
						.unwrap_or_else(|| service.preset_id.clone());

					file_preset
						.config
						.services
						.insert(service_name, ServicePresetRef::PresetId(service.preset_id));
				}

				let file_data = file_preset
					.process_data(preset.as_deref().unwrap_or("__from_cli"), &docker_config)?;

				let output = output.unwrap_or_else(|| "compose.yaml".into());

				create_parent_dirs(&output)?;

				serialize_yaml(&file_data, &output, overwrite)?;
			}
			Commands::PreCommit { output, preset } => {
				let content = config.get_pre_commit_preset(&preset)?.config;

				let output = output.unwrap_or_else(|| ".pre-commit-config.yaml".into());

				create_parent_dirs(&output)?;

				serialize_yaml(&content, &output, overwrite)?;
			}
			Commands::Repo {
				remote,
				preset,
				dir,
				overrides,
			} => {
				let mut preset = if let Some(id) = preset {
					config.get_repo_preset(&id)?
				} else {
					RepoPreset::default()
				};

				if let Some(overrides) = overrides {
					preset.merge(overrides);
				}

				let out_dir = dir.unwrap_or_else(get_cwd);

				create_all_dirs(&out_dir)?;

				config.init_repo(preset, remote.as_deref(), &out_dir, &cli_vars)?;
			}

			Commands::Render {
				template: template_id,
				content,
				output,
				file,
				preset: preset_id,
			} => {
				let is_single_template = preset_id.is_none();

				let output_root = if is_single_template {
					get_cwd()
				} else {
					output
						.clone()
						.context("The output path is required when using presets")?
				};

				let template_data = if let Some(preset_id) = preset_id {
					TemplatingPresetRef::PresetId {
						preset_id,
						context: Default::default(),
					}
				} else {
					let template = if let Some(id) = template_id {
						TemplateRef::Id(id)
					} else if let Some(content) = content {
						TemplateRef::Inline {
							name: "__from_cli".to_string(),
							content,
						}
					} else if let Some(file) = file {
						let file_content =
							read_to_string(&file).map_err(|e| AppError::ReadError {
								path: file.clone(),
								source: e,
							})?;

						TemplateRef::Inline {
							name: format!("__from_file_{}", file.display()),
							content: file_content,
						}
					} else {
						return Err(anyhow!("Missing id or content for template generation").into());
					};

					let output = if let Some(path) = output {
						TemplateOutputKind::Path(path)
					} else {
						TemplateOutputKind::Stdout
					};

					TemplatingPresetRef::Preset(TemplatingPreset {
						templates: vec![TemplateKind::Single(TemplateData { template, output })],
						..Default::default()
					})
				};

				config.generate_templates(&output_root, vec![template_data], &cli_vars)?;
			}
			Commands::New { output } => {
				let output_path = output.unwrap_or_else(|| PathBuf::from("sketch.yaml"));

				create_parent_dirs(&output_path)?;

				let format = get_extension(&output_path)?.to_string_lossy();

				let new_config = Config::default();

				match format.as_ref() {
					"yaml" => serialize_yaml(&new_config, &output_path, overwrite)?,
					"toml" => {
						serialize_toml(&new_config, &output_path, overwrite)?;
					}
					"json" => serialize_json(&new_config, &output_path, overwrite)?,
					_ => {
						return Err(anyhow!(
							"Invalid config format. Allowed formats are: yaml, toml, json"
						)
						.into());
					}
				};
			}
			Commands::Exec { cmd } => {
				cmd.execute(&config, &cli_vars)?;
			}
			Commands::Ts { command, .. } => {
				command.execute(config, &cli_vars).await?;
			}
		}
		Ok(())
	}
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
	#[arg(short, long, value_name = "FILE")]
	pub config: Option<PathBuf>,

	/// Ignores any automatically detected config files, uses cli instructions and config file defined with --config.
	#[arg(long)]
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
	#[arg(long = "set", short = 'S', value_parser = parse_serializable_key_value_pair, value_name = "KEY=VALUE")]
	pub vars_overrides: Vec<(String, Value)>,

	/// One or more paths to json, yaml or toml files to extract template variables from, in the given order.
	#[arg(long = "vars-file")]
	pub vars_files: Vec<PathBuf>,
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
		overrides: Option<RepoPreset>,

		/// The link of the git remote to use for the new repo.
		#[arg(short, long)]
		remote: Option<String>,
	},

	/// Renders a single template to a file or to stdout
	Render {
		/// The output path for the template/preset. Implies `stdout` if absent for single templates. Required when a preset is selected.
		#[arg(requires = "input")]
		output: Option<PathBuf>,

		/// The id of a templating preset
		#[arg(short, long, group = "input", requires = "output")]
		preset: Option<String>,

		/// The path to the template file
		#[arg(short, long, group = "input")]
		file: Option<PathBuf>,

		/// The id of the template to use (a name for config-defined templates, or a relative path to a file from `templates_dir`)
		#[arg(short, long, group = "input")]
		template: Option<String>,

		/// The literal definition for the template
		#[arg(short, long, group = "input")]
		content: Option<String>,
	},

	/// Renders a template and executes it as a shell command
	Exec {
		#[command(flatten)]
		cmd: ExecCmd,
	},

	/// Generates a `.gitignore` file from a preset.
	Gitignore {
		/// The preset id
		preset: String,

		/// The output path of the new file [default: `.gitignore`]
		output: Option<PathBuf>,
	},

	/// Generates a Github workflow.
	GhWorkflow {
		/// The preset id
		preset: String,

		/// The output path of the new file
		output: PathBuf,
	},

	/// Generates a Docker Compose file from a preset.
	DockerCompose {
		/// The preset id. Not required if services are added manually with the `--service` flag.
		preset: Option<String>,

		/// The output path of the new file [default: `compose.yaml`]
		output: Option<PathBuf>,

		/// Adds one or many service presets to the generated file. Can specify the preset ID and the name of the service in the output file, or just the preset ID to also use it for the service name.
		#[arg(short = 's', long = "service", value_parser = ServiceFromCli::from_cli, help = "PRESET_ID|id=PRESET,name=NAME")]
		services: Vec<ServiceFromCli>,
	},

	/// Generates a `pre-commit` config file from a preset.
	PreCommit {
		/// The preset id
		preset: String,

		/// The output path of the new file [default: `.pre-commit-config.yaml`]
		output: Option<PathBuf>,
	},

	/// The subcommands to generate files used in Rust workspaces.
	Rust {
		#[command(subcommand)]
		command: RustCommands,
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

	/// Generates a license file
	License {
		#[arg(value_enum)]
		license: License,

		/// The path of the output file [default: `LICENSE`]
		#[arg(short, long)]
		output: Option<PathBuf>,
	},

	#[cfg(feature = "schemars")]
	/// Generates the json schema for the configuration file
	JsonSchema {
		/// The output path for the json schema
		output: PathBuf,
	},
}
