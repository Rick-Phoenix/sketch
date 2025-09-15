use std::{
  fs::File,
  io::Write,
  path::{Path, PathBuf},
  process::Command,
  sync::LazyLock,
};

use clap::Parser;
use maplit::btreeset;
use pretty_assertions::assert_eq;

use super::reset_testing_dir;
use crate::{
  cli::{execute_cli, Cli},
  package_json::{PackageJson, Person, PersonData},
  ts_config::{CompilerOptions, TsConfig},
};

static EXAMPLE_CMDS_DIR: LazyLock<PathBuf> =
  LazyLock::new(|| PathBuf::from("tests/output/example_cmds"));

fn extract_example_and_get_cmd(
  cmd: &str,
  output: &str,
  config_path: &Path,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
  let mut cmd: Vec<String> = cmd.split_whitespace().map(|s| s.to_string()).collect();

  let mut file = File::create(EXAMPLE_CMDS_DIR.join(output))?;

  file.write_all(cmd.join(" ").as_bytes())?;

  cmd.insert(1, "-c".to_string());
  cmd.insert(2, config_path.to_string_lossy().to_string());

  Ok(cmd)
}

#[tokio::test]
async fn ts_examples() -> Result<(), Box<dyn std::error::Error>> {
  let examples_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../examples/typescript");
  let output_dir = PathBuf::from("tests/output/ts_examples");

  reset_testing_dir(EXAMPLE_CMDS_DIR.as_path());
  reset_testing_dir(&output_dir);

  let monorepo_cmd = extract_example_and_get_cmd(
    "sketch ts monorepo",
    "monorepo_cmd",
    &examples_dir.join("root_package.yaml"),
  )?;

  let monorepo_setup = Cli::try_parse_from(&monorepo_cmd)?;

  execute_cli(monorepo_setup).await?;

  Command::new("tree")
    .current_dir(&output_dir)
    .arg("-a")
    .arg("-I")
    .arg("*.txt")
    .arg("-o")
    .arg("tree_output.txt")
    .output()?;

  let tsconfigs_cmd = extract_example_and_get_cmd(
    "sketch ts package --preset tsconfig-example",
    "tsconfig_cmd",
    &examples_dir.join("tsconfig_presets.yaml"),
  )?;

  let tsconfigs_example = Cli::try_parse_from(tsconfigs_cmd)?;

  execute_cli(tsconfigs_example).await?;

  let tsconfigs_output = output_dir.join("packages/tsconfig-example");

  let tsconfig_with_override =
    deserialize_json!(TsConfig, tsconfigs_output.join("tsconfig.src.json"));

  assert_eq!(
    tsconfig_with_override,
    TsConfig {
      compiler_options: Some(CompilerOptions {
        verbatim_module_syntax: Some(true),
        emit_declaration_only: Some(true),
        ..Default::default()
      }),
      ..Default::default()
    }
  );

  let extended_preset = deserialize_json!(TsConfig, tsconfigs_output.join("tsconfig.dev.json"));

  assert_eq!(
    extended_preset,
    TsConfig {
      include: Some(btreeset! { "src".to_string(), "tests".to_string() }),
      compiler_options: Some(CompilerOptions {
        no_emit: Some(true),
        ..Default::default()
      }),
      ..Default::default()
    }
  );

  let package_json_cmd = extract_example_and_get_cmd(
    "sketch ts package --preset svelte_frontend",
    "package_json_cmd",
    &examples_dir.join("extending_package_json.yaml"),
  )?;

  let extended_package_json_example = Cli::try_parse_from(package_json_cmd)?;

  execute_cli(extended_package_json_example).await?;

  let extended = deserialize_json!(
    PackageJson,
    output_dir.join("packages/svelte_frontend/package.json")
  );

  assert_eq!(extended.license.unwrap(), "MIT");
  assert_eq!(
    extended.author.unwrap(),
    Person::Data(PersonData {
      name: "Bruce Wayne".to_string(),
      email: Some("i-may-or-may-not-be-batman@gotham.com".to_string()),
      ..Default::default()
    })
  );
  assert_eq!(extended.scripts.get("dev").unwrap(), "vite dev");
  assert_eq!(extended.scripts.get("build").unwrap(), "vite build");

  let people_cmd = extract_example_and_get_cmd(
    "sketch ts package --preset people-example",
    "people_cmd",
    &examples_dir.join("people.yaml"),
  )?;

  let people_example = Cli::try_parse_from(people_cmd)?;

  execute_cli(people_example).await?;

  let catalog_cmd = extract_example_and_get_cmd(
    "sketch ts package --preset with_catalog",
    "catalog_cmd",
    &examples_dir.join("catalog.yaml"),
  )?;

  let catalog_example = Cli::try_parse_from(catalog_cmd)?;

  execute_cli(catalog_example).await?;

  Ok(())
}
