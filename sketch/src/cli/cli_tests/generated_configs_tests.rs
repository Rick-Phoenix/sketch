use super::*;

use crate::{
	docker::{ComposeFile, Port, ServiceVolume},
	ts::package_json::PackageJson,
};
use ::gh_workflow::{
	ActionRunner, Event, Job, JobPresetRef, RunsOn, Shell, StringNumOrBool, StringOrBool,
	StringOrNum, Workflow,
};

#[tokio::test]
async fn js_catalog() -> Result<(), Box<dyn std::error::Error>> {
	let output_dir = PathBuf::from("tests/output/bun_package");
	let config_path = PathBuf::from("tests/bun.yaml");

	reset_testing_dir(&output_dir);

	Cli::execute_with([
		"sketch",
		"--ignore-config",
		"-c",
		&config_path.to_string_lossy(),
		"ts",
		"package",
		"--preset",
		"with_catalog",
		&output_dir.to_string_lossy(),
	])
	.await?;

	let target_package_json: PackageJson = deserialize_json(&output_dir.join("package.json"))?;

	#[cfg(feature = "npm-version")]
	{
		assert!(target_package_json.catalog.contains_key("hono"));
		assert!(
			target_package_json
				.catalogs
				.get("svelte")
				.unwrap()
				.contains_key("svelte")
		);
	}

	Ok(())
}

#[tokio::test]
async fn generated_configs() -> Result<(), Box<dyn std::error::Error>> {
	let output_dir = PathBuf::from("tests/output/generated_configs");
	let commands_dir = output_dir.join("commands");

	reset_testing_dir(&output_dir);
	reset_testing_dir(&commands_dir);

	Cli::execute_with([
		"sketch",
		"--ignore-config",
		"new",
		&output_dir
			.join("default_config.yaml")
			.to_string_lossy(),
	])
	.await?;

	let default_config_output: Config = deserialize_yaml(&output_dir.join("default_config.yaml"))?;

	assert_eq!(default_config_output, Config::default());

	let config_file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../examples/presets.yaml");
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

	let output_file = output_dir.join("workflow.yaml");

	let gh_workflow_cmd = [
		"sketch",
		"--ignore-config",
		"-c",
		&config_file.to_string_lossy(),
		"gh-workflow",
		"extended",
		&output_file.to_string_lossy(),
	];

	Cli::execute_with(gh_workflow_cmd).await?;

	get_clean_example_cmd(
		&gh_workflow_cmd,
		&[1, 2, 3, 6],
		&commands_dir.join("workflow"),
	)?;

	verify_generated_workflow(&output_file)?;

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

pub(crate) fn verify_generated_workflow(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
	let output: Workflow = deserialize_yaml(path)?;

	let on = if let Event::Object(on) = output.on.unwrap() {
		on
	} else {
		unreachable!();
	};

	assert!(on.push.unwrap().branches.contains("main"));

	let env = output.env;

	assert_eq!(
		env.get("my_env").unwrap(),
		&StringNumOrBool::String("somevalue".to_string())
	);

	assert_eq!(
		env.get("another_env").unwrap(),
		&StringNumOrBool::String("anothervalue".to_string())
	);

	assert_eq!(output.defaults.unwrap().run.shell.unwrap(), Shell::Bash);

	let jobs = output.jobs;

	assert_eq!(jobs.len(), 2);

	for (name, job) in jobs {
		let mut job = if let JobPresetRef::Data(data) = job
			&& let Job::Normal(content) = data.job
		{
			content
		} else {
			panic!()
		};

		assert_eq!(
			job.runs_on.unwrap(),
			RunsOn::Single(ActionRunner::UbuntuLatest)
		);

		let env = job.env;

		assert_eq!(
			env.get("my_env").unwrap(),
			&StringNumOrBool::String("somevalue".to_string())
		);

		assert_eq!(
			env.get("another_env").unwrap(),
			&StringNumOrBool::String("anothervalue".to_string())
		);

		assert_eq!(job.timeout_minutes.unwrap(), StringOrNum::Num(25));

		let continue_on_error = unwrap_variant!(StringOrBool, Bool, job.continue_on_error.unwrap());
		assert!(!continue_on_error);

		if name == "check_main_branch" {
			assert_eq!(
				env.get("another_other_value").unwrap(),
				&StringNumOrBool::String("yetanothervalue".to_string())
			);

			assert_eq!(
				job.outputs.get("is_on_main").unwrap(),
				"${{ steps.branch_check.outputs.is_on_main }}"
			);

			let first_step = job.steps.remove(0).as_config().unwrap();

			assert_eq!(
				first_step.name.as_ref().unwrap(),
				"Check if a tag is on the main branch"
			);
			assert_eq!(first_step.id.as_ref().unwrap(), "branch_check");
			assert_eq!(
				first_step.run.unwrap().trim(),
				indoc! {
				  r#"
            if git branch -r --contains ${{ github.ref }} | grep -q 'origin/main'; then
            echo "On main branch. Proceeding with the workflow..."
            echo "is_on_main=true" >> "$GITHUB_OUTPUT"
            else
            echo "Not on main branch. Skipping workflow..."
            fi
          "#
				}
				.trim()
			);
		} else {
			assert_eq!(
				job.name.as_ref().unwrap(),
				"Do something while on main branch"
			);

			assert_eq!(
				job.if_.unwrap(),
				"needs.check_branch.outputs.is_on_main == 'true'"
			);

			let first_step = job.steps.remove(0).as_config().unwrap();

			assert_eq!(
				first_step.run.as_ref().unwrap(),
				"echo \"Done something from main branch!\""
			);
		}
	}

	Ok(())
}
