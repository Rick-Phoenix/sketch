use super::*;
use crate::rust::*;

#[tokio::test]
async fn docs_example() -> Result<(), Box<dyn std::error::Error>> {
	let output_dir = PathBuf::from("tests/output/rust_example");

	reset_testing_dir(&output_dir);

	let config_file = examples_dir().join("presets.yaml");
	let output_file = output_dir.join("cargo-example.toml");

	let cmd = [
		"sketch",
		"--ignore-config",
		"-c",
		&config_file.to_string_lossy(),
		"rust",
		"manifest",
		"cli-custom",
		&output_file.to_string_lossy(),
	];

	Cli::execute_with(cmd).await?;

	get_clean_example_cmd(&cmd, &[1, 2, 3], &output_dir.join("rust-example-cmd"))?;

	Ok(())
}

#[tokio::test]
async fn rust_workspace() -> Result<(), Box<dyn std::error::Error>> {
	let output_dir = PathBuf::from("tests/output/rust_tests/workspace-single");

	reset_testing_dir(&output_dir);

	let config_file = PathBuf::from("tests/cargo_toml_tests/cargo_toml_tests.yaml");

	let cargo_cmd = [
		"sketch",
		"--ignore-config",
		"-c",
		&config_file.to_string_lossy(),
		"rust",
		"crate",
		"-p",
		"test-workspace",
		&output_dir.to_string_lossy(),
	];

	Cli::execute_with(cargo_cmd).await?;

	let output: Manifest = deserialize_toml(&output_dir.join("Cargo.toml")).unwrap();

	let workspace = output.workspace.unwrap();

	let string_list_example = BTreeSet::from_iter(
		["they", "are", "taking", "the", "hobbits", "to", "Isengard!"]
			.into_iter()
			.map(str::to_string),
	);

	let package = workspace.package.unwrap();

	pretty_assert_eq!(&package.categories, &string_list_example);
	pretty_assert_eq!(package.edition.unwrap(), Edition::E2024);
	pretty_assert_eq!(package.description.unwrap(), "description");
	pretty_assert_eq!(package.documentation.unwrap(), "abc.com");
	pretty_assert_eq!(package.homepage.unwrap(), "abc.com");
	pretty_assert_eq!(package.license.unwrap(), "Apache-2.0");
	pretty_assert_eq!(package.publish.unwrap(), Publish::Flag(false));
	pretty_assert_eq!(package.readme.unwrap(), OptionalFile::Flag(true));
	pretty_assert_eq!(package.repository.unwrap(), "abc");
	pretty_assert_eq!(package.rust_version.unwrap(), "1.82");
	pretty_assert_eq!(package.version.unwrap(), "0.1.0");
	pretty_assert_eq!(&package.keywords, &string_list_example);
	pretty_assert_eq!(&package.include, &string_list_example);
	pretty_assert_eq!(&package.exclude, &string_list_example);

	let release_metadata = workspace
		.metadata
		.get("release")
		.unwrap()
		.as_object()
		.unwrap();

	assert!(
		release_metadata
			.get("tags")
			.unwrap()
			.as_bool()
			.unwrap()
	);

	assert!(
		release_metadata
			.get("shared-version")
			.unwrap()
			.as_bool()
			.unwrap()
	);

	pretty_assert_eq!(workspace.resolver.unwrap(), Resolver::V3);

	let shorter_list = BTreeSet::from_iter(["a".to_string(), "b".to_string()]);

	pretty_assert_eq!(&workspace.members, &shorter_list);
	pretty_assert_eq!(&workspace.default_members, &shorter_list);

	let lints = workspace.lints.unwrap();

	pretty_assert_eq!(
		lints.clippy.get("useless_let_if_seq").unwrap(),
		&LintKind::Simple(LintLevel::Allow)
	);

	pretty_assert_eq!(
		lints.clippy.get("abc").unwrap(),
		&LintKind::Detailed(Lint {
			level: LintLevel::Allow,
			priority: Some(0)
		})
	);

	pretty_assert_eq!(
		lints.rust.get("abc").unwrap(),
		&LintKind::Simple(LintLevel::Allow)
	);

	let serde_dep = workspace.dependencies.get("serde").unwrap();

	pretty_assert_eq!(
		serde_dep.as_detailed().unwrap(),
		&DependencyDetail {
			version: Some("1".to_string()),
			default_features: Some(false),
			features: BTreeSet::from_iter(["derive".to_string(), "std".to_string()]),
			..Default::default()
		}
	);

	pretty_assert_eq!(
		workspace.dependencies.get("clap").unwrap(),
		&Dependency::Simple("4.5".to_string())
	);

	Ok(())
}

