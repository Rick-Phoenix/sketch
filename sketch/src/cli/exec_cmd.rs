use super::*;

#[derive(Args, Debug, Clone)]
pub struct ExecCmd {
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
}

impl ExecCmd {
	pub fn execute(self, config: &Config, cli_vars: &IndexMap<String, Value>) -> AppResult {
		let Self {
			cmd: command,
			print_cmd,
			shell,
			cwd,
			file,
			template,
		} = self;

		let command = if let Some(literal) = command {
			TemplateRef::Inline {
				name: "__from_cli".to_string(),
				content: literal,
			}
		} else if let Some(id) = template {
			TemplateRef::Id(id)
		} else if let Some(file_path) = file {
			let content = read_to_string(&file_path).map_err(|e| AppError::ReadError {
				path: file_path.clone(),
				source: e,
			})?;

			TemplateRef::Inline {
				name: format!("__from_file_{}", file_path.display()),
				content,
			}
		} else {
			return Err(anyhow!("At least one between command and file must be set.").into());
		};

		let cwd = cwd.unwrap_or_else(get_cwd);

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
			cli_vars,
			print_cmd,
		)?;

		Ok(())
	}
}
