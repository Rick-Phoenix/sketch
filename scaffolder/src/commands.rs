use std::process::{Command, Stdio};

fn execute_shell_command(command_string: &str) -> Result<(), String> {
  let shell_executable = if cfg!(target_os = "windows") {
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
