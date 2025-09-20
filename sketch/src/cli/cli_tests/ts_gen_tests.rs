use std::{fs::File, path::PathBuf};

use clap::Parser;
use pretty_assertions::{assert_eq, assert_ne};

use super::reset_testing_dir;
use crate::{
  cli::{execute_cli, Cli},
  ts::{
    package_json::PackageJson,
    pnpm::PnpmWorkspace,
    ts_config::{
      tsconfig_defaults::{
        get_default_dev_tsconfig, get_default_package_tsconfig, get_default_root_tsconfig,
        get_default_src_tsconfig,
      },
      TsConfig, TsConfigReference,
    },
  },
};

#[tokio::test]
async fn ts_gen() -> Result<(), Box<dyn std::error::Error>> {
  let ts_repo_root = PathBuf::from("tests/output/ts_repo");

  reset_testing_dir(&ts_repo_root);

  let cli = Cli::try_parse_from([
    "sketch",
    "--out-dir",
    "tests/output/ts_repo",
    "ts",
    "monorepo",
  ])?;

  execute_cli(cli).await?;

  let root_tsconfig_path = ts_repo_root.join("tsconfig.json");

  assert!(root_tsconfig_path.exists());

  for file in [".oxlintrc.json", "package.json", "pnpm-workspace.yaml"] {
    assert!(ts_repo_root.join(file).exists());
  }

  let options_tsconfig = extract_tsconfig!(ts_repo_root.join("tsconfig.options.json"));

  for dir in ["packages"] {
    assert_dir_exists!(ts_repo_root.join(dir));
  }

  assert_eq!(get_default_root_tsconfig(), options_tsconfig);

  let package_cmd = Cli::try_parse_from([
    "sketch",
    "--out-dir",
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
    get_default_package_tsconfig("../../tsconfig.options.json".to_string(), false),
    package_tsconfig
  );

  let src_tsconfig = extract_tsconfig!(package_dir.join("tsconfig.src.json"));

  assert_eq!(
    get_default_src_tsconfig(false, "../../.out/test_package"),
    src_tsconfig
  );

  let dev_tsconfig = extract_tsconfig!(package_dir.join("tsconfig.dev.json"));

  assert_eq!(
    get_default_dev_tsconfig("../../.out/test_package"),
    dev_tsconfig
  );

  for dir in ["tests", "tests/setup", "src"] {
    let dir = package_dir.join(dir);
    assert!(dir.exists() && dir.is_dir());
  }

  for file in ["src/index.ts"] {
    assert!(package_dir.join(file).exists());
  }

  let package_json = deserialize_json!(PackageJson, package_dir.join("package.json"));

  assert!(package_json
    .contributors
    .iter()
    .any(|p| p.name == "Legolas"));

  assert_eq!(package_json.name.unwrap(), "test_package");

  let app_test = Cli::try_parse_from([
    "sketch",
    "--out-dir",
    "tests/output/ts_repo",
    "ts",
    "package",
    "--app",
    "--update-root-tsconfig",
    "--oxlint",
    "--no-vitest",
    "--package-json",
    "app",
    "apps/app_test",
  ])?;

  execute_cli(app_test).await?;

  let app_package_dir = ts_repo_root.join("apps/app_test");

  // Library files and dirs should be missing
  assert!(!app_package_dir.join("tests").exists());
  assert!(!app_package_dir.join("tsconfig.dev.json").exists());

  // The enabled features
  for file in [".oxlintrc.json"] {
    assert!(app_package_dir.join(file).exists());
  }

  let app_package_json = extract_package_json!(app_package_dir.join("package.json"));

  assert_eq!(app_package_json.name.unwrap(), "app_test");

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
