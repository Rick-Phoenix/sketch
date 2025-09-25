use std::{
  path::Path,
  process::{Command, Stdio},
};

use serde_json::Value;
use tera::Context;

use crate::{
  custom_templating::TemplateData, fs::create_all_dirs, tera_setup::get_default_context, Config,
  GenError,
};

pub(crate) fn default_shell() -> &'static str {
  if cfg!(target_os = "windows") {
    "cmd.exe"
  } else {
    "sh"
  }
}

impl Config {
  pub fn execute_command(
    self,
    shell: Option<&str>,
    cwd: &Path,
    command_template: TemplateData,
    cli_vars: Option<Vec<(String, Value)>>,
  ) -> Result<(), GenError> {
    let mut tera = self.initialize_tera()?;

    let mut context = Context::from_serialize(self.vars)
      .map_err(|e| GenError::TemplateContextParsing { source: e })?;

    context.extend(get_default_context());

    if let Some(overrides) = cli_vars {
      for (key, val) in overrides {
        context.insert(&key, &val);
      }
    }

    let template_name = match command_template {
      TemplateData::Id(id) => id,
      TemplateData::Content { name, content } => {
        tera
          .add_raw_template(&name, &content)
          .map_err(|e| GenError::TemplateParsing {
            template: name.clone(),
            source: e,
          })?;

        name
      }
    };

    let rendered_command =
      tera
        .render(&template_name, &context)
        .map_err(|e| GenError::TemplateParsing {
          template: template_name,
          source: e,
        })?;

    let shell = shell.unwrap_or_else(|| default_shell());

    let shell_arg = if shell == "cmd.exe" { "/C" } else { "-c" };

    create_all_dirs(cwd)?;

    launch_command(shell, &[shell_arg, &rendered_command], cwd, None)
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

  if !output.status.success() {
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

    Err(GenError::Custom(error_message.to_string()))
  } else {
    Ok(())
  }
}
