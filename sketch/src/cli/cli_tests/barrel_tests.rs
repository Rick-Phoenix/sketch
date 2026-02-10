use super::*;

#[tokio::test]
async fn rendered_commands() -> Result<(), Box<dyn std::error::Error>> {
	let output_dir = PathBuf::from("tests/output/ts_barrel");
	let commands_dir = output_dir.join("commands");

	reset_testing_dir(&output_dir);
	reset_testing_dir(&commands_dir);

	macro_rules! write_command {
		($cmd:expr, $list:expr, $out_file:expr) => {
			get_clean_example_cmd(&$cmd, &$list, &commands_dir.join($out_file))?
		};
	}

	get_tree_output("tests/ts_barrel", Some(output_dir.join("tree")))?;

	let output_file = output_dir.join("index.ts");

	let with_exclude_and_ext_cmd = [
		"sketch",
		"--ignore-config",
		"ts",
		"barrel",
		"--exclude",
		"**/nested2/*",
		"--js-ext",
		"-o",
		&output_file.to_string_lossy(),
		"tests/ts_barrel",
	];

	Cli::execute_with(with_exclude_and_ext_cmd).await?;

	let output = read_to_string(&output_file)?;

	assert_eq!(
		output,
		indoc! {r#"
    export * from "nested/file1.js";
  "#}
	);

	let with_allowed_ext_cmd = [
		"sketch",
		"--ignore-config",
		"ts",
		"barrel",
		"--exclude",
		"**/file1.ts",
		"--keep-ext",
		"ts",
		"-o",
		&output_file.to_string_lossy(),
		"tests/ts_barrel",
	];

	Cli::execute_with(with_allowed_ext_cmd).await?;

	let output = read_to_string(&output_file)?;

	assert_eq!(
		output,
		indoc! {r#"
    export * from "nested/nested2/file2.ts";
  "#}
	);

	let basic_cmd = [
		"sketch",
		"--ignore-config",
		"ts",
		"barrel",
		"-o",
		&output_file.to_string_lossy(),
		"tests/ts_barrel",
	];

	write_command!(basic_cmd, [1, 4, 5, 6], "barrel");

	Cli::execute_with(basic_cmd).await?;

	let output = read_to_string(&output_file)?;

	assert_eq!(
		output,
		indoc! {r#"
    export * from "nested/file1";
    export * from "nested/nested2/file2";
  "#}
	);

	Ok(())
}
