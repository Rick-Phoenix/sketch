use std::{fs::read_to_string, path::PathBuf};

use clap::Parser;
use pretty_assertions::assert_eq;

use super::reset_testing_dir;
use crate::cli::{cli_tests::get_clean_example_cmd, execute_cli, Cli};

#[tokio::test]
async fn rendered_commands() -> Result<(), Box<dyn std::error::Error>> {
  let output_dir = PathBuf::from("tests/output/commands_tests");
  let config_file = PathBuf::from("tests/commands_tests/commands_tests.toml");
  let commands_dir = output_dir.join("commands");

  reset_testing_dir(&output_dir);
  reset_testing_dir(&commands_dir);

  macro_rules! write_command {
    ($cmd:expr, $range:expr, $out_file:expr) => {
      get_clean_example_cmd(&$cmd, $range, &commands_dir.join($out_file))?
    };
  }

  let literal_template_cmd = [
    "sketch",
    "--set",
    "general=\"kenobi\"",
    "exec",
    "--cwd",
    &output_dir.to_string_lossy(),
    "echo \"hello there!\\ngeneral {{ general }}.\" > command_output.txt",
  ];

  write_command!(literal_template_cmd, 1..3, "exec_literal_cmd");

  let literal = Cli::try_parse_from(literal_template_cmd)?;

  execute_cli(literal).await?;

  let output: String = read_to_string(output_dir.join("command_output.txt"))?;

  assert_eq!(output, "hello there!\ngeneral kenobi.\n");

  let from_file_cmd = [
    "sketch",
    "--set",
    "something=\"space\"",
    "exec",
    "--cwd",
    &output_dir.to_string_lossy(),
    "-f",
    "tests/commands_tests/cmd_from_file.j2",
  ];

  write_command!(from_file_cmd, 1..3, "cmd_from_file");

  let from_file = Cli::try_parse_from(from_file_cmd)?;

  execute_cli(from_file).await?;

  let rendered_from_file: String = read_to_string(output_dir.join("output_from_file.txt"))?;

  assert_eq!(
    rendered_from_file,
    "all the time you have to leave the space!\n"
  );

  let from_template_cmd = [
    "sketch",
    "-c",
    &config_file.to_string_lossy(),
    "--set",
    "category=\"gp2\"",
    "exec",
    "--cwd",
    &output_dir.to_string_lossy(),
    "-t",
    "cmd_template.j2",
  ];

  write_command!(from_template_cmd, 1..3, "exec_from_template_cmd");

  let from_file_in_templates_dir = Cli::try_parse_from(from_template_cmd)?;

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
    "--cwd",
    &output_dir.to_string_lossy(),
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
