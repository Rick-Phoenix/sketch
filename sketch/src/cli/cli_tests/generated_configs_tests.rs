use std::{fs::File, path::PathBuf};

use clap::Parser;
use pretty_assertions::assert_eq;

use super::reset_testing_dir;
use crate::{
  cli::{cli_tests::get_clean_example_cmd, execute_cli, Cli},
  Config,
};

#[tokio::test]
async fn generated_configs() -> Result<(), Box<dyn std::error::Error>> {
  let output_dir = PathBuf::from("tests/output/generated_configs");

  reset_testing_dir(&output_dir);

  let default_config = Cli::try_parse_from([
    "sketch",
    "--ignore-config-file",
    "new",
    &output_dir.join("default_config.yaml").to_string_lossy(),
  ])?;

  execute_cli(default_config).await?;

  let default_config_output = deserialize_yaml!(Config, output_dir.join("default_config.yaml"));

  assert_eq!(default_config_output, Config::default());

  let with_extras_cmd = [
    "sketch",
    "--ignore-config-file",
    "--out-dir",
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
    path_to_str!(output_dir.join("with_extras.yaml")),
  ];

  get_clean_example_cmd(&with_extras_cmd, 1..4, &output_dir.join("with_extras_cmd"))?;

  let with_extras = Cli::try_parse_from(with_extras_cmd)?;

  execute_cli(with_extras).await?;

  let with_extras_output = deserialize_yaml!(Config, output_dir.join("with_extras.yaml"));

  assert_eq!(
    with_extras_output.out_dir,
    Some(PathBuf::from("tests/output"))
  );
  assert_eq!(
    with_extras_output.templates_dir,
    Some(PathBuf::from("tests/templates"))
  );
  assert_eq!(with_extras_output.shell.unwrap(), "zsh");
  assert_eq!(with_extras_output.vars.get("hello").unwrap(), "there");
  assert_eq!(with_extras_output.vars.get("general").unwrap(), "kenobi");

  Ok(())
}
