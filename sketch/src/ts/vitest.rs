use std::{fmt::Display, path::PathBuf};

use askama::Template;
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
  Id(String),
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

  /// The path to the setup directory, starting from the `tests_dir`. [default: 'setup']
  pub setup_dir: String,

  /// A list of setup files. The paths will be joined to the `setup_dir`.
  pub setup_files: Vec<String>,

  /// The environment that will be used for testing. See more: https://vitest.dev/config/#environment
  pub environment: Environment,

  /// By default, vitest does not provide global APIs for explicitness. If you prefer to use the APIs globally like Jest, you can pass the --globals option to CLI or add globals: true in the config. See more: https://vitest.dev/config/#globals
  pub globals: bool,

  /// Silent console output from tests.
  /// Use 'passed-only' to see logs from failing tests only. Logs from failing tests are printed after a test has finished.
  pub silent: Silent,

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
      setup_dir: "setup".to_string(),
      src_rel_path: "../src".to_string(),
      environment: Environment::Node,
      globals: true,
      setup_files: Default::default(),
      silent: Silent::PassedOnly,
    }
  }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum Silent {
  #[serde(rename = "passed-only")]
  PassedOnly,
  #[serde(untagged)]
  Bool(bool),
}

impl Display for Silent {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Silent::PassedOnly => write!(f, "\"passed-only\""),
      Silent::Bool(val) => write!(f, "{:?}", val),
    }
  }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Environment {
  Node,
  Jsdom,
  HappyDom,
  EdgeRuntime,
  #[serde(untagged)]
  Other(String),
}

impl Display for Environment {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Environment::Node => write!(f, "node"),
      Environment::Jsdom => write!(f, "jsdom"),
      Environment::HappyDom => write!(f, "happy-dom"),
      Environment::EdgeRuntime => write!(f, "edge-runtime"),
      Environment::Other(val) => write!(f, "{val}"),
    }
  }
}
