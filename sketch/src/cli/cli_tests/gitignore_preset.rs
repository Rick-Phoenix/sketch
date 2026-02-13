use super::*;

#[tokio::test]
async fn gitignore() -> Result<(), Box<dyn std::error::Error>> {
	let output_dir = PathBuf::from("tests/output/generated_configs/gitignore");

	reset_testing_dir(&output_dir);

	let config_file = examples_dir().join("presets.yaml");
	let gitignore_path = output_dir.join(".gitignore");

	let gitignore_cmd = [
		"sketch",
		"--ignore-config",
		"-c",
		&config_file.to_string_lossy(),
		"gitignore",
		"ts",
		&gitignore_path.to_string_lossy(),
	];

	Cli::execute_with(gitignore_cmd).await?;

	let gitignore_output = read_to_string(&gitignore_path)?;

	let gitignore_entries: Vec<&str> = gitignore_output.split('\n').collect();

	for entry in ["*.env", "dist", "*.tsBuildInfo", "node_modules"] {
		assert!(gitignore_entries.contains(&entry));
	}

	Ok(())
}
