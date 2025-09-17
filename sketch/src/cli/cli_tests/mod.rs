#[macro_use]
mod tests_macros;
mod examples_tests;
mod exec_tests;
mod generated_configs_tests;
mod overwriting_tests;
mod rendering_tests;
mod ts_gen_tests;

use std::{
  fs::{create_dir_all, remove_dir_all, remove_file, File},
  io::Write,
  ops::Range,
  path::{Path, PathBuf},
  process::Command,
};

use crate::cli::Cli;

fn get_tree_output<T: Into<PathBuf>>(dir: T, file: &str) -> Result<(), Box<dyn std::error::Error>> {
  let dir: PathBuf = dir.into();

  let output_file = dir.join(file);

  if output_file.is_file() {
    remove_file(&output_file)?;
  }

  Command::new("tree")
    .current_dir(dir)
    .arg("-a")
    .arg("-I")
    .arg(file)
    .arg("-I")
    .arg("commands")
    .arg("--noreport")
    .arg("-o")
    .arg(file)
    .output()?;

  Ok(())
}

fn get_clean_example_cmd(
  cmd: &[&str],
  remove_range: Range<usize>,
  output: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
  let mut example = String::new();

  for (i, segment) in cmd.iter().enumerate() {
    if !remove_range.contains(&i) {
      if segment.contains(' ') {
        if segment.contains('\'') {
          example.push_str(&format!("\"{}\"", segment));
        } else {
          example.push('\'');
          example.push_str(segment);
          example.push('\'');
        }
      } else {
        example.push_str(segment);
      }

      if i != cmd.len() - 1 {
        example.push(' ');
      }
    }
  }

  let mut file = File::create(output)?;

  file.write_all(example.as_bytes())?;

  Ok(())
}

#[test]
fn generate_cli_docs() -> Result<(), Box<dyn std::error::Error>> {
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
