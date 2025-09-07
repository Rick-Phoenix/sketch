use askama::Template;
use convert_case::{Case, Casing};
use serde::{Deserialize, Serialize};

/// The types of configuration for generating a vitest setup.
/// Can be set to:
/// - True or false to use the default or disable generation altogether.
/// - A string, indicating a preset stored in the global config
/// - A object, with a literal definition
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VitestConfig {
  Boolean(bool),
  Id(String),
  Config(VitestConfigStruct),
}

impl Default for VitestConfig {
  fn default() -> Self {
    Self::Config(Default::default())
  }
}

/// The data used to generate a new vitest setup.
#[derive(Clone, Debug, Template, Serialize, Deserialize)]
#[template(path = "vitest.config.ts.j2")]
#[serde(default)]
pub struct VitestConfigStruct {
  pub plugins: Vec<String>,
  pub setup_dir: String,
  pub setup_files: Vec<String>,
  pub(crate) src_rel_path: String,
}

#[derive(Template)]
#[template(path = "tests_setup.ts.j2")]
pub(crate) struct TestsSetupFile;

impl Default for VitestConfigStruct {
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
