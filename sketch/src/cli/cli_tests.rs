use std::{
  fs::{create_dir_all, read_to_string, remove_dir_all, File},
  path::PathBuf,
  sync::{LazyLock, Once},
};

use clap::Parser;
use pretty_assertions::{assert_eq, assert_ne};
use serde::{Deserialize, Serialize};

use crate::{
  cli::{execute_cli, get_config_from_cli, Cli},
  package_json::{PackageJson, Person},
  pnpm::PnpmWorkspace,
  ts_config::{
    tsconfig_defaults::{
      get_default_dev_tsconfig, get_default_package_tsconfig, get_default_root_tsconfig,
      get_default_src_tsconfig,
    },
    TsConfig, TsConfigReference,
  },
  Config,
};

static TS_TESTS_ROOT: LazyLock<PathBuf> = LazyLock::new(|| PathBuf::from("tests/output/ts_repo"));

static SETUP: Once = Once::new();

fn reset_testing_dir<T: Into<PathBuf>>(dir: T) {
  let dir: PathBuf = dir.into();
  if dir.exists() {
    remove_dir_all(dir.as_path())
      .unwrap_or_else(|e| panic!("Failed to empty the output dir '{}': {}", dir.display(), e));
  }

  create_dir_all(dir.as_path())
    .unwrap_or_else(|e| panic!("Failed to create the output dir '{}': {}", dir.display(), e));
}

macro_rules! assert_dir_exists {
  ($path:expr) => {
    assert!($path.exists() && $path.is_dir())
  };
}

macro_rules! extract_tsconfig {
  ($path:expr) => {
    deserialize_json!(TsConfig, $path)
  };
}

macro_rules! extract_package_json {
  ($path:expr) => {
    deserialize_json!(PackageJson, $path)
  };
}

macro_rules! deserialize_json {
  ($ty:ty, $path:expr) => {{
    let file = File::open(PathBuf::from(&$path))
      .unwrap_or_else(|e| panic!("Failed to open {}: {}", $path.display(), e));
    let data: $ty = serde_json::from_reader(&file)
      .unwrap_or_else(|e| panic!("Failed to deserialize {}: {}", $path.display(), e));
    data
  }};
}

macro_rules! deserialize_yaml {
  ($ty:ty, $path:expr) => {{
    let file = File::open(PathBuf::from(&$path))
      .unwrap_or_else(|e| panic!("Failed to open {}: {}", $path.display(), e));
    let data: $ty = serde_yaml_ng::from_reader(&file)
      .unwrap_or_else(|e| panic!("Failed to deserialize {}: {}", $path.display(), e));
    data
  }};
}

