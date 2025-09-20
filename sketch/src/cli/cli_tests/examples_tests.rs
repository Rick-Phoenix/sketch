use std::path::PathBuf;

use clap::Parser;

use super::{get_clean_example_cmd, reset_testing_dir};
use crate::cli::{cli_tests::get_tree_output, execute_cli, Cli};

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
