use std::{
  fs::{read_to_string, File},
  path::PathBuf,
};

use clap::Parser;
use pretty_assertions::assert_eq;
use serde::{Deserialize, Serialize};

use super::reset_testing_dir;
use crate::cli::{execute_cli, get_config_from_cli, Cli};

#[derive(Debug, Serialize, Deserialize)]
struct CustomTemplateTest {
  pub my_var: usize,
}

#[tokio::test]
async fn cli_rendering() -> Result<(), Box<dyn std::error::Error>> {
  let output_dir = PathBuf::from("tests/output/custom_templates");

  reset_testing_dir(&output_dir);

  let rendering_cmd = Cli::try_parse_from([
    "sketch",
    "-c",
    "tests/custom_templates/custom_templates.toml",
    "--root-dir",
    "tests/output/custom_templates",
    "render-preset",
    "test",
  ])?;

  let with_cli_override = Cli::try_parse_from([
    "sketch",
    "-c",
    "tests/custom_templates/custom_templates.toml",
    "--root-dir",
    "tests/output/custom_templates",
    "--set",
    "my_var=25",
    "render",
    "--id",
    "lit_template",
    "with_cli_override.yaml",
  ])?;

  execute_cli(rendering_cmd.clone()).await?;
  execute_cli(with_cli_override.clone()).await?;

  let config = get_config_from_cli(rendering_cmd).await?;

  let templates = config.templating_presets.get("test").unwrap();

  for template in templates {
    let output_path = output_dir.join(&template.output);
    let output = deserialize_yaml!(CustomTemplateTest, output_path);

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

  let from_literal = Cli::try_parse_from([
    "sketch",
    "--root-dir",
    "tests/output/custom_templates",
    "--set",
    "location=\"Isengard\"",
    "render",
    "--content",
    "they're taking the hobbits to {{ location }}!",
    "from_literal.txt",
  ])?;

  execute_cli(from_literal).await?;

  let from_literal_output: String = read_to_string(output_dir.join("from_literal.txt"))?;

  assert_eq!(
    from_literal_output,
    "they're taking the hobbits to Isengard!"
  );

  let mut cmd = assert_cmd::Command::cargo_bin("sketch").expect("Failed to find the app binary");

  cmd
    .args([
      "--root-dir",
      "tests/output/custom_templates",
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
