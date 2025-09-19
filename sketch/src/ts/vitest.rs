use std::path::PathBuf;

use askama::Template;
use convert_case::{Case, Casing};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The types of configuration for generating a vitest setup.
/// Can be set to:
/// - True or false to use the default or disable generation altogether.
/// - A string, indicating a preset stored in the global config
/// - A object, with a literal definition
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(untagged)]
pub enum VitestConfigKind {
  Bool(bool),
  Config(VitestConfig),
}

impl VitestConfigKind {
  pub fn is_enabled(&self) -> bool {
    !matches!(self, Self::Bool(false))
  }
}

impl Default for VitestConfigKind {
  fn default() -> Self {
    Self::Bool(true)
  }
}

/// The data used to generate a new vitest setup.
#[derive(Clone, Debug, Template, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[template(path = "ts/vitest.config.ts.j2")]
#[serde(default)]
pub struct VitestConfig {
  /// The path to the tests directory, from the root of the package. [default: 'tests']
  pub tests_dir: String,

  /// The directory where the config file should be placed, starting from the root of the package.
  /// If unset, the `tests_dir` will be used.
  pub out_dir: Option<PathBuf>,

  /// A list of plugins, which will be set up in the config file.
  pub plugins: Vec<String>,

  /// The path to the setup directory, starting from the `tests_dir`. [default: 'setup']
  pub setup_dir: String,

  #[serde(skip)]
  pub(crate) src_rel_path: String,
}

#[derive(Template)]
#[template(path = "ts/tests_setup.ts.j2")]
pub(crate) struct TestsSetupFile;

impl Default for VitestConfig {
  fn default() -> Self {
    Self {
      out_dir: None,
      tests_dir: "tests".to_string(),
      plugins: vec![],
      setup_dir: "setup".to_string(),
      src_rel_path: "../src".to_string(),
    }
  }
}

fn to_camel_case(string: &str) -> String {
  string.to_case(Case::Camel)
}
