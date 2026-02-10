use super::*;
use crate::rust::*;

#[tokio::test]
async fn rust_manifest() -> Result<(), Box<dyn std::error::Error>> {
	let output_dir = PathBuf::from("tests/output/rust_tests");
	let commands_dir = output_dir.join("commands");

	reset_testing_dir(&output_dir);
	reset_testing_dir(&commands_dir);

	let config_file = PathBuf::from("tests/cargo_toml_tests/cargo_toml_tests.yaml");

	let output_file = output_dir.join("Cargo.toml");

	let cargo_cmd = [
		"sketch",
		"--ignore-config",
		"-c",
		&config_file.to_string_lossy(),
		"rust",
		"crate",
		"test-workspace",
		&output_file.to_string_lossy(),
	];

	Cli::execute_with(cargo_cmd).await?;

	get_clean_example_cmd(&cargo_cmd, &[1, 2, 3, 6], &commands_dir.join("cargo"))?;

	let output: Manifest = deserialize_toml(&output_file)?;

	let _package = output.package.unwrap();

	Ok(())
}
