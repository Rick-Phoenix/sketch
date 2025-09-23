use std::{fs::read_to_string, path::PathBuf};

use clap::Parser;
use pretty_assertions::assert_eq;
use serde::{Deserialize, Serialize};

use super::{get_clean_example_cmd, reset_testing_dir};
use crate::{
  cli::{cli_tests::get_tree_output, execute_cli, get_config_from_cli, Cli},
  fs::deserialize_yaml,
};

#[derive(Debug, Serialize, Deserialize)]
struct CustomTemplateTest {
  pub my_var: usize,
}

#[tokio::test]
async fn cli_rendering() -> Result<(), Box<dyn std::error::Error>> {
  let output_dir = PathBuf::from("tests/output/custom_templates");
  let commands_dir = output_dir.join("commands");

  macro_rules! write_command {
    ($args:expr, $remove_range:expr, $out_file:expr) => {
      get_clean_example_cmd(&$args, $remove_range, &commands_dir.join($out_file))?
    };
  }

  reset_testing_dir(&output_dir);
  reset_testing_dir(&commands_dir);

  let preset_rendering_args = [
    "sketch",
    "-c",
    "tests/custom_templates/custom_templates.yaml",
    "--out-dir",
    "tests/output/custom_templates",
    "render-preset",
    "test",
  ];

  let rendering_cmd = Cli::try_parse_from(preset_rendering_args)?;

  execute_cli(rendering_cmd.clone()).await?;

  write_command!(preset_rendering_args, 1..5, "render_preset_cmd");

  get_tree_output(&output_dir, "render_preset_tree.txt")?;

  let from_single_file_cmd = [
    "sketch",
    "-c",
    "tests/custom_templates/custom_templates.yaml",
    "--out-dir",
    "tests/output/custom_templates",
    "render",
    "-f",
    "tests/custom_templates/single_file.j2",
    "from_single_file.yaml",
  ];

  let from_single_file = Cli::try_parse_from(from_single_file_cmd)?;

  execute_cli(from_single_file.clone()).await?;

  write_command!(from_single_file_cmd, 1..5, "from_single_file_cmd");

  let output: CustomTemplateTest = deserialize_yaml(&output_dir.join("from_single_file.yaml"))?;

  assert_eq!(output.my_var, 15);

  let from_config_template_cmd = [
    "sketch",
    "-c",
    "tests/custom_templates/custom_templates.yaml",
    "--out-dir",
    "tests/output/custom_templates",
    "render",
    "--id",
    "lit_template",
    "from_config_template.yaml",
  ];

  let from_config_template = Cli::try_parse_from(from_config_template_cmd)?;

  execute_cli(from_config_template.clone()).await?;

  write_command!(from_config_template_cmd, 1..5, "from_config_template_cmd");

  let output: CustomTemplateTest = deserialize_yaml(&output_dir.join("from_config_template.yaml"))?;

  assert_eq!(output.my_var, 15);

  let from_template_file_cmd = [
    "sketch",
    "-c",
    "tests/custom_templates/custom_templates.yaml",
    "--out-dir",
    "tests/output/custom_templates",
    "render",
    "--id",
    "subdir/nested.j2",
    "from_template_file.yaml",
  ];

  let from_template_file = Cli::try_parse_from(from_template_file_cmd)?;

  execute_cli(from_template_file.clone()).await?;

  get_tree_output("tests/templates", "templates_tree.txt")?;

  write_command!(from_template_file_cmd, 1..5, "from_template_file_cmd");

  let output: CustomTemplateTest = deserialize_yaml(&output_dir.join("from_template_file.yaml"))?;

  assert_eq!(output.my_var, 15);

  let cli_override_args = [
    "sketch",
    "-c",
    "tests/custom_templates/custom_templates.yaml",
    "--out-dir",
    "tests/output/custom_templates",
    "--set",
    "my_var=25",
    "render",
    "--id",
    "lit_template",
    "with_cli_override.yaml",
  ];

  let with_cli_override = Cli::try_parse_from(cli_override_args)?;

  execute_cli(with_cli_override).await?;

  write_command!(cli_override_args, 1..5, "cli_override_cmd");

  let config = get_config_from_cli(rendering_cmd).await?;

  let templates = config.templating_presets.get("test").unwrap();

  for template in templates {
    let output_path = output_dir.join(&template.output);
    let output: CustomTemplateTest = deserialize_yaml(&output_path)?;

    let output_path_str = output_path.to_string_lossy();

    // Checking local context override
    if output_path_str.ends_with("with_override.yaml") {
      assert_eq!(output.my_var, 20);
      // Checking override from cli
    } else if output_path_str == "with_cli_override.yaml" {
      assert_eq!(output.my_var, 25);
    } else {
      assert_eq!(output.my_var, 15);
    }
  }

  let literal_template_cmd = [
    "sketch",
    "--out-dir",
    "tests/output/custom_templates",
    "--set",
    "location=\"Isengard\"",
    "render",
    "--content",
    "they're taking the hobbits to {{ location }}!",
    "from_literal.txt",
  ];

  write_command!(literal_template_cmd, 1..3, "literal_template_cmd");

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
