use askama::Template;
use convert_case::{Case, Casing};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VitestConfig {
  Boolean(bool),
  Named(String),
  Definition(VitestConfigStruct),
}

impl Default for VitestConfig {
  fn default() -> Self {
    Self::Definition(Default::default())
  }
}

#[derive(Debug, Template, Serialize, Deserialize)]
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
