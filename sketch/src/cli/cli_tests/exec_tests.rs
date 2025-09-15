use std::{fs::read_to_string, path::PathBuf};

use clap::Parser;
use pretty_assertions::assert_eq;

use super::reset_testing_dir;
use crate::cli::{execute_cli, Cli};

#[tokio::test]
async fn rendered_commands() -> Result<(), Box<dyn std::error::Error>> {
  let output_dir = PathBuf::from("tests/output/commands_tests");
  let config_file = PathBuf::from("tests/commands_tests/commands_tests.toml");

  reset_testing_dir(&output_dir);

  let literal = Cli::try_parse_from([
    "sketch",
    "--root-dir",
    &output_dir.to_string_lossy(),
    "--set",
    "general=\"kenobi\"",
    "exec",
    "echo \"hello there!\\ngeneral {{ general }}.\" > command_output.txt",
  ])?;

  execute_cli(literal).await?;

  let output: String = read_to_string(output_dir.join("command_output.txt"))?;

  assert_eq!(output, "hello there!\ngeneral kenobi.\n");

  let from_file = Cli::try_parse_from([
    "sketch",
    "--root-dir",
    &output_dir.to_string_lossy(),
    "--set",
    "something=\"space\"",
    "exec",
    "-f",
    "../../commands_tests/cmd_from_file.j2",
  ])?;

  execute_cli(from_file).await?;

  let rendered_from_file: String = read_to_string(output_dir.join("output_from_file.txt"))?;

  assert_eq!(
    rendered_from_file,
    "all the time you have to leave the space!\n"
  );

  let from_file_in_templates_dir = Cli::try_parse_from([
    "sketch",
    "-c",
    &config_file.to_string_lossy(),
    "--set",
    "category=\"gp2\"",
    "exec",
    "-t",
    "cmd_template.j2",
  ])?;

  execute_cli(from_file_in_templates_dir).await?;

  let rendered_from_file_in_templates_dir: String =
    read_to_string(output_dir.join("output_from_templates_dir.txt"))?;

  assert_eq!(
    rendered_from_file_in_templates_dir,
    "gp2 engine... gp2... argh!\n"
  );

  let from_template_id = Cli::try_parse_from([
    "sketch",
    "-c",
    &config_file.to_string_lossy(),
    "--set",
    "condition=\"slower\"",
    "exec",
    "-t",
    "cmd_template",
  ])?;

  execute_cli(from_template_id).await?;

  let rendered_from_file_in_templates_dir: String =
    read_to_string(output_dir.join("output_from_template_id.txt"))?;

  assert_eq!(
    rendered_from_file_in_templates_dir,
    "engine feels good, much slower than before... amazing\n"
  );

  Ok(())
}
