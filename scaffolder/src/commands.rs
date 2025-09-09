use std::process::{Command, Stdio};

use tera::{Context, Tera};

use crate::{Config, GenError};

impl Config {
  pub fn execute_command(self, shell: Option<&str>, command: &str) -> Result<(), GenError> {
    let context = Context::from_serialize(self.global_templates_vars)
      .map_err(|e| GenError::TemplateContextParsing { source: e })?;

    let result =
      Tera::one_off(command, &context, false).map_err(|e| GenError::TemplateParsing {
        template: "command".to_string(),
        source: e,
      })?;

    execute_shell_command(shell, &result).map_err(|e| GenError::Custom(e.to_string()))?;

    Ok(())
  }
}

fn execute_shell_command(shell: Option<&str>, command_string: &str) -> Result<(), String> {
  let shell_executable = if let Some(shell) = shell {
    shell
  } else if cfg!(target_os = "windows") {
    "cmd.exe"
  } else {
    "sh"
  };

  let shell_arg = if cfg!(target_os = "windows") {
    "/C"
  } else {
    "-c"
  };

  let output = Command::new(shell_executable)
    .arg(shell_arg)
    .arg(command_string)
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit())
    .output()
    .map_err(|e| {
      format!(
        "Failed to execute shell command '{}': {}",
        command_string, e
      )
    })?;

  if !output.status.success() {
    Err(format!(
      "Shell command '{}' failed with exit code: {:?}",
      command_string,
      output.status.code()
    ))
  } else {
    Ok(())
  }
}
