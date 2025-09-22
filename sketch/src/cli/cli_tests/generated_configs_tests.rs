use std::path::PathBuf;

use clap::Parser;
use pretty_assertions::assert_eq;

use super::reset_testing_dir;
use crate::{
  cli::{execute_cli, Cli},
  fs::deserialize_yaml,
  Config,
};

#[tokio::test]
async fn generated_configs() -> Result<(), Box<dyn std::error::Error>> {
  let output_dir = PathBuf::from("tests/output/generated_configs");

  reset_testing_dir(&output_dir);

  let default_config = Cli::try_parse_from([
    "sketch",
    "--ignore-config",
    "new",
    &output_dir.join("default_config.yaml").to_string_lossy(),
  ])?;

  execute_cli(default_config).await?;

  let default_config_output: Config = deserialize_yaml(&output_dir.join("default_config.yaml"))?;

  assert_eq!(default_config_output, Config::default());

  Ok(())
}