#[tokio::test]
async fn ts_gen() -> Result<(), Box<dyn std::error::Error>> {
  SETUP.call_once(|| reset_testing_dir(TS_TESTS_ROOT.clone()));

  let ts_repo_root = PathBuf::from("tests/output/ts_repo");

  let cli = Cli::try_parse_from([
    "sketch",
    "--root-dir",
    "tests/output/ts_repo",
    "ts",
    "monorepo",
    "--moonrepo",
  ])?;

  execute_cli(cli).await?;

  let root_tsconfig_path = ts_repo_root.join("tsconfig.json");

  assert!(root_tsconfig_path.exists());

  for file in [".oxlintrc.json", "package.json", "pnpm-workspace.yaml"] {
    assert!(ts_repo_root.join(file).exists());
  }

  let options_tsconfig = extract_tsconfig!(ts_repo_root.join("tsconfig.options.json"));

  for dir in [".moon", "packages", "apps", ".out"] {
    assert_dir_exists!(ts_repo_root.join(dir));
  }

  assert_eq!(get_default_root_tsconfig(), options_tsconfig);

  let package_cmd = Cli::try_parse_from([
    "sketch",
    "--root-dir",
    "tests/output/ts_repo",
    "ts",
    "package",
    "--update-root-tsconfig",
    "--preset",
    "test",
  ])?;

  execute_cli(package_cmd).await?;

  let package_dir = ts_repo_root.join("packages/test_package");

  let root_tsconfig = extract_tsconfig!(root_tsconfig_path);

  assert!(root_tsconfig
    .references
    .expect("missing references in root tsconfig")
    .contains(&TsConfigReference {
      path: "packages/test_package/tsconfig.json".to_string()
    }));

  let package_tsconfig = extract_tsconfig!(package_dir.join("tsconfig.json"));
  assert_eq!(
    get_default_package_tsconfig(
      "../../tsconfig.options.json".to_string(),
      "tsconfig.src.json",
      Some("tsconfig.dev.json"),
    ),
    package_tsconfig
  );

  let src_tsconfig = extract_tsconfig!(package_dir.join("tsconfig.src.json"));

  assert_eq!(
    get_default_src_tsconfig(false, "../../.out/test_package"),
    src_tsconfig
  );

  let dev_tsconfig = extract_tsconfig!(package_dir.join("tsconfig.dev.json"));

  assert_eq!(
    get_default_dev_tsconfig("tsconfig.src.json", "../../.out/test_package"),
    dev_tsconfig
  );

  for dir in ["tests", "tests/setup", "src"] {
    let dir = package_dir.join(dir);
    assert!(dir.exists() && dir.is_dir());
  }

  for file in ["src/index.ts", "moon.yml"] {
    assert!(package_dir.join(file).exists());
  }

  let package_json = deserialize_json!(PackageJson, package_dir.join("package.json"));

  assert!(package_json.contributors.iter().any(|p| {
    match p {
      Person::Id(_) => panic!("found person with id in generated package.json"),
      Person::Data(person_data) => person_data.name == "Legolas",
    }
  }));

  assert_eq!(package_json.name, "test_package");

  let app_test = Cli::try_parse_from([
    "sketch",
    "--root-dir",
    "tests/output/ts_repo",
    "ts",
    "package",
    "--app",
    "--update-root-tsconfig",
    "--oxlint",
    "--no-vitest",
    "--moonrepo",
    "-p",
    "app",
    "apps/app_test",
  ])?;

  execute_cli(app_test).await?;

  let app_package_dir = ts_repo_root.join("apps/app_test");

  // Library files and dirs should be missing
  assert!(!app_package_dir.join("tests").exists());
  assert!(!app_package_dir.join("tsconfig.dev.json").exists());

  // The enabled features
  for file in [".oxlintrc.json", "moon.yml"] {
    assert!(app_package_dir.join(file).exists());
  }

  let app_package_json = extract_package_json!(app_package_dir.join("package.json"));

  assert_eq!(app_package_json.name, "app_test");

  // Latest should be converted to version range
  let svelte_dep = app_package_json.dev_dependencies.get("svelte").unwrap();

  assert_ne!(svelte_dep, "latest");

  assert!(svelte_dep.starts_with("^"));

  // New dependency with catalog: should be added to pnpm-workspace
  let pnpm_workspace = deserialize_yaml!(PnpmWorkspace, ts_repo_root.join("pnpm-workspace.yaml"));

  // And it should also have a version range
  assert!(pnpm_workspace
    .catalog
    .get("tailwindcss")
    .unwrap()
    .starts_with("^"));

  // The new named catalog should be added automatically
  assert!(pnpm_workspace
    .catalogs
    .get("svelte")
    .unwrap()
    .get("@sveltejs/kit")
    .unwrap()
    .starts_with("^"));

  Ok(())
}

#[tokio::test]
async fn generated_configs() -> Result<(), Box<dyn std::error::Error>> {
  let output_dir = PathBuf::from("tests/output/generated_configs");

  reset_testing_dir(&output_dir);

  let default_config = Cli::try_parse_from([
    "sketch",
    "--no-config-file",
    "new",
    &output_dir.join("default_config.yaml").to_string_lossy(),
  ])?;

  execute_cli(default_config).await?;

  let default_config_output = deserialize_yaml!(Config, output_dir.join("default_config.yaml"));

  assert_eq!(default_config_output, Config::default());

  let with_extras = Cli::try_parse_from([
    "sketch",
    "--no-config-file",
    "--root-dir",
    "tests/output",
    "--templates-dir",
    "tests/templates",
    "--shell",
    "zsh",
    "--set",
    "hello=\"there\"",
    "--set",
    "general=\"kenobi\"",
    "new",
    &output_dir.join("with_extras.yaml").to_string_lossy(),
  ])?;

  execute_cli(with_extras).await?;

  let with_extras_output = deserialize_yaml!(Config, output_dir.join("with_extras.yaml"));

  assert_eq!(
    with_extras_output.root_dir,
    Some(PathBuf::from("tests/output"))
  );
  assert_eq!(
    with_extras_output.templates_dir,
    Some(PathBuf::from("tests/templates"))
  );
  assert_eq!(with_extras_output.shell.unwrap(), "zsh");
  assert_eq!(
    with_extras_output
      .global_templates_vars
      .get("hello")
      .unwrap(),
    "there"
  );
  assert_eq!(
    with_extras_output
      .global_templates_vars
      .get("general")
      .unwrap(),
    "kenobi"
  );

  Ok(())
}

