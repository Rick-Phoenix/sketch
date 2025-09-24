use std::path::PathBuf;

use clap::Parser;

use super::reset_testing_dir;
use crate::cli::{execute_cli, Cli};

#[tokio::test]
async fn overwrite_test() -> Result<(), Box<dyn std::error::Error>> {
  let output_dir = PathBuf::from("tests/output/overwrite_test");

  reset_testing_dir(&output_dir);

  let output_file = output_dir.join("overwrite_test.txt");

  let first_write = Cli::try_parse_from([
    "sketch",
    "render",
    "--content",
    "they're taking the hobbits to Isengard!",
    &output_file.to_string_lossy(),
  ])?;

  execute_cli(first_write).await?;

  let mut cmd = get_bin!();

  // Ensuring the second write fails
  cmd
    .args([
      "--no-overwrite",
      "render",
      "--content",
      "they're taking the hobbits to Isengard!",
      &output_file.to_string_lossy(),
    ])
    .assert()
    .failure();

  Ok(())
}
