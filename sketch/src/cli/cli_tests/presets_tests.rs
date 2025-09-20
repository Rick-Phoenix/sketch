use std::path::PathBuf;

use clap::Parser;
use maplit::{btreemap, btreeset};
use pretty_assertions::assert_eq;

use super::reset_testing_dir;
use crate::{
  cli::{execute_cli, Cli},
  fs::deserialize_json,
  ts::{
    oxlint::OxlintConfig,
    package_json::PackageJson,
    ts_config::{TsConfig, TsConfigReference},
  },
};

#[tokio::test]
async fn presets() -> Result<(), Box<dyn std::error::Error>> {
  let examples_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../examples/typescript");
  let out_dir = PathBuf::from("tests/output/presets/packages/extending_presets_example");

  reset_testing_dir(&out_dir);

  let oxlint_test = Cli::try_parse_from([
    "sketch",
    "-c",
    path_to_str!(examples_dir.join("presets.yaml")),
    "ts",
    "package",
    "--preset",
    "example",
  ])?;

  execute_cli(oxlint_test).await?;

  let oxlint_result: OxlintConfig = deserialize_json(&out_dir.join(".oxlintrc.json"))?;

  assert_eq!(
    oxlint_result.ignore_patterns.unwrap(),
    btreeset! { "**/node_modules/**".to_string(), ".cache".to_string(), ".output".to_string() }
  );

  let package_json_result: PackageJson = deserialize_json(&out_dir.join("package.json"))?;

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

  let tsconfig_result: TsConfig = deserialize_json(&out_dir.join("tsconfig.json"))?;

  assert_eq!(
    tsconfig_result.references.unwrap(),
    btreeset! {
      TsConfigReference { path: "/some/path".to_string() },
      TsConfigReference { path: "/other/path".to_string() },
    }
  );

  assert_eq!(
    tsconfig_result.include.unwrap(),
    btreeset! {
      "src".to_string(), "tests".to_string(), "scripts".to_string()
    }
  );

  let compiler_options = tsconfig_result.compiler_options.unwrap();

  assert_eq!(compiler_options.no_emit.unwrap(), false);

  assert_eq!(compiler_options.verbatim_module_syntax.unwrap(), true);

  Ok(())
}
