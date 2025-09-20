use std::{fs::File, path::PathBuf};

use clap::Parser;
use maplit::btreeset;
use pretty_assertions::assert_eq;

use super::{get_clean_example_cmd, reset_testing_dir};
use crate::{
  cli::{cli_tests::get_tree_output, execute_cli, Cli},
  ts::{
    package_json::{PackageJson, PersonData},
    ts_config::{CompilerOptions, TsConfig},
  },
};

#[tokio::test]
async fn ts_examples() -> Result<(), Box<dyn std::error::Error>> {
  let examples_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../examples/typescript");
  let output_dir = PathBuf::from("tests/output/ts_examples");
  let commands_dir = output_dir.join("commands");

  reset_testing_dir(&output_dir);
  reset_testing_dir(&commands_dir);

  macro_rules! write_command {
    ($args:expr, $remove_range:expr, $out_file:expr) => {
      get_clean_example_cmd(&$args, $remove_range, &commands_dir.join($out_file))?
    };
  }

  let monorepo_cmd = [
    "sketch",
    "-c",
    path_to_str!(examples_dir.join("root_package.yaml")),
    "ts",
    "monorepo",
  ];

  write_command!(monorepo_cmd, 1..3, "monorepo_cmd");

  let monorepo_setup = Cli::try_parse_from(&monorepo_cmd)?;

  execute_cli(monorepo_setup).await?;

  get_tree_output(&output_dir, "tree_output.txt")?;

  let tsconfigs_cmd = [
    "sketch",
    "-c",
    path_to_str!(examples_dir.join("tsconfig_presets.yaml")),
    "ts",
    "package",
    "--preset",
    "tsconfig-example",
  ];

  write_command!(tsconfigs_cmd, 1..3, "tsconfig_cmd");

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

  let package_json_cmd = [
    "sketch",
    "-c",
    path_to_str!(examples_dir.join("extending_package_json.yaml")),
    "ts",
    "package",
    "--preset",
    "svelte_frontend",
  ];

  write_command!(package_json_cmd, 1..3, "package_json_cmd");

  let extended_package_json_example = Cli::try_parse_from(package_json_cmd)?;

  execute_cli(extended_package_json_example).await?;

  let extended = deserialize_json!(
    PackageJson,
    output_dir.join("packages/svelte_frontend/package.json")
  );

  assert_eq!(extended.license.unwrap(), "MIT");
  assert_eq!(
    extended.author.unwrap(),
    PersonData {
      name: "Bruce Wayne".to_string(),
      email: Some("i-may-or-may-not-be-batman@gotham.com".to_string()),
      ..Default::default()
    }
  );
  assert_eq!(extended.scripts.get("dev").unwrap(), "vite dev");
  assert_eq!(extended.scripts.get("build").unwrap(), "vite build");

  let people_cmd = [
    "sketch",
    "-c",
    path_to_str!(examples_dir.join("people.yaml")),
    "ts",
    "package",
    "--preset",
    "people-example",
  ];

  write_command!(people_cmd, 1..3, "people_cmd");

  let people_example = Cli::try_parse_from(people_cmd)?;

  execute_cli(people_example).await?;

  let catalog_cmd = [
    "sketch",
    "-c",
    path_to_str!(examples_dir.join("catalog.yaml")),
    "ts",
    "package",
    "--preset",
    "with_catalog",
  ];

  write_command!(catalog_cmd, 1..3, "catalog_cmd");

  let catalog_example = Cli::try_parse_from(catalog_cmd)?;

  execute_cli(catalog_example).await?;

  let package_gen_cmd = [
    "sketch",
    "-c",
    path_to_str!(examples_dir.join("new_package.yaml")),
    "ts",
    "package",
    "--preset",
    "frontend",
  ];

  write_command!(package_gen_cmd, 1..3, "package_gen_cmd");

  let package_gen = Cli::try_parse_from(package_gen_cmd)?;

  execute_cli(package_gen).await?;

  get_tree_output(&output_dir.join("packages/frontend"), "tree_output.txt")?;

  Ok(())
}
