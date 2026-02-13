use super::*;

use crate::init_repo::pre_commit::{
	FileType, GITLEAKS_REPO, Language, LocalRepo, PreCommitConfig, PreCommitHook, Repo,
};

#[tokio::test]
async fn repo_preset() -> Result<(), Box<dyn std::error::Error>> {
	let examples_dir = examples_dir();
	let out_dir = PathBuf::from("tests/output/presets/repo");

	reset_testing_dir(&out_dir);

	let git_preset_args = [
		"sketch",
		"--ignore-config",
		"-c",
		path_to_str!(examples_dir.join("presets.yaml")),
		"repo",
		"--preset",
		"ts_package",
		"tests/output/presets/repo",
	];
	Cli::execute_with(git_preset_args).await?;

	get_tree_output(&out_dir, None)?;

	get_clean_example_cmd(&git_preset_args, &[1, 2, 3], &out_dir.join("cmd"))?;

	verify_generated_workflow(&out_dir.join(".github/workflows/my_workflow.yaml"))?;

	let pre_commit_output: PreCommitConfig =
		deserialize_yaml(&out_dir.join(".pre-commit-config.yaml"))?;

	pretty_assert_eq!(
		pre_commit_output,
		PreCommitConfig {
			repos: btreeset! {
			  GITLEAKS_REPO.clone(),
			  Repo::Local { repo: LocalRepo::Local, hooks: btreeset! {
				  PreCommitHook {
					id: "oxlint".to_string(),
					name: Some("oxlint".to_string()),
					entry: Some("oxlint".to_string()),
					language: Some(Language::System),
					files: Some(r"\.svelte$|\.js$|\.ts$".to_string()),
					types: Some(btreeset!{ FileType::File }),
					..Default::default()
				  }
				}
			  }
			},
			..Default::default()
		}
	);

	let gitignore_output = read_to_string(out_dir.join(".gitignore"))?;

	let gitignore_entries: Vec<&str> = gitignore_output.split('\n').collect();

	for entry in ["*.env", "dist", "*.tsBuildInfo", "node_modules"] {
		assert!(gitignore_entries.contains(&entry));
	}

	let root_dockerfile_output = read_to_string(out_dir.join("Dockerfile"))?;

	let expected_dockerfile = indoc! {r#"
    FROM node:23-alpine

    COPY . .
    EXPOSE 9530
    CMD ["npm", "run", "dev"]
  "#};

	pretty_assert_eq!(root_dockerfile_output, expected_dockerfile);

	Ok(())
}
