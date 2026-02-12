#![allow(deprecated)]

#[macro_use]
mod tests_macros;

mod barrel_tests;
mod examples_tests;
mod exec_tests;
mod generated_configs_tests;
mod overwriting_tests;
mod presets_tests;
mod rendering_tests;
mod rust_gen_tests;
mod vars_files_tests;

use std::fmt::Write as FmtWrite;
use std::fs::remove_file;

use crate::*;
use indoc::indoc;

use crate::cli::*;
use maplit::btreemap;
use pretty_assertions::assert_eq as pretty_assert_eq;

impl Cli {
	async fn execute_with<I, T>(itr: I) -> Result<(), AppError>
	where
		I: IntoIterator<Item = T>,
		T: Into<std::ffi::OsString> + Clone,
	{
		Self::try_parse_from(itr)
			.expect("Failed to parse options")
			.execute()
			.await
	}
}

fn get_tree_output<T: AsRef<Path>>(
	dir: T,
	file: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
	let dir = dir.as_ref();

	let output_file = if let Some(path) = file {
		path
	} else {
		dir.join("tree_output.txt")
	};

	if output_file.is_file() {
		remove_file(&output_file)?;
	}

	Command::new("tree")
		.arg(dir.to_string_lossy().to_string())
		.arg("-a")
		.arg("-I")
		.arg("tree_output.txt")
		.arg("-I")
		.arg(".git")
		.arg("-I")
		.arg("commands")
		.arg("--noreport")
		.arg("-o")
		.arg(output_file)
		.output()?;

	Ok(())
}

fn get_clean_example_cmd(
	cmd: &[&str],
	discarded_segments: &[usize],
	output: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
	let mut example = String::new();

	for (i, segment) in cmd.iter().enumerate() {
		if !discarded_segments.contains(&i) {
			if segment.contains(' ') {
				if segment.contains('\'') {
					let _ = write!(example, "\"{segment}\"");
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

	let mut file = File::create("../docs/src/cli_docs.md")?;

	file.write_all(markdown.as_bytes())?;

	Ok(())
}

fn reset_testing_dir<T: Into<PathBuf>>(dir: T) {
	let dir: PathBuf = dir.into();
	if dir.exists() {
		remove_dir_all(dir.as_path()).unwrap_or_else(|e| {
			panic!("Failed to empty the output dir '{}': {}", dir.display(), e)
		});
	}

	create_dir_all(dir.as_path())
		.unwrap_or_else(|e| panic!("Failed to create the output dir '{}': {}", dir.display(), e));
}

#[test]
fn verify_cli() {
	use clap::CommandFactory;
	Cli::command().debug_assert();
}
