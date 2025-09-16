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
  Id(String),
  Config(VitestConfig),
}

impl Default for VitestConfigKind {
  fn default() -> Self {
    Self::Bool(true)
  }
}

/// The data used to generate a new vitest setup.
#[derive(Clone, Debug, Template, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[template(path = "vitest.config.ts.j2")]
#[serde(default)]
pub struct VitestConfig {
  pub plugins: Vec<String>,
  pub setup_dir: String,
  pub setup_files: Vec<String>,
  #[serde(skip)]
  pub(crate) src_rel_path: String,
}

#[derive(Template)]
#[template(path = "tests_setup.ts.j2")]
pub(crate) struct TestsSetupFile;

impl Default for VitestConfig {
  fn default() -> Self {
    Self {
      plugins: vec![],
      setup_dir: "setup".to_string(),
      setup_files: vec![],
      src_rel_path: "../../src".to_string(),
    }
  }
}

fn to_camel_case(string: &str) -> String {
  string.to_case(Case::Camel)
}
