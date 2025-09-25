use std::{fs::read_to_string, path::PathBuf};

use clap::Parser;
use pretty_assertions::assert_eq;

use super::{get_clean_example_cmd, reset_testing_dir};
use crate::cli::{execute_cli, Cli};

#[tokio::test]
async fn rendering() -> Result<(), Box<dyn std::error::Error>> {
  let output_dir = PathBuf::from("tests/output/custom_templates");
  let commands_dir = output_dir.join("commands");

  macro_rules! write_command {
    ($args:expr, $list:expr, $out_file:expr) => {
      get_clean_example_cmd(&$args, &$list, &commands_dir.join($out_file))?
    };
  }

  reset_testing_dir(&output_dir);
  reset_testing_dir(&commands_dir);

  let literal_template_cmd = [
    "sketch",
    "--set",
    "location=\"Isengard\"",
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

  Ok(())
}
