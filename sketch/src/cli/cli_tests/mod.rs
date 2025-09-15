#[macro_use]
mod tests_macros;
mod examples_tests;
mod exec_tests;
mod generated_configs_tests;
mod overwriting_tests;
mod rendering_tests;
mod ts_gen_tests;

use std::{
  fs::{create_dir_all, remove_dir_all, File},
  io::Write,
  path::PathBuf,
};

use crate::cli::Cli;

#[test]
fn generate_docs() -> Result<(), Box<dyn std::error::Error>> {
  let markdown: String = clap_markdown::help_markdown::<Cli>();

  let mut file = File::create("../docs/src/cli.md")?;

  file.write_all(markdown.as_bytes())?;

  Ok(())
}

fn reset_testing_dir<T: Into<PathBuf>>(dir: T) {
  let dir: PathBuf = dir.into();
  if dir.exists() {
    remove_dir_all(dir.as_path())
      .unwrap_or_else(|e| panic!("Failed to empty the output dir '{}': {}", dir.display(), e));
  }

  create_dir_all(dir.as_path())
    .unwrap_or_else(|e| panic!("Failed to create the output dir '{}': {}", dir.display(), e));
}

#[test]
fn verify_cli() {
  use clap::CommandFactory;
  Cli::command().debug_assert();
}
