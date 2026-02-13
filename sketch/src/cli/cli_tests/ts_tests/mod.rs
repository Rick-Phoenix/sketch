use package_json::{Person, PersonData};

use super::*;

mod ts_barrel_tests;
mod ts_package_preset;

use crate::ts::{
	oxlint::OxlintConfig,
	package_json::PackageJson,
	ts_config::{TsConfig, TsConfigReference},
};

use crate::ts::pnpm::PnpmWorkspace;

#[tokio::test]
async fn monorepo_with_pnpm_catalog() -> Result<(), Box<dyn std::error::Error>> {
	let ts_examples_dir = examples_dir().join("typescript");
	let output_dir = PathBuf::from("tests/output/ts_monorepo");
	let commands_dir = output_dir.join("commands");

	reset_testing_dir(&output_dir);
	reset_testing_dir(&commands_dir);

	let monorepo_cmd = [
		"sketch",
		"--ignore-config",
		"-c",
		path_to_str!(ts_examples_dir.join("root_package.yaml")),
		"ts",
		"monorepo",
		"--root-package",
		"root",
		"--pnpm",
		"base",
		&output_dir.to_string_lossy(),
	];

	get_clean_example_cmd(
		&monorepo_cmd,
		&[1, 2, 3],
		&commands_dir.join("monorepo_cmd"),
	)?;

	Cli::execute_with(monorepo_cmd).await?;

	get_tree_output(&output_dir, None)?;

	let catalog_cmd = [
		"sketch",
		"--ignore-config",
		"-c",
		path_to_str!(ts_examples_dir.join("catalog.yaml")),
		"ts",
		"package",
		"--preset",
		"with_catalog",
		path_to_str!(output_dir.join("packages/with-catalog")),
	];

	get_clean_example_cmd(&catalog_cmd, &[1, 2, 3], &commands_dir.join("catalog_cmd"))?;

	Cli::execute_with(catalog_cmd).await?;

	// Checking if the pnpm-workspace file contains the right config + the updated versions
	let pnpm_file: PnpmWorkspace = deserialize_yaml(&output_dir.join("pnpm-workspace.yaml"))?;

	assert!(
		pnpm_file
			.only_built_dependencies
			.contains("esbuild")
	);
	assert!(pnpm_file.packages.contains("packages/*"));

	#[cfg(feature = "npm-version")]
	{
		assert!(
			pnpm_file
				.catalog
				.get("hono")
				.unwrap()
				.starts_with('^')
		);

		assert!(
			pnpm_file
				.catalogs
				.get("svelte")
				.unwrap()
				.get("svelte")
				.unwrap()
				.starts_with('^')
		);
	}

	assert_eq!(pnpm_file.minimum_release_age.unwrap(), 1440);

	// Check if the workspaces directories were created correctly
	assert!(output_dir.join("packages").is_dir());
	assert!(output_dir.join("apps/test").is_dir());

	Ok(())
}

#[tokio::test]
async fn people_registration() -> Result<(), Box<dyn std::error::Error>> {
	let ts_examples_dir = examples_dir().join("typescript");
	let output_dir = PathBuf::from("tests/output/ts-people-example");
	let commands_dir = output_dir.join("commands");

	reset_testing_dir(&output_dir);
	reset_testing_dir(&commands_dir);

	let people_cmd = [
		"sketch",
		"--ignore-config",
		"-c",
		path_to_str!(ts_examples_dir.join("people.yaml")),
		"ts",
		"package",
		"--preset",
		"people-example",
		&output_dir.to_string_lossy(),
	];

	get_clean_example_cmd(&people_cmd, &[1, 2, 3], &commands_dir.join("people_cmd"))?;

	Cli::execute_with(people_cmd).await?;

	let output: PackageJson = deserialize_json(&output_dir.join("package.json")).unwrap();

	let bruce_wayne = Person::Data(PersonData {
		name: "Bruce Wayne".to_string(),
		email: Some("bruce@gotham.com".to_string()),
		url: Some("brucewayne.com".to_string()),
	});
	let clark_kent = Person::Data(PersonData {
		name: "Clark Kent".to_string(),
		email: Some("clark-kent@dailyplanet.com".to_string()),
		url: Some("clarkkent.com".to_string()),
	});

	pretty_assert_eq!(output.author.unwrap(), bruce_wayne);

	assert!(output.contributors.contains(&bruce_wayne));
	assert!(output.maintainers.contains(&bruce_wayne));
	assert!(output.maintainers.contains(&clark_kent));

	Ok(())
}

#[tokio::test]
async fn example_from_docs() -> Result<(), Box<dyn std::error::Error>> {
	let ts_examples_dir = examples_dir().join("typescript");
	let output_dir = PathBuf::from("tests/output/ts_example");
	let commands_dir = output_dir.join("commands");

	reset_testing_dir(&output_dir);
	reset_testing_dir(&commands_dir);

	let package_gen_cmd = [
		"sketch",
		"--ignore-config",
		"-c",
		path_to_str!(ts_examples_dir.join("new_package.yaml")),
		"ts",
		"package",
		"--preset",
		"frontend",
		path_to_str!(output_dir.join("packages/frontend")),
	];

	get_clean_example_cmd(
		&package_gen_cmd,
		&[1, 2, 3],
		&commands_dir.join("package_gen_cmd"),
	)?;

	Cli::execute_with(package_gen_cmd).await?;

	get_tree_output(output_dir.join("packages/frontend"), None)?;

	Ok(())
}

#[tokio::test]
async fn bun_catalog() -> Result<(), Box<dyn std::error::Error>> {
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
