use std::{
  path::Path,
  process::{Command, Stdio},
};

use tera::Context;

use crate::{
  custom_templating::{get_default_context, TemplateData},
  fs::create_parent_dirs,
  Config, GenError,
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
  ) -> Result<(), GenError> {
    let mut tera = self.initialize_tera()?;

    let mut context = Context::from_serialize(self.vars)
      .map_err(|e| GenError::TemplateContextParsing { source: e })?;

    context.extend(get_default_context());

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
          template: "command".to_string(),
          source: e,
        })?;

    if self.debug {
      eprintln!("DEBUG: Rendered command: {}", rendered_command);
    }

    let shell = shell.unwrap_or_else(|| default_shell());

    let shell_arg = if shell == "cmd.exe" { "/C" } else { "-c" };

    create_parent_dirs(cwd)?;

    launch_command(Some(shell), &[shell_arg, &rendered_command], cwd, None)
  }
}

pub(crate) fn launch_command(
  shell: Option<&str>,
  commands: &[&str],
  cwd: &Path,
  custom_error_message: Option<&str>,
) -> Result<(), GenError> {
  let shell = shell.unwrap_or_else(|| default_shell());
  let output = Command::new(shell)
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
