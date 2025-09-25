use std::{fs::read_to_string, path::PathBuf};

use clap::Parser;
use indoc::indoc;

use crate::cli::{
  cli_tests::{get_clean_example_cmd, reset_testing_dir},
  execute_cli, Cli,
};

#[tokio::test]
async fn rendered_commands() -> Result<(), Box<dyn std::error::Error>> {
  let output_dir = PathBuf::from("tests/output/ts_barrel");
  let commands_dir = output_dir.join("commands");

  reset_testing_dir(&output_dir);
  reset_testing_dir(&commands_dir);

  macro_rules! write_command {
    ($cmd:expr, $list:expr, $out_file:expr) => {
      get_clean_example_cmd(&$cmd, &$list, &commands_dir.join($out_file))?
    };
  }

  let output_file = output_dir.join("index.ts");

  let basic_cmd = [
    "sketch",
    "ts",
    "barrel",
    "-o",
    &output_file.to_string_lossy(),
    "tests/ts_barrel",
  ];

  write_command!(basic_cmd, [3, 4], "barrel");

  let basic = Cli::try_parse_from(basic_cmd)?;

  execute_cli(basic).await?;

  let output = read_to_string(&output_file)?;

  assert_eq!(
    output,
    indoc! {r###"
    export * from "nested/file1";
    export * from "nested/nested2/file2";
  "###}
  );

  let with_exclude_and_ext_cmd = [
    "sketch",
    "ts",
    "barrel",
    "--exclude",
    "**/nested2/*",
    "--js-ext",
    "-o",
    &output_file.to_string_lossy(),
    "tests/ts_barrel",
  ];

  let with_exclude = Cli::try_parse_from(with_exclude_and_ext_cmd)?;

  execute_cli(with_exclude).await?;

  let output = read_to_string(&output_file)?;

  assert_eq!(
    output,
    indoc! {r###"
    export * from "nested/file1.js";
  "###}
  );

  let with_allowed_ext_cmd = [
    "sketch",
    "ts",
    "barrel",
    "--exclude",
    "**/file1.ts",
    "--keep-ext",
    "ts",
    "-o",
    &output_file.to_string_lossy(),
    "tests/ts_barrel",
  ];

  let with_allowed_ext = Cli::try_parse_from(with_allowed_ext_cmd)?;

  execute_cli(with_allowed_ext).await?;

  let output = read_to_string(&output_file)?;

  assert_eq!(
    output,
    indoc! {r###"
    export * from "nested/nested2/file2.ts";
  "###}
  );

  Ok(())
}
