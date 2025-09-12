use std::{
  fs::{create_dir_all, remove_dir_all, File},
  path::PathBuf,
  sync::{LazyLock, Once},
};

use clap::Parser;

use crate::{
  cli::{execute_cli, Cli},
  package_json::{PackageJson, Person},
  pnpm::PnpmWorkspace,
  ts_config::{
    tsconfig_defaults::{
      get_default_dev_tsconfig, get_default_package_tsconfig, get_default_root_tsconfig,
      get_default_src_tsconfig,
    },
    TsConfig, TsConfigReference,
  },
};

static OUTPUT_DIR: LazyLock<PathBuf> = LazyLock::new(|| PathBuf::from("tests/output/ts_repo"));

static SETUP: Once = Once::new();

fn run_setup() {
  if OUTPUT_DIR.exists() {
    remove_dir_all(OUTPUT_DIR.as_path()).expect("Failed to empty the output dir");
  }

  create_dir_all(OUTPUT_DIR.as_path()).expect("Failed to create OUTPUT_DIR");
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
      .unwrap_or_else(|_| panic!("Failed to open {}", $path.display()));
    let data: $ty = serde_json::from_reader(&file)
      .unwrap_or_else(|_| panic!("Failed to deserialize {}", $path.display()));
    data
  }};
}

macro_rules! deserialize_yaml {
  ($ty:ty, $path:expr) => {{
    let file = File::open(PathBuf::from(&$path))
      .unwrap_or_else(|_| panic!("Failed to open {}", $path.display()));
    let data: $ty = serde_yaml_ng::from_reader(&file)
      .unwrap_or_else(|_| panic!("Failed to deserialize {}", $path.display()));
    data
  }};
}

#[tokio::test]
async fn cli_root_dir() -> Result<(), Box<dyn std::error::Error>> {
  SETUP.call_once(|| run_setup());

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

#[test]
fn verify_cli() {
  use clap::CommandFactory;
  Cli::command().debug_assert();
}
