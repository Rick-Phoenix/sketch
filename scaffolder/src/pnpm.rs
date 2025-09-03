use std::collections::BTreeMap;

use askama::Template;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{versions::get_latest_version, VersionRange};

#[derive(Debug, Template, Serialize, Deserialize)]
#[template(path = "pnpm-workspace.yaml.j2")]
pub struct PnpmWorkspaceTemplate {
  pub catalog: Option<BTreeMap<String, String>>,
  pub packages: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PnpmWorkspaceStruct {
  pub catalog: Option<BTreeMap<String, String>>,
  pub packages: Vec<String>,
  #[serde(flatten)]
  pub extra: BTreeMap<String, Value>,
}

impl PnpmWorkspaceStruct {
  pub async fn add_to_catalog(
    &mut self,
    range_kind: VersionRange,
    entries: &BTreeMap<String, String>,
  ) {
    if self.catalog.is_none() {
      self.catalog = Some(BTreeMap::new());
    }
    for (name, version) in entries {
      if version == ":catalog" && !self.catalog.as_mut().unwrap().contains_key(name.as_str()) {
        let version = get_latest_version(name)
          .await
          .unwrap_or_else(|_| "latest".to_string());
        let range = range_kind.create(version);
        self.catalog.as_mut().unwrap().insert(name.clone(), range);
      }
    }
  }
}
