use super::*;

use ::gh_workflow_config::{
	ActionRunner, Event, GhJobPresetRef, Job, RunsOn, Shell, StringNumOrBool, StringOrBool,
	StringOrNum, Workflow,
};

#[tokio::test]
async fn gh_workflow() -> Result<(), Box<dyn std::error::Error>> {
	let output_dir = PathBuf::from("tests/output/generated_configs/gh-workflow");
	let commands_dir = output_dir.join("commands");

	reset_testing_dir(&output_dir);
	reset_testing_dir(&commands_dir);

	let config_file = examples_dir().join("presets.yaml");
	let output_file = output_dir.join("workflow.yaml");

	let gh_workflow_cmd = [
		"sketch",
		"--ignore-config",
		"-c",
		&config_file.to_string_lossy(),
		"gh-workflow",
		"-p",
		"extended",
		&output_file.to_string_lossy(),
	];

	Cli::execute_with(gh_workflow_cmd).await?;

	get_clean_example_cmd(&gh_workflow_cmd, &[1, 2, 3], &commands_dir.join("workflow"))?;

	verify_generated_workflow(&output_file)?;

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
		let mut job = if let GhJobPresetRef::Preset(data) = job
			&& let Job::Normal(content) = data.job
		{
			content
		} else {
			panic!("Unresolved job preset")
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
