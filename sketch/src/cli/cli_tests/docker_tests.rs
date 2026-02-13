use super::*;

use crate::docker::{ComposeFile, Port, ServiceVolume};

#[tokio::test]
async fn compose_file() -> Result<(), Box<dyn std::error::Error>> {
	let output_dir = PathBuf::from("tests/output/generated_configs/compose-file");
	let commands_dir = output_dir.join("commands");

	reset_testing_dir(&output_dir);
	reset_testing_dir(&commands_dir);

	let config_file = examples_dir().join("presets.yaml");
	let output_file = output_dir.join("compose.yaml");

	let compose_file_cmd = [
		"sketch",
		"--ignore-config",
		"-c",
		&config_file.to_string_lossy(),
		"docker-compose",
		"--service",
		"caddy",
		"extended",
		&output_file.to_string_lossy(),
	];

	get_clean_example_cmd(
		&compose_file_cmd,
		&[1, 2, 3, 8],
		&commands_dir.join("compose"),
	)?;

	Cli::execute_with(compose_file_cmd).await?;

	let output: ComposeFile = deserialize_yaml(&output_file)?;

	let mut services = output.services;

	let caddy_service = services
		.remove("caddy")
		.unwrap()
		.as_config()
		.unwrap();

	assert!(
		caddy_service
			.networks
			.as_ref()
			.unwrap()
			.contains("my_network")
	);
	assert!(
		caddy_service
			.ports
			.contains(&Port::String("80:80".to_string()))
	);
	assert!(
		caddy_service
			.ports
			.contains(&Port::String("443:443".to_string()))
	);

	let service = services
		.remove("my_service")
		.unwrap()
		.as_config()
		.unwrap();

	assert!(
		service
			.networks
			.as_ref()
			.unwrap()
			.contains("my_network")
	);
	assert!(
		service
			.volumes
			.contains(&ServiceVolume::Simple("my_volume:/target".to_string()))
	);

	let db_service = services
		.remove("db")
		.unwrap()
		.as_config()
		.unwrap();

	assert!(db_service.image.unwrap() == "postgres");
	assert!(
		db_service
			.networks
			.as_ref()
			.unwrap()
			.contains("my_network")
	);

	assert_eq!(db_service.environment.get("TZ").unwrap(), "Europe/Berlin");

	let networks = output.networks;
	let my_network = networks.get("my_network").unwrap();

	assert!(my_network.external.unwrap());

	let volumes = output.volumes;
	let my_volume = volumes.get("my_volume").unwrap();

	assert!(my_volume.external.unwrap());

	let my_other_volume = volumes.get("my_other_volume").unwrap();

	assert!(my_other_volume.external.unwrap());

	Ok(())
}
