use super::*;

#[tokio::test]
async fn tera_filters() -> Result<(), Box<dyn std::error::Error>> {
	let examples_dir = examples_dir();
	let output_dir = PathBuf::from("tests/output/templating_examples");

	reset_testing_dir(&output_dir);

	let mut bin = get_bin!();

	let args = [
		"--ignore-config",
		"--templates-dir",
		path_to_str!(examples_dir.join("templating/templates")),
		"render",
		"--template",
		"example.j2",
	];

	let var_name = "GREETING";
	let var_value = "hello,world";

	let cmd_str = format!(
		"{}=\"{}\" sketch {}",
		var_name,
		var_value,
		args.split_at(3).1.join(" ")
	);

	let output = bin.env(var_name, var_value).args(args).output()?;

	let output_str = std::str::from_utf8(&output.stdout)?.trim();

	if output_str.is_empty() {
		panic!(
			"Error in the template output: {}",
			std::str::from_utf8(&output.stderr)?
		);
	}

	assert!(output_str.contains("Current arch is: x86_64"));
	assert!(output_str.contains("Current os is: linux"));
	assert!(output_str.contains("Current os family is: unix"));
	assert!(output_str.contains("Is unix: true"));
	assert!(output_str.contains("It's a dir!"));
	assert!(output_str.contains("It's a file!"));
	assert!(output_str.contains("First segment is: hello"));
	assert!(output_str.contains("Second segment is: world"));
	assert!(output_str.contains("Basename is: myfile"));
	assert!(output_str.contains("Parent dir is: mydir"));
	assert!(output_str.contains("Path is: Cargo"));
	assert!(output_str.contains("Extension is: toml"));
	assert!(output_str.contains("Matches glob: true"));
	assert!(output_str.contains("They're taking the hobbits to Isengard!"));
	assert!(output_str.contains("Major is: 0"));
	assert!(output_str.contains("Minor is: 1"));
	assert!(output_str.contains("Patch is: 0"));
	assert!(output_str.contains("Version matches >=0.1.0: true"));
	assert!(output_str.contains("Version matches >=0.2.0: false"));
	assert!(output_str.contains("To camelCase: myVar"));
	assert!(output_str.contains("To snake_case: my_var"));
	assert!(output_str.contains("To SCREAMING_CASE: MY_VAR"));
	assert!(output_str.contains("To PascalCase: MyVar"));
	assert!(output_str.contains("Luke, I am your father!"));
	assert!(output_str.contains("Entry: example.j2"));
	assert!(output_str.contains("In yaml form:\npath: Cargo\nextension: toml"));
	assert!(output_str.contains("In toml form:\npath = \"Cargo\"\nextension = \"toml\""));

	write_file(&output_dir.join("cmd"), &cmd_str, true)?;
	write_file(&output_dir.join("output"), output_str, true)?;

	Ok(())
}

#[tokio::test]
async fn single_templates() -> Result<(), Box<dyn std::error::Error>> {
	let output_dir = PathBuf::from("tests/output/single_templates");
	let commands_dir = output_dir.join("commands");

	let config_dir = PathBuf::from("../examples/templating");
	let config_file = config_dir.join("templating.yaml");

	macro_rules! write_command {
		($args:expr, $list:expr, $out_file:expr) => {
			get_clean_example_cmd(&$args, &$list, &commands_dir.join($out_file))?
		};
	}

	macro_rules! exists {
		($name:literal) => {
			assert!(output_dir.join($name).is_file())
		};
	}

	reset_testing_dir(&output_dir);
	reset_testing_dir(&commands_dir);

	get_tree_output(config_dir.join("templates"), Some(output_dir.join("tree")))?;

	// From known template

	let from_template_id_cmd = [
		"sketch",
		"--ignore-config",
		"-c",
		&config_file.to_string_lossy(),
		"render",
		"--template",
		"hobbits",
		"tests/output/single_templates/from_template_id.txt",
	];

	write_command!(from_template_id_cmd, [1, 2, 3], "from_id");

	Cli::execute_with(from_template_id_cmd).await?;

	exists!("from_template_id.txt");

	// From any file

	let from_template_file_cmd = [
		"sketch",
		"--ignore-config",
		"-c",
		&config_file.to_string_lossy(),
		"render",
		"--template",
		"subdir/nested_file.j2",
		"tests/output/single_templates/from_template_file.txt",
	];

	write_command!(from_template_file_cmd, [1, 2, 3], "from_template_file");

	Cli::execute_with(from_template_file_cmd).await?;

	exists!("from_template_file.txt");

	let literal_template_cmd = [
		"sketch",
		"--ignore-config",
		"-c",
		&config_file.to_string_lossy(),
		"render",
		"--content",
		"they're taking the hobbits to {{ location }}!",
		"tests/output/single_templates/from_literal.txt",
	];

	write_command!(literal_template_cmd, [1, 2, 3], "literal_template_cmd");

	Cli::execute_with(literal_template_cmd).await?;

	let from_literal_output: String = read_to_string(output_dir.join("from_literal.txt"))?;

	assert_eq!(
		from_literal_output,
		"they're taking the hobbits to Isengard!"
	);

	let mut cmd = get_bin!();

	cmd.args([
		"--ignore-config",
		"--set",
		"location=\"Isengard\"",
		"render",
		"--content",
		"they're taking the hobbits to {{ location }}!",
	])
	.assert()
	.stdout("they're taking the hobbits to Isengard!\n");

	Ok(())
}

