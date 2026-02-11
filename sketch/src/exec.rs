use crate::tera_setup::get_default_context;
use crate::*;

/// A command (rendered as a template) to execute
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Hook {
	/// The template id or definition for the command to execute
	pub command: TemplateRef,
	/// Local context variables (they override previously set variables with the same name)
	#[serde(default)]
	pub context: IndexMap<String, Value>,
}

impl FromStr for Hook {
	type Err = Infallible;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(Self {
			command: TemplateRef::Id(s.trim().to_string()),
			context: Default::default(),
		})
	}
}

pub(crate) const fn default_shell() -> &'static str {
	if cfg!(target_os = "windows") {
		"cmd.exe"
	} else {
		"sh"
	}
}

impl Config {
	pub fn execute_command(
		&self,
		shell: Option<&str>,
		cwd: &Path,
		commands: Vec<Hook>,
		cli_vars: &IndexMap<String, Value>,
		print_cmd: bool,
	) -> Result<(), GenError> {
		let mut tera = self.initialize_tera()?;

		let mut context = create_context(&self.vars)?;

		context.extend(get_default_context());

		let context = Arc::new(context);

		for cmd in commands {
			let mut overrides = cmd.context;

			for (key, val) in cli_vars {
				overrides.insert(key.clone(), val.clone());
			}

			let local_context =
				get_local_context(ContextRef::Original(context.clone()), &overrides);

			let template_name = match &cmd.command {
				TemplateRef::Id(id) => id,
				TemplateRef::Inline { name, content } => {
					tera.add_raw_template(name, content)
						.map_err(|e| GenError::TemplateParsing {
							template: name.clone(),
							source: e,
						})?;

					name
				}
			};

			let rendered_command = tera
				.render(template_name, local_context.as_ref())
				.map_err(|e| GenError::TemplateParsing {
					template: template_name.clone(),
					source: e,
				})?;

			if print_cmd {
				println!("Rendered command:");
				println!("{rendered_command}");
			}

			let shell = shell.unwrap_or_else(|| default_shell());

			let shell_arg = if shell == "cmd.exe" { "/C" } else { "-c" };

			create_all_dirs(cwd)?;

			launch_command(shell, &[shell_arg, &rendered_command], cwd, None)?;
		}

		Ok(())
	}
}

pub(crate) fn launch_command(
	program: &str,
	commands: &[&str],
	cwd: &Path,
	custom_error_message: Option<&str>,
) -> Result<(), GenError> {
	let output = Command::new(program)
		.args(commands)
		.current_dir(cwd)
		.stdout(Stdio::inherit())
		.stderr(Stdio::inherit())
		.output()
		.map_err(|e| {
			GenError::Custom(format!(
				"Failed to execute shell command '{}': {}",
				commands.join(" "),
				e
			))
		})?;

	if output.status.success() {
		Ok(())
	} else {
		let error_message = custom_error_message.map_or_else(
			|| {
				format!(
					"Shell command '{}' failed with exit code: {:?}",
					commands.join(" "),
					output.status.code()
				)
			},
			|m| m.to_string(),
		);

		Err(GenError::Custom(error_message))
	}
}
