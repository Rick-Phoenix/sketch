use std::path::PathBuf;

use clap::Parser;

use super::reset_testing_dir;
use crate::cli::{execute_cli, Cli};

#[tokio::test]
async fn overwrite_test() -> Result<(), Box<dyn std::error::Error>> {
  let output_dir = PathBuf::from("tests/output/overwrite_test");

  reset_testing_dir(&output_dir);

  let first_write = Cli::try_parse_from([
    "sketch",
    "--root-dir",
    &output_dir.to_string_lossy(),
    "render",
    "--content",
    "they're taking the hobbits to Isengard!",
    "overwrite_test.txt",
  ])?;

  execute_cli(first_write).await?;

  let mut cmd = get_bin!();

  cmd
    .args([
      "--no-overwrite",
      "--root-dir",
      &output_dir.to_string_lossy(),
      "render",
      "--content",
      "they're taking the hobbits to Isengard!",
      "overwrite_test.txt",
    ])
    .assert()
    .failure();

  Ok(())
}
