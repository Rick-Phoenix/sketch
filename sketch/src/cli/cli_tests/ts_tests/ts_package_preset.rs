use super::*;

#[tokio::test]
async fn ts_package_preset() -> Result<(), Box<dyn std::error::Error>> {
	let examples_dir = examples_dir();
	let out_dir = PathBuf::from("tests/output/presets/ts-package");

	reset_testing_dir(&out_dir);

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
		&out_dir.to_string_lossy(),
	])
	.await?;

	get_tree_output(&out_dir, None)?;

	let package_dockerfile_output = read_to_string(out_dir.join("Dockerfile"))?;

	let expected_dockerfile = indoc! {r#"
    FROM node:23-alpine

    COPY . .
    EXPOSE 9530
    CMD ["npm", "run", "dev"]
  "#};
	pretty_assert_eq!(package_dockerfile_output, expected_dockerfile);

	let oxlint_result: OxlintConfig = deserialize_json(&out_dir.join(".oxlintrc.json"))?;

	pretty_assert_eq!(
		oxlint_result.ignore_patterns,
		btreeset! { "**/node_modules/**".to_string(), ".cache".to_string(), ".output".to_string() }
	);

	let package_json_result: PackageJson = deserialize_json(&out_dir.join("package.json"))?;

	pretty_assert_eq!(
		package_json_result.description.unwrap(),
		"I am the frontend preset"
	);
	pretty_assert_eq!(package_json_result.license.unwrap(), "MIT");
	pretty_assert_eq!(
		package_json_result.dev_dependencies,
		btreemap! {
		  "svelte".to_string() => "*".to_string(),
		  "tailwindcss".to_string() => "*".to_string(),
		  "vite".to_string() => "*".to_string(),
		}
	);

	pretty_assert_eq!(
		package_json_result.scripts,
		btreemap! {
		  "dev".to_string() => "vite dev".to_string(),
		  "build".to_string() => "vite build".to_string(),
		}
	);

	let tsconfig_result: TsConfig = deserialize_json(&out_dir.join("tsconfig.json"))?;

	pretty_assert_eq!(
		tsconfig_result.references,
		btreeset! {
		  TsConfigReference { path: "/some/path".to_string() },
		  TsConfigReference { path: "/other/path".to_string() },
		}
	);

	pretty_assert_eq!(
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