#[tokio::test]
async fn rust_package() -> Result<(), Box<dyn std::error::Error>> {
	let output_dir = PathBuf::from("tests/output/rust_tests/workspace");

	reset_testing_dir(&output_dir);

	let config_file = PathBuf::from("tests/cargo_toml_tests/cargo_toml_tests.yaml");

	let workspace_manifest_output = output_dir.join("Cargo.toml");

	// First we place the workspace manifest
	Cli::execute_with([
		"sketch",
		"--ignore-config",
		"-c",
		&config_file.to_string_lossy(),
		"rust",
		"manifest",
		"test-workspace",
		&workspace_manifest_output.to_string_lossy(),
	])
	.await?;

	let package_dir = output_dir.join("test-package");

	let package_gen_cmd = [
		"sketch",
		"--ignore-config",
		"-c",
		&config_file.to_string_lossy(),
		"rust",
		"crate",
		"-p",
		"test-package",
		&package_dir.to_string_lossy(),
	];

	Cli::execute_with(package_gen_cmd).await?;

	let output: Manifest = deserialize_toml(&package_dir.join("Cargo.toml")).unwrap();

	let package = output.package.unwrap();

	let string_list_example = BTreeSet::from_iter(
		["they", "are", "taking", "the", "hobbits", "to", "Isengard!"]
			.into_iter()
			.map(str::to_string),
	);

	pretty_assert_eq!(
		output.features.get("default").unwrap(),
		&BTreeSet::default()
	);
	pretty_assert_eq!(output.features.get("abc").unwrap(), &string_list_example);

	pretty_assert_eq!(
		output
			.lints
			.unwrap()
			.as_value()
			.unwrap()
			.clippy
			.get("useless_let_if_seq")
			.unwrap(),
		&LintKind::Simple(LintLevel::Allow)
	);

	let expected_serde_dep_output = DependencyDetail {
		version: Some("1".to_string()),
		default_features: Some(false),
		features: BTreeSet::from_iter(["derive".to_string(), "std".to_string()]),
		..Default::default()
	};

	pretty_assert_eq!(
		output
			.dependencies
			.get("serde")
			.unwrap()
			.as_detailed()
			.unwrap(),
		&expected_serde_dep_output
	);

	assert!(
		output
			.dependencies
			.get("clap")
			.unwrap()
			.as_inherited()
			.unwrap()
			.workspace,
	);

	pretty_assert_eq!(
		output
			.build_dependencies
			.get("serde")
			.unwrap()
			.as_detailed()
			.unwrap(),
		&expected_serde_dep_output
	);
	assert!(
		output
			.build_dependencies
			.get("clap")
			.unwrap()
			.as_inherited()
			.unwrap()
			.workspace,
	);

	pretty_assert_eq!(
		output
			.dev_dependencies
			.get("serde")
			.unwrap()
			.as_detailed()
			.unwrap(),
		&expected_serde_dep_output
	);
	assert!(
		output
			.dev_dependencies
			.get("clap")
			.unwrap()
			.as_inherited()
			.unwrap()
			.workspace,
	);

	assert!(output.lib.unwrap().proc_macro);

	pretty_assert_eq!(
		output.target.get("cfg(windows)").unwrap(),
		&Target {
			dependencies: btreemap! { "winhttp".to_string() => Dependency::Simple("0.1.0".to_string()) },
			..Default::default()
		}
	);

	pretty_assert_eq!(
		output.target.get("cfg(unix)").unwrap(),
		&Target {
			dependencies: btreemap! { "openssl".to_string() => Dependency::Simple("0.1.0".to_string()) },
			..Default::default()
		}
	);

	let dummy_product = Product {
		name: Some("abc".to_string()),
		path: Some("abc".to_string()),
		..Default::default()
	};

	let dummy_product2 = Product {
		name: Some("abcde".to_string()),
		path: Some("abcde".to_string()),
		..Default::default()
	};

	assert!(output.bin.contains(&dummy_product));
	assert!(output.bin.contains(&dummy_product2));
	assert!(output.test.contains(&dummy_product));
	assert!(output.test.contains(&dummy_product2));
	assert!(output.bench.contains(&dummy_product));
	assert!(output.bench.contains(&dummy_product2));
	assert!(output.example.contains(&dummy_product));
	assert!(output.example.contains(&dummy_product2));

	let dummy_profile = Profile {
		opt_level: Some(OptLevel::Zero),
		lto: Some(LtoSetting::Fat),
		debug: Some(DebugSetting::None),
		package: btreemap! { "*".to_string() => Profile {
			debug: Some(DebugSetting::None),
			opt_level: Some(OptLevel::One),
			..Default::default()
		} },
		..Default::default()
	};

	let profiles = output.profile.unwrap();

	pretty_assert_eq!(&profiles.dev.unwrap(), &dummy_profile);
	pretty_assert_eq!(&profiles.release.unwrap(), &dummy_profile);

	pretty_assert_eq!(
		output
			.patch
			.get("crates-io")
			.unwrap()
			.get("serde")
			.unwrap()
			.as_detailed()
			.unwrap(),
		&expected_serde_dep_output
	);

	pretty_assert_eq!(package.name.unwrap(), "test-package");

	let release_metadata = package
		.metadata
		.get("release")
		.unwrap()
		.as_object()
		.unwrap();

	assert!(
		release_metadata
			.get("tags")
			.unwrap()
			.as_bool()
			.unwrap()
	);

	pretty_assert_eq!(
		release_metadata
			.get("prefix")
			.unwrap()
			.as_str()
			.unwrap(),
		"0.1"
	);

	assert!(!package.autobenches.unwrap());
	assert!(!package.autobins.unwrap());
	assert!(!package.autoexamples.unwrap());
	assert!(!package.autolib.unwrap());
	assert!(!package.autotests.unwrap());

	pretty_assert_eq!(package.default_run.unwrap(), "a");
	pretty_assert_eq!(
		package.build.unwrap(),
		OptionalFile::Path("build.rs".into())
	);
	pretty_assert_eq!(
		package.license_file.unwrap(),
		Inheritable::Value(PathBuf::from("abc"))
	);
	pretty_assert_eq!(package.links.unwrap(), "abc");
	pretty_assert_eq!(package.resolver.unwrap(), Resolver::V3);

	pretty_assert_eq!(package.categories.as_value().unwrap(), &string_list_example);

	pretty_assert_eq!(
		package.edition.unwrap().as_value().unwrap(),
		&Edition::E2024
	);
	pretty_assert_eq!(
		package.description.unwrap().as_value().unwrap(),
		"description"
	);
	pretty_assert_eq!(
		package.documentation.unwrap().as_value().unwrap(),
		"abc.com"
	);
	pretty_assert_eq!(package.homepage.unwrap().as_value().unwrap(), "abc.com");
	pretty_assert_eq!(package.license.unwrap().as_value().unwrap(), "Apache-2.0");
	pretty_assert_eq!(
		package.publish.unwrap().as_value().unwrap(),
		&Publish::Flag(false)
	);
	pretty_assert_eq!(
		package.readme.unwrap().as_value().unwrap(),
		&OptionalFile::Flag(true)
	);
	pretty_assert_eq!(package.repository.unwrap().as_value().unwrap(), "abc");
	pretty_assert_eq!(package.rust_version.unwrap().as_value().unwrap(), "1.82");
	pretty_assert_eq!(package.version.unwrap().as_value().unwrap(), "0.1.0");
	pretty_assert_eq!(package.keywords.as_value().unwrap(), &string_list_example);
	pretty_assert_eq!(package.include.as_value().unwrap(), &string_list_example);
	pretty_assert_eq!(package.exclude.as_value().unwrap(), &string_list_example);

	// Gitignore generation
	let gitignore = read_to_string(package_dir.join(".gitignore")).unwrap();

	pretty_assert_eq!(gitignore, "target");

	// License generation
	let license = read_to_string(package_dir.join("LICENSE")).unwrap();

	pretty_assert_eq!(license, License::Mpl2.get_content());

	// Usage of templating presets
	let lib_rs_file = read_to_string(package_dir.join("src/lib.rs")).unwrap();

	pretty_assert_eq!(lib_rs_file, "use clap::Parser;\n");

	let second_pkg_dir = output_dir.join("simple_test");

	let package_gen_cmd = [
		"sketch",
		"--ignore-config",
		"-c",
		&config_file.to_string_lossy(),
		"rust",
		"crate",
		"-p",
		"simple_test",
		&second_pkg_dir.to_string_lossy(),
	];

	Cli::execute_with(package_gen_cmd).await?;

	let second_pkg_manifest: Manifest =
		deserialize_toml(&second_pkg_dir.join("Cargo.toml")).unwrap();

	let second_pkg = second_pkg_manifest.package.unwrap();

	// Checking that workspace inheritance is added automatically, if the fields are unset
	assert!(
		second_pkg_manifest
			.lints
			.is_some_and(|l| l.is_workspace())
	);

	macro_rules! assert_is_workspace {
		($($names:ident),*) => {
			$(
				assert!(
					second_pkg
						.$names
						.is_some_and(|v| v.is_workspace())
				);
			)*
		};
	}

	assert_is_workspace!(
		edition,
		license,
		homepage,
		rust_version,
		description,
		readme,
		documentation,
		publish,
		version,
		repository
	);

	assert!(second_pkg.keywords.is_workspace());
	assert!(second_pkg.categories.is_workspace());
	assert!(second_pkg.exclude.is_workspace());
	assert!(second_pkg.include.is_workspace());

	let workspace_manifest: Manifest = deserialize_toml(&workspace_manifest_output).unwrap();

	let workspace = workspace_manifest.workspace.unwrap();

	// Checking that new members are added like `cargo new` does
	assert!(workspace.members.contains("simple_test"));
	assert!(workspace.members.contains("test-package"));

	Ok(())
}
