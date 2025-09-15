use std::{fs::File, path::PathBuf};

use clap::Parser;
use pretty_assertions::assert_eq;

use super::reset_testing_dir;
use crate::{
  cli::{execute_cli, Cli},
  Config,
};

#[tokio::test]
async fn generated_configs() -> Result<(), Box<dyn std::error::Error>> {
  let output_dir = PathBuf::from("tests/output/generated_configs");

  reset_testing_dir(&output_dir);

  let default_config = Cli::try_parse_from([
    "sketch",
    "--no-config-file",
    "new",
    &output_dir.join("default_config.yaml").to_string_lossy(),
  ])?;

  execute_cli(default_config).await?;

  let default_config_output = deserialize_yaml!(Config, output_dir.join("default_config.yaml"));

  assert_eq!(default_config_output, Config::default());

  let with_extras = Cli::try_parse_from([
    "sketch",
    "--no-config-file",
    "--root-dir",
    "tests/output",
    "--templates-dir",
    "tests/templates",
    "--shell",
    "zsh",
    "--set",
    "hello=\"there\"",
    "--set",
    "general=\"kenobi\"",
    "new",
    &output_dir.join("with_extras.yaml").to_string_lossy(),
  ])?;

  execute_cli(with_extras).await?;

  let with_extras_output = deserialize_yaml!(Config, output_dir.join("with_extras.yaml"));

  assert_eq!(
    with_extras_output.root_dir,
    Some(PathBuf::from("tests/output"))
  );
  assert_eq!(
    with_extras_output.templates_dir,
    Some(PathBuf::from("tests/templates"))
  );
  assert_eq!(with_extras_output.shell.unwrap(), "zsh");
  assert_eq!(
    with_extras_output
      .global_templates_vars
      .get("hello")
      .unwrap(),
    "there"
  );
  assert_eq!(
    with_extras_output
      .global_templates_vars
      .get("general")
      .unwrap(),
    "kenobi"
  );

  Ok(())
}
