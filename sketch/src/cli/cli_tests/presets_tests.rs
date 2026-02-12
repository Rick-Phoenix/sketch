use super::*;

use crate::{
	cli::cli_tests::generated_configs_tests::verify_generated_workflow,
	init_repo::pre_commit::{
		FileType, GITLEAKS_REPO, Language, LocalRepo, PreCommitConfig, PreCommitHook, Repo,
	},
	ts::{
		oxlint::OxlintConfig,
		package_json::PackageJson,
		ts_config::{TsConfig, TsConfigReference},
	},
};

#[tokio::test]
async fn presets() -> Result<(), Box<dyn std::error::Error>> {
	let examples_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../examples");
	let out_dir = PathBuf::from("tests/output/presets");

	reset_testing_dir(&out_dir);

	let git_preset_args = [
		"sketch",
		"--ignore-config",
		"-c",
		path_to_str!(examples_dir.join("presets.yaml")),
		"repo",
		"--preset",
		"ts_package",
		&out_dir.to_string_lossy(),
	];
	Cli::execute_with(git_preset_args).await?;

	get_tree_output(&out_dir, None)?;

	get_clean_example_cmd(&git_preset_args, &[1, 2, 3, 7], &out_dir.join("cmd"))?;

	verify_generated_workflow(&out_dir.join(".github/workflows/my_workflow.yaml"))?;

	let pre_commit_output: PreCommitConfig =
		deserialize_yaml(&out_dir.join(".pre-commit-config.yaml"))?;

	assert_eq!(
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

	let hook_pre_output = read_to_string(out_dir.join("pre.txt"))?;

	assert_eq!(hook_pre_output, "hi\n");

	let hook_post_output = read_to_string(out_dir.join("post.txt"))?;

	assert_eq!(hook_post_output, "hi\n");

	let root_dockerfile_output = read_to_string(out_dir.join("Dockerfile"))?;

	let expected_dockerfile = indoc! {r#"
    FROM node:23-alpine

    COPY . .
    EXPOSE 9530
    CMD ["npm", "run", "dev"]
  "#};

	assert_eq!(root_dockerfile_output, expected_dockerfile);

	let package_out_dir = out_dir.join("packages/presets_example");

	Cli::execute_with([
		"sketch",
		"--ignore-config",
		"-c",
		path_to_str!(examples_dir.join("presets.yaml")),
		"ts",
		"package",
		"--template",
		"dockerfile",
		"--preset",
		"example",
		&package_out_dir.to_string_lossy(),
	])
	.await?;

	get_tree_output(&package_out_dir, None)?;

	let package_dockerfile_output = read_to_string(package_out_dir.join("Dockerfile"))?;
	assert_eq!(package_dockerfile_output, expected_dockerfile);

	let oxlint_result: OxlintConfig = deserialize_json(&package_out_dir.join(".oxlintrc.json"))?;

	assert_eq!(
		oxlint_result.ignore_patterns,
		btreeset! { "**/node_modules/**".to_string(), ".cache".to_string(), ".output".to_string() }
	);

	let package_json_result: PackageJson = deserialize_json(&package_out_dir.join("package.json"))?;

	assert_eq!(
		package_json_result.description.unwrap(),
		"I am the frontend preset"
	);
	assert_eq!(package_json_result.license.unwrap(), "MIT");
	assert_eq!(
		package_json_result.dev_dependencies,
		btreemap! {
		  "svelte".to_string() => "*".to_string(),
		  "tailwindcss".to_string() => "*".to_string(),
		  "vite".to_string() => "*".to_string(),
		}
	);

	assert_eq!(
		package_json_result.scripts,
		btreemap! {
		  "dev".to_string() => "vite dev".to_string(),
		  "build".to_string() => "vite build".to_string(),
		}
	);

	let tsconfig_result: TsConfig = deserialize_json(&package_out_dir.join("tsconfig.json"))?;

	assert_eq!(
		tsconfig_result.references,
		btreeset! {
		  TsConfigReference { path: "/some/path".to_string() },
		  TsConfigReference { path: "/other/path".to_string() },
		}
	);

	assert_eq!(
		tsconfig_result.include,
		btreeset! {
		  "src".to_string(), "tests".to_string(), "scripts".to_string()
		}
	);

	let compiler_options = tsconfig_result.compiler_options.unwrap();

	assert!(!compiler_options.no_emit.unwrap());

	assert!(compiler_options.verbatim_module_syntax.unwrap());

	Ok(())
}