#[tokio::test]
async fn rendered_commands() -> Result<(), Box<dyn std::error::Error>> {
  let output_dir = PathBuf::from("tests/output/commands_tests");
  let config_file = PathBuf::from("tests/commands_tests/commands_tests.toml");

  reset_testing_dir(&output_dir);

  let literal = Cli::try_parse_from([
    "sketch",
    "--root-dir",
    &output_dir.to_string_lossy(),
    "--set",
    "general=\"kenobi\"",
    "exec",
    "echo \"hello there!\\ngeneral {{ general }}.\" > command_output.txt",
  ])?;

  execute_cli(literal).await?;

  let output: String = read_to_string(output_dir.join("command_output.txt"))?;

  assert_eq!(output, "hello there!\ngeneral kenobi.\n");

  let from_file = Cli::try_parse_from([
    "sketch",
    "--root-dir",
    &output_dir.to_string_lossy(),
    "--set",
    "something=\"space\"",
    "exec",
    "-f",
    "../../commands_tests/cmd_from_file.j2",
  ])?;

  execute_cli(from_file).await?;

  let rendered_from_file: String = read_to_string(output_dir.join("output_from_file.txt"))?;

  assert_eq!(
    rendered_from_file,
    "all the time you have to leave the space!\n"
  );

  let from_file_in_templates_dir = Cli::try_parse_from([
    "sketch",
    "-c",
    &config_file.to_string_lossy(),
    "--set",
    "category=\"gp2\"",
    "exec",
    "-t",
    "cmd_template.j2",
  ])?;

  execute_cli(from_file_in_templates_dir).await?;

  let rendered_from_file_in_templates_dir: String =
    read_to_string(output_dir.join("output_from_templates_dir.txt"))?;

  assert_eq!(
    rendered_from_file_in_templates_dir,
    "gp2 engine... gp2... argh!\n"
  );

  let from_template_id = Cli::try_parse_from([
    "sketch",
    "-c",
    &config_file.to_string_lossy(),
    "--set",
    "condition=\"slower\"",
    "exec",
    "-t",
    "cmd_template",
  ])?;

  execute_cli(from_template_id).await?;

  let rendered_from_file_in_templates_dir: String =
    read_to_string(output_dir.join("output_from_template_id.txt"))?;

  assert_eq!(
    rendered_from_file_in_templates_dir,
    "engine feels good, much slower than before... amazing\n"
  );

  Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct CustomTemplateTest {
  pub my_var: usize,
}

#[tokio::test]
async fn cli_rendering() -> Result<(), Box<dyn std::error::Error>> {
  let output_dir = PathBuf::from("tests/output/custom_templates");

  reset_testing_dir(&output_dir);

  let rendering_cmd = Cli::try_parse_from([
    "sketch",
    "-c",
    "tests/custom_templates/custom_templates.toml",
    "--root-dir",
    "tests/output/custom_templates",
    "render-preset",
    "test",
  ])?;

  let with_cli_override = Cli::try_parse_from([
    "sketch",
    "-c",
    "tests/custom_templates/custom_templates.toml",
    "--root-dir",
    "tests/output/custom_templates",
    "--set",
    "my_var=25",
    "render",
    "--id",
    "lit_template",
    "with_cli_override.yaml",
  ])?;

  execute_cli(rendering_cmd.clone()).await?;
  execute_cli(with_cli_override.clone()).await?;

  let config = get_config_from_cli(rendering_cmd).await?;

  let templates = config.templating_presets.get("test").unwrap();

  for template in templates {
    let output_path = output_dir.join(&template.output);
    let output = deserialize_yaml!(CustomTemplateTest, output_path);

    let output_path_str = output_path.to_string_lossy();

    // Checking local context override
    if output_path_str.ends_with("with_override.yaml") {
      assert_eq!(output.my_var, 20);
      // Checking override from cli
    } else if output_path_str == "with_cli_override.yaml" {
      assert_eq!(output.my_var, 25);
    } else {
      assert_eq!(output.my_var, 15);
    }
  }

  let from_literal = Cli::try_parse_from([
    "sketch",
    "--root-dir",
    "tests/output/custom_templates",
    "--set",
    "location=\"Isengard\"",
    "render",
    "--content",
    "they're taking the hobbits to {{ location }}!",
    "from_literal.txt",
  ])?;

  execute_cli(from_literal).await?;

  let from_literal_output: String = read_to_string(output_dir.join("from_literal.txt"))?;

  assert_eq!(
    from_literal_output,
    "they're taking the hobbits to Isengard!"
  );

  let mut cmd = assert_cmd::Command::cargo_bin("sketch").expect("Failed to find the app binary");

  cmd
    .args([
      "--root-dir",
      "tests/output/custom_templates",
      "--set",
      "location=\"Isengard\"",
      "render",
      "--content",
      "they're taking the hobbits to {{ location }}!",
      "--stdout",
    ])
    .assert()
    .stdout("they're taking the hobbits to Isengard!\n");

  Ok(())
}

#[test]
fn verify_cli() {
  use clap::CommandFactory;
  Cli::command().debug_assert();
}