#[tokio::test]
async fn remote_preset() -> Result<(), Box<dyn std::error::Error>> {
	let output_dir = PathBuf::from("tests/output/templating_presets/remote");
	let commands_dir = output_dir.join("commands");

	let config_file = PathBuf::from("../examples/templating/templating.yaml");

	macro_rules! write_command {
		($args:expr, $list:expr, $out_file:expr) => {
			get_clean_example_cmd(&$args, &$list, &commands_dir.join($out_file))?
		};
	}

	reset_testing_dir(&output_dir);
	reset_testing_dir(&commands_dir);

	let from_remote_preset_cmd = [
		"sketch",
		"--ignore-config",
		"-c",
		&config_file.to_string_lossy(),
		"--set",
		"continuation=\"gp2 engine... gp2!\"",
		"render",
		"--preset",
		"remote",
		"tests/output/templating_presets/remote",
	];

	Cli::execute_with(from_remote_preset_cmd).await?;

	write_command!(from_remote_preset_cmd, [1, 2, 3], "remote");
	get_tree_output("tests/output/templating_presets/remote", None)?;

	let expected_output = "Roses are red, violets are blue, gp2 engine... gp2!\n";

	let top_level_file = read_to_string(output_dir.join("some_file"))?;

	assert_eq!(top_level_file, expected_output);

	let nested_file = read_to_string(output_dir.join("subdir/nested/nested_file"))?;

	assert_eq!(nested_file, expected_output);

	Ok(())
}

#[tokio::test]
async fn simple_templating_preset() -> Result<(), Box<dyn std::error::Error>> {
	let output_dir = PathBuf::from("tests/output/templating_presets/simple");
	let commands_dir = output_dir.join("commands");

	let config_file = PathBuf::from("../examples/templating/templating.yaml");

	macro_rules! write_command {
		($args:expr, $list:expr, $out_file:expr) => {
			get_clean_example_cmd(&$args, &$list, &commands_dir.join($out_file))?
		};
	}

	reset_testing_dir(&output_dir);
	reset_testing_dir(&commands_dir);

	let collection_preset = [
		"sketch",
		"--ignore-config",
		"-c",
		&config_file.to_string_lossy(),
		"render",
		"--preset",
		"lotr",
		"tests/output/templating_presets/simple",
	];

	write_command!(collection_preset, [1, 2, 3], "collection_preset");

	Cli::execute_with(collection_preset).await?;

	assert!(output_dir.join("hobbits.txt").is_file());
	assert!(output_dir.join("subdir/breakfast.txt").is_file());

	get_tree_output(&output_dir, None)?;

	Ok(())
}

#[tokio::test]
async fn structured_presets() -> Result<(), Box<dyn std::error::Error>> {
	let output_dir = PathBuf::from("tests/output/templating_presets/structured");
	let commands_dir = output_dir.join("commands");

	let config_file = PathBuf::from("../examples/templating/templating.yaml");

	macro_rules! write_command {
		($args:expr, $list:expr, $out_file:expr) => {
			get_clean_example_cmd(&$args, &$list, &commands_dir.join($out_file))?
		};
	}

	reset_testing_dir(&output_dir);
	reset_testing_dir(&commands_dir);

	// Structured

	let structured_preset = [
		"sketch",
		"--ignore-config",
		"-c",
		&config_file.to_string_lossy(),
		"render",
		"--preset",
		"structured",
		"tests/output/templating_presets/structured",
	];

	write_command!(structured_preset, [1, 2, 3], "structured_preset");

	Cli::execute_with(structured_preset).await?;

	assert!(output_dir.join("nested_file").is_file());
	assert!(
		output_dir
			.join("nested/more_nested_file")
			.is_file()
	);

	get_tree_output(&output_dir, None)?;

	Ok(())
}

#[tokio::test]
async fn extended_templating_preset() -> Result<(), Box<dyn std::error::Error>> {
	let output_dir = PathBuf::from("tests/output/templating_presets/extended");

	let config_file = PathBuf::from("../examples/templating/templating.yaml");

	reset_testing_dir(&output_dir);

	// Extended

	let extended_preset = [
		"sketch",
		"--ignore-config",
		"-c",
		&config_file.to_string_lossy(),
		"render",
		"--preset",
		"lotr",
		"tests/output/templating_presets/extended",
	];

	Cli::execute_with(extended_preset).await?;

	assert!(output_dir.join("hobbits.txt").is_file());
	assert!(output_dir.join("subdir/breakfast.txt").is_file());

	Ok(())
}
