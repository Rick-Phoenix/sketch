use std::{fs::read_to_string, path::PathBuf};

use clap::Parser;
use pretty_assertions::assert_eq;

use super::{get_clean_example_cmd, reset_testing_dir};
use crate::cli::{cli_tests::get_tree_output, execute_cli, Cli};

#[tokio::test]
async fn rendering() -> Result<(), Box<dyn std::error::Error>> {
  let output_dir = PathBuf::from("tests/output/custom_templates");
  let commands_dir = output_dir.join("commands");

  let config_dir = PathBuf::from("../examples/templating");
  let config_file = config_dir.join("templating.yaml");

  macro_rules! write_command {
    ($args:expr, $list:expr, $out_file:expr) => {
      get_clean_example_cmd(&$args, &$list, &commands_dir.join($out_file))?
    };
  }

  reset_testing_dir(&output_dir);
  reset_testing_dir(&commands_dir);

  get_tree_output(&config_dir.join("templates"), Some(output_dir.join("tree")))?;

  let from_template_id_cmd = [
    "sketch",
    "-c",
    &config_file.to_string_lossy(),
    "render",
    "--id",
    "hobbits",
    "tests/output/custom_templates/from_template_id.txt",
  ];

  write_command!(from_template_id_cmd, [1, 2], "from_id");

  let from_template_id = Cli::try_parse_from(from_template_id_cmd)?;

  execute_cli(from_template_id).await?;

  let from_template_file_cmd = [
    "sketch",
    "-c",
    &config_file.to_string_lossy(),
    "render",
    "--id",
    "subdir/nested_file.j2",
    "tests/output/custom_templates/from_template_file.txt",
  ];

  write_command!(from_template_file_cmd, [1, 2], "from_template_file");

  let from_template_file = Cli::try_parse_from(from_template_file_cmd)?;

  execute_cli(from_template_file).await?;

  let literal_template_cmd = [
    "sketch",
    "-c",
    &config_file.to_string_lossy(),
    "render",
    "--content",
    "they're taking the hobbits to {{ location }}!",
    "tests/output/custom_templates/from_literal.txt",
  ];

  write_command!(literal_template_cmd, [1, 2], "literal_template_cmd");

  let from_literal = Cli::try_parse_from(literal_template_cmd)?;

  execute_cli(from_literal).await?;

  let from_literal_output: String = read_to_string(output_dir.join("from_literal.txt"))?;

  assert_eq!(
    from_literal_output,
    "they're taking the hobbits to Isengard!"
  );

  let mut cmd = assert_cmd::Command::cargo_bin("sketch").expect("Failed to find the app binary");

  cmd
    .args([
      "--set",
      "location=\"Isengard\"",
      "render",
      "--content",
      "they're taking the hobbits to {{ location }}!",
      "--stdout",
    ])
    .assert()
    .stdout("they're taking the hobbits to Isengard!\n");

  let collection_preset = [
    "sketch",
    "-c",
    &config_file.to_string_lossy(),
    "render-preset",
    "lotr",
    "tests/output/custom_templates/lotr",
  ];

  write_command!(collection_preset, [1, 2, 5], "collection_preset");

  let from_collection_preset = Cli::try_parse_from(collection_preset)?;

  execute_cli(from_collection_preset).await?;

  get_tree_output("tests/output/custom_templates/lotr", None)?;

  let structured_preset = [
    "sketch",
    "-c",
    &config_file.to_string_lossy(),
    "render-preset",
    "structured",
    "tests/output/custom_templates/structured",
  ];

  write_command!(structured_preset, [1, 2, 5], "structured_preset");

  let from_structured_preset = Cli::try_parse_from(structured_preset)?;

  execute_cli(from_structured_preset).await?;

  get_tree_output("tests/output/custom_templates/structured", None)?;

  Ok(())
}
