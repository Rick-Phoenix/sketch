use std::{
  env::current_dir,
  fs::create_dir_all,
  path::PathBuf,
  process::{Command, Stdio},
};

use tera::Context;

use crate::{tera::get_default_context, Config, GenError};

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
    cwd: Option<PathBuf>,
    command: &str,
  ) -> Result<(), GenError> {
    let mut tera = self.initialize_tera()?;

    let mut context = Context::from_serialize(self.global_templates_vars)
      .map_err(|e| GenError::TemplateContextParsing { source: e })?;

    context.extend(get_default_context());

    tera
      .add_raw_template("__command", command)
      .map_err(|e| GenError::TemplateParsing {
        template: "__command".to_string(),
        source: e,
      })?;

    let rendered_command =
      tera
        .render("__command", &context)
        .map_err(|e| GenError::TemplateParsing {
          template: "command".to_string(),
          source: e,
        })?;

    if self.debug {
      println!("DEBUG: Rendered command: {}", rendered_command);
    }

    let shell = shell.unwrap_or_else(|| default_shell());

    let shell_arg = if shell == "cmd.exe" { "/C" } else { "-c" };

    let dir = cwd.unwrap_or_else(|| current_dir().expect("Could not get the cwd."));

    create_dir_all(&dir).map_err(|e| GenError::DirCreation {
      path: dir.clone(),
      source: e,
    })?;

    launch_command(Some(shell), &[shell_arg], &dir.to_string_lossy(), None)
  }
}

pub(crate) fn launch_command(
  shell: Option<&str>,
  commands: &[&str],
  cwd: &str,
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
