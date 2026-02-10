use super::*;

#[tokio::test]
async fn overwrite_test() -> Result<(), Box<dyn std::error::Error>> {
	let output_dir = PathBuf::from("tests/output/overwrite_test");

	reset_testing_dir(&output_dir);

	let output_file = output_dir.join("overwrite_test.txt");

	Cli::execute_with([
		"sketch",
		"--ignore-config",
		"render",
		"--content",
		"they're taking the hobbits to Isengard!",
		&output_file.to_string_lossy(),
	])
	.await?;

	let mut cmd = get_bin!();

	// Ensuring the second write fails
	cmd.args([
		"--ignore-config",
		"--no-overwrite",
		"render",
		"--content",
		"they're taking the hobbits to Isengard!",
		&output_file.to_string_lossy(),
	])
	.assert()
	.failure();

	Ok(())
}
