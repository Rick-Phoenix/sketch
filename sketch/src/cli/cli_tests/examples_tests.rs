use std::{path::PathBuf, str::from_utf8};

use clap::Parser;

use super::{get_clean_example_cmd, reset_testing_dir};
use crate::{
  cli::{cli_tests::get_tree_output, execute_cli, Cli},
  fs::write_file,
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
    "--out-dir",
    &output_dir.to_string_lossy(),
    "-c",
    path_to_str!(examples_dir.join("root_package.yaml")),
    "ts",
    "monorepo",
  ];

  write_command!(monorepo_cmd, 1..5, "monorepo_cmd");

  let monorepo_setup = Cli::try_parse_from(&monorepo_cmd)?;

  execute_cli(monorepo_setup).await?;

  get_tree_output(&output_dir, "tree_output.txt")?;

  let people_cmd = [
    "sketch",
    "--out-dir",
    &output_dir.to_string_lossy(),
    "-c",
    path_to_str!(examples_dir.join("people.yaml")),
    "ts",
    "package",
    "--preset",
    "people-example",
  ];

  write_command!(people_cmd, 1..5, "people_cmd");

  let people_example = Cli::try_parse_from(people_cmd)?;

  execute_cli(people_example).await?;

  let catalog_cmd = [
    "sketch",
    "--out-dir",
    &output_dir.to_string_lossy(),
    "-c",
    path_to_str!(examples_dir.join("catalog.yaml")),
    "ts",
    "package",
    "--preset",
    "with_catalog",
  ];

  write_command!(catalog_cmd, 1..5, "catalog_cmd");

  let catalog_example = Cli::try_parse_from(catalog_cmd)?;

  execute_cli(catalog_example).await?;

  let package_gen_cmd = [
    "sketch",
    "--out-dir",
    &output_dir.to_string_lossy(),
    "-c",
    path_to_str!(examples_dir.join("new_package.yaml")),
    "ts",
    "package",
    "--preset",
    "frontend",
  ];

  write_command!(package_gen_cmd, 1..5, "package_gen_cmd");

  let package_gen = Cli::try_parse_from(package_gen_cmd)?;

  execute_cli(package_gen).await?;

  get_tree_output(&output_dir.join("packages/frontend"), "tree_output.txt")?;

  Ok(())
}

#[tokio::test]
async fn tera_example() -> Result<(), Box<dyn std::error::Error>> {
  let examples_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../examples");
  let output_dir = PathBuf::from("tests/output/templating_examples");

  reset_testing_dir(&output_dir);

  let mut bin = get_bin!();

  let args = [
    "--templates-dir",
    path_to_str!(examples_dir.join("templating")),
    "render",
    "--id",
    "example.j2",
    "--stdout",
  ];

  let var_name = "GREETING";
  let var_value = "hello,world";

  let cmd_str = format!(
    "{}=\"{}\" sketch {}",
    var_name,
    var_value,
    args.split_at(2).1.join(" ")
  );

  let output = bin.env(var_name, var_value).args(args).output()?;

  let output_str = from_utf8(&output.stdout)?.trim();

  if output_str.is_empty() {
    panic!(
      "Error in the template output: {}",
      from_utf8(&output.stderr)?
    );
  }

  assert!(output_str.contains("Current arch is: x86_64"));
  assert!(output_str.contains("Current os is: linux"));
  assert!(output_str.contains("Current os family is: unix"));
  assert!(output_str.contains("Is unix: true"));
  assert!(output_str.contains("It's a dir!"));
  assert!(output_str.contains("It's a file!"));
  assert!(output_str.contains("First segment is: hello"));
  assert!(output_str.contains("Second segment is: world"));
  assert!(output_str.contains("Basename is: myfile"));
  assert!(output_str.contains("Parent dir is: mydir"));
  assert!(output_str.contains("Path is: sketch"));
  assert!(output_str.contains("Extension is: toml"));

  write_file(&output_dir.join("cmd"), &cmd_str, true)?;
  write_file(&output_dir.join("output"), &output_str, true)?;

  Ok(())
}
