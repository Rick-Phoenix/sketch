use std::{
  fs::{create_dir_all, read_dir, remove_dir_all, remove_file},
  path::PathBuf,
  sync::{LazyLock, Once},
};

use clap::Parser;

use crate::cli::{execute_cli, Cli};

static OUTPUT_DIR: LazyLock<PathBuf> = LazyLock::new(|| PathBuf::from("tests/output/ts_repo"));

static SETUP: Once = Once::new();

fn run_setup() {
  if OUTPUT_DIR.exists() {
    remove_dir_all(OUTPUT_DIR.as_path()).expect("Failed to empty the output dir");
  }

  create_dir_all(OUTPUT_DIR.as_path()).expect("Failed to create OUTPUT_DIR");
}

#[tokio::test]
async fn cli_root_dir() -> Result<(), Box<dyn std::error::Error>> {
  SETUP.call_once(|| run_setup());

  let cli = Cli::try_parse_from([
    "sketch",
    "--dry-run",
    "--debug",
    "-c",
    "sketch.toml",
    "ts",
    "monorepo",
  ])?;

  execute_cli(cli).await?;

  Ok(())
}

#[test]
fn verify_cli() {
  use clap::CommandFactory;
  Cli::command().debug_assert();
}
