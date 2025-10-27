use std::{path::PathBuf, str::from_utf8};

use clap::Parser;

use super::{get_clean_example_cmd, reset_testing_dir};
use crate::{
  cli::{cli_tests::get_tree_output, execute_cli, Cli},
  fs::{deserialize_yaml, write_file},
  ts::pnpm::PnpmWorkspace,
};

#[tokio::test]
async fn ts_examples() -> Result<(), Box<dyn std::error::Error>> {
  let examples_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../examples/typescript");
  let output_dir = PathBuf::from("tests/output/ts_examples");
  let commands_dir = output_dir.join("commands");

  reset_testing_dir(&output_dir);
  reset_testing_dir(&commands_dir);

  macro_rules! write_command {
    ($args:expr, $list:expr, $out_file:expr) => {
      get_clean_example_cmd(&$args, &$list, &commands_dir.join($out_file))?
    };
  }

  let monorepo_cmd = [
    "sketch",
    "--ignore-config",
    "-c",
    path_to_str!(examples_dir.join("root_package.yaml")),
    "ts",
    "monorepo",
    "--root-package",
    "root",
    "--pnpm",
    "base",
    "tests/output/ts_examples",
  ];

  write_command!(monorepo_cmd, [1, 2, 3, 9], "monorepo_cmd");

  let monorepo_setup = Cli::try_parse_from(monorepo_cmd)?;

  execute_cli(monorepo_setup).await?;

  get_tree_output(&output_dir, None)?;

  let people_cmd = [
    "sketch",
    "--ignore-config",
    "-c",
    path_to_str!(examples_dir.join("people.yaml")),
    "ts",
    "package",
    "--preset",
    "people-example",
    "tests/output/ts_examples/packages/people-example",
  ];

  write_command!(people_cmd, [1, 2, 3, 7], "people_cmd");

  let people_example = Cli::try_parse_from(people_cmd)?;

  execute_cli(people_example).await?;

  let catalog_cmd = [
    "sketch",
    "--ignore-config",
    "-c",
    path_to_str!(examples_dir.join("catalog.yaml")),
    "ts",
    "package",
    "--preset",
    "with_catalog",
    "tests/output/ts_examples/packages/with-catalog",
  ];

  write_command!(catalog_cmd, [1, 2, 3, 7], "catalog_cmd");

  let catalog_example = Cli::try_parse_from(catalog_cmd)?;

  execute_cli(catalog_example).await?;

  // Checking if the pnpm-workspace file contains the right config + the updated versions
  let pnpm_file: PnpmWorkspace = deserialize_yaml(&output_dir.join("pnpm-workspace.yaml"))?;

  assert!(pnpm_file
    .only_built_dependencies
    .unwrap()
    .contains("esbuild"));
  assert!(pnpm_file.packages.contains("packages/*"));
  assert!(pnpm_file.catalog.get("hono").unwrap().starts_with('^'));
  assert!(pnpm_file
    .catalogs
    .get("svelte")
    .unwrap()
    .get("svelte")
    .unwrap()
    .starts_with('^'));
  assert_eq!(pnpm_file.minimum_release_age.unwrap(), 1440);

  // Check if the workspaces directories were created correctly
  assert!(output_dir.join("packages").is_dir());
  assert!(output_dir.join("apps/test").is_dir());

  let package_gen_cmd = [
    "sketch",
    "--ignore-config",
    "-c",
    path_to_str!(examples_dir.join("new_package.yaml")),
    "ts",
    "package",
    "--preset",
    "frontend",
    "tests/output/ts_examples/packages/frontend",
  ];

  write_command!(package_gen_cmd, [1, 2, 3, 7], "package_gen_cmd");

  let package_gen = Cli::try_parse_from(package_gen_cmd)?;

  execute_cli(package_gen).await?;

  get_tree_output(output_dir.join("packages/frontend"), None)?;

  Ok(())
}

#[tokio::test]
async fn tera_example() -> Result<(), Box<dyn std::error::Error>> {
  let examples_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../examples");
  let output_dir = PathBuf::from("tests/output/templating_examples");

  reset_testing_dir(&output_dir);

  let mut bin = get_bin!();

  let args = [
    "--ignore-config",
    "--templates-dir",
    path_to_str!(examples_dir.join("templating/templates")),
    "render",
    "--template",
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
  assert!(output_str.contains("Path is: Cargo"));
  assert!(output_str.contains("Extension is: toml"));
  assert!(output_str.contains("Matches glob: true"));
  assert!(output_str.contains("They're taking the hobbits to Isengard!"));
  assert!(output_str.contains("Major is: 0"));
  assert!(output_str.contains("Minor is: 1"));
  assert!(output_str.contains("Patch is: 0"));
  assert!(output_str.contains("Version matches >=0.1.0: true"));
  assert!(output_str.contains("Version matches >=0.2.0: false"));
  assert!(output_str.contains("To camelCase: myVar"));
  assert!(output_str.contains("To snake_case: my_var"));
  assert!(output_str.contains("To SCREAMING_CASE: MY_VAR"));
  assert!(output_str.contains("To PascalCase: MyVar"));
  assert!(output_str.contains("Luke, I am your father!"));
  assert!(output_str.contains("Entry: example.j2"));
  assert!(output_str.contains("In yaml form:\npath: Cargo\nextension: toml"));
  assert!(output_str.contains("In toml form:\npath = \"Cargo\"\nextension = \"toml\""));

  write_file(&output_dir.join("cmd"), &cmd_str, true)?;
  write_file(&output_dir.join("output"), output_str, true)?;

  Ok(())
}
