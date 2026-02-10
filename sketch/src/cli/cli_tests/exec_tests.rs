use super::*;

use pretty_assertions::assert_eq;

#[tokio::test]
async fn rendered_commands() -> Result<(), Box<dyn std::error::Error>> {
	let output_dir = PathBuf::from("tests/output/commands_tests");
	let config_file = PathBuf::from("tests/commands_tests/commands_tests.toml");
	let commands_dir = output_dir.join("commands");

	reset_testing_dir(&output_dir);
	reset_testing_dir(&commands_dir);

	macro_rules! write_command {
		($cmd:expr, $list:expr, $out_file:expr) => {
			get_clean_example_cmd(&$cmd, &$list, &commands_dir.join($out_file))?
		};
	}

	let literal_template_cmd = [
		"sketch",
		"--ignore-config",
		"--set",
		"condition=\"slower\"",
		"exec",
		"--cwd",
		&output_dir.to_string_lossy(),
		"echo \"engine feels good... much {{ condition }} than before... amazing\" > command_output.txt",
	];

	write_command!(literal_template_cmd, [1, 4, 5], "exec_literal_cmd");

	Cli::execute_with(literal_template_cmd).await?;

	let output: String = read_to_string(output_dir.join("command_output.txt"))?;

	assert_eq!(
		output,
		"engine feels good... much slower than before... amazing\n"
	);

	let from_file_cmd = [
		"sketch",
		"--ignore-config",
		"--set",
		"something=\"space\"",
		"exec",
		"--cwd",
		&output_dir.to_string_lossy(),
		"-f",
		"tests/commands_tests/cmd_from_file.j2",
	];

	write_command!(from_file_cmd, [1, 4, 5], "cmd_from_file");

	Cli::execute_with(from_file_cmd).await?;

	let rendered_from_file: String = read_to_string(output_dir.join("output_from_file.txt"))?;

	assert_eq!(
		rendered_from_file,
		"all the time you have to leave the space!\n"
	);

	let from_template_cmd = [
		"sketch",
		"--ignore-config",
		"-c",
		&config_file.to_string_lossy(),
		"--set",
		"category=\"gp2\"",
		"exec",
		"--cwd",
		&output_dir.to_string_lossy(),
		"-t",
		"cmd_template.j2",
	];

	write_command!(from_template_cmd, [1, 2, 3, 6, 7], "exec_from_template_cmd");

	Cli::execute_with(from_template_cmd).await?;

	let rendered_from_file_in_templates_dir: String =
		read_to_string(output_dir.join("output_from_templates_dir.txt"))?;

	assert_eq!(
		rendered_from_file_in_templates_dir,
		"gp2 engine... gp2... argh!\n"
	);

	Cli::execute_with([
		"sketch",
		"-c",
		&config_file.to_string_lossy(),
		"--set",
		"condition=\"slower\"",
		"exec",
		"--cwd",
		&output_dir.to_string_lossy(),
		"-t",
		"cmd_template",
	])
	.await?;

	let rendered_from_file_in_templates_dir: String =
		read_to_string(output_dir.join("output_from_template_id.txt"))?;

	assert_eq!(
		rendered_from_file_in_templates_dir,
		"engine feels good, much slower than before... amazing\n"
	);

	Ok(())
}
