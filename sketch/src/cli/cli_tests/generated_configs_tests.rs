use std::path::PathBuf;

use clap::Parser;
use pretty_assertions::assert_eq;

use super::reset_testing_dir;
use crate::{
  cli::{cli_tests::get_clean_example_cmd, execute_cli, Cli},
  docker::compose::{ComposeFile, ServiceVolume},
  fs::deserialize_yaml,
  Config,
};

#[tokio::test]
async fn generated_configs() -> Result<(), Box<dyn std::error::Error>> {
  let output_dir = PathBuf::from("tests/output/generated_configs");
  let commands_dir = output_dir.join("commands");

  reset_testing_dir(&output_dir);
  reset_testing_dir(&commands_dir);

  let default_config = Cli::try_parse_from([
    "sketch",
    "--ignore-config",
    "new",
    &output_dir.join("default_config.yaml").to_string_lossy(),
  ])?;

  execute_cli(default_config).await?;

  let default_config_output: Config = deserialize_yaml(&output_dir.join("default_config.yaml"))?;

  assert_eq!(default_config_output, Config::default());

  let config_file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../examples/presets.yaml");
  let output_file = output_dir.join("compose.yaml");

  let compose_file_cmd = [
    "sketch",
    "-c",
    &config_file.to_string_lossy(),
    "docker-compose",
    "extended",
    &output_file.to_string_lossy(),
  ];

  let compose_file = Cli::try_parse_from(compose_file_cmd)?;

  get_clean_example_cmd(&compose_file_cmd, &[1, 2, 5], &commands_dir.join("compose"))?;

  execute_cli(compose_file).await?;

  let output: ComposeFile = deserialize_yaml(&output_file)?;

  let services = output.services.unwrap();

  let service = services.get("my_service").unwrap();

  assert!(service.networks.as_ref().unwrap().contains("my_network"));
  assert!(service
    .volumes
    .as_ref()
    .unwrap()
    .contains(&ServiceVolume::Simple("my_volume:/target".to_string())));

  let networks = output.networks.unwrap();
  let my_network = networks.get("my_network").unwrap();

  assert!(my_network.external.unwrap());

  let volumes = output.volumes.unwrap();
  let my_volume = volumes.get("my_volume").unwrap();

  assert!(my_volume.external.unwrap());

  let my_other_volume = volumes.get("my_other_volume").unwrap();

  assert!(my_other_volume.external.unwrap());

  Ok(())
}
