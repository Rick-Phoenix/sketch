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

  macro_rules! exists {
    ($name:literal) => {
      assert!(output_dir.join($name).is_file())
    };
  }

  reset_testing_dir(&output_dir);
  reset_testing_dir(&commands_dir);

  get_tree_output(&config_dir.join("templates"), Some(output_dir.join("tree")))?;

  // From known template

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

  exists!("from_template_id.txt");

  // From any file

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

  exists!("from_template_file.txt");

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

  // Presets tests

  // Remote

  let out_dir = output_dir.join("remote");

  let from_remote_preset_cmd = [
    "sketch",
    "-c",
    &config_file.to_string_lossy(),
    "--set",
    "continuation=\"gp2 engine... gp2!\"",
    "render-preset",
    "remote",
    &out_dir.to_string_lossy(),
  ];

  let from_remote_preset = Cli::try_parse_from(from_remote_preset_cmd)?;

  execute_cli(from_remote_preset).await?;

  write_command!(from_remote_preset_cmd, [1, 2, 7], "remote");
  get_tree_output("tests/output/custom_templates/remote", None)?;

  let expected_output = "Roses are red, violets are blue, gp2 engine... gp2!\n";

  let top_level_file = read_to_string(out_dir.join("some_file"))?;

  assert_eq!(top_level_file, expected_output);

  let nested_file = read_to_string(out_dir.join("subdir/nested/nested_file"))?;

  assert_eq!(nested_file, expected_output);

  // Granular

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

  exists!("lotr/hobbits.txt");
  exists!("lotr/subdir/breakfast.txt");

  get_tree_output("tests/output/custom_templates/lotr", None)?;

  // Structured

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

  exists!("structured/nested_file");
  exists!("structured/nested/more_nested_file");

  get_tree_output("tests/output/custom_templates/structured", None)?;

  // Extended

  let extended_preset = [
    "sketch",
    "-c",
    &config_file.to_string_lossy(),
    "render-preset",
    "lotr",
    "tests/output/custom_templates/extended",
  ];

  let from_extended_preset = Cli::try_parse_from(extended_preset)?;

  execute_cli(from_extended_preset).await?;

  exists!("extended/hobbits.txt");
  exists!("extended/subdir/breakfast.txt");

  Ok(())
}
