use std::collections::BTreeMap;

use askama::Template;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{
  versions::get_latest_version, PackageJson, StringKeyVal, StringKeyValMap, VersionRange,
};

#[derive(Debug, Template, Serialize, Deserialize, Default)]
#[serde(default)]
#[template(path = "pnpm-workspace.yaml.j2")]
pub struct PnpmWorkspace {
  pub catalog: StringKeyVal,
  pub catalogs: BTreeMap<String, StringKeyVal>,
  pub packages: Vec<String>,
  #[serde(flatten)]
  pub extra: StringKeyValMap,
}

impl PnpmWorkspace {
  pub async fn add_dependencies_to_catalog(
    &mut self,
    range_kind: VersionRange,
    package_json: &PackageJson,
  ) {
    self
      .add_to_catalog(range_kind, &package_json.dependencies)
      .await;
    self
      .add_to_catalog(range_kind, &package_json.dev_dependencies)
      .await;

    if let Some(ref peer_dependencies) = package_json.peer_dependencies {
      self.add_to_catalog(range_kind, peer_dependencies).await;
    }

    if let Some(ref optional_dependencies) = package_json.optional_dependencies {
      self.add_to_catalog(range_kind, optional_dependencies).await;
    }

    if let Some(ref bundle_dependencies) = package_json.bundle_dependencies {
      self.add_to_catalog(range_kind, bundle_dependencies).await;
    }
  }

  pub async fn add_to_catalog(
    &mut self,
    range_kind: VersionRange,
    entries: &BTreeMap<String, String>,
  ) {
    let catalog_regex = Regex::new(r"^catalog:(?<name>\w+)?$").unwrap();

    for (name, version) in entries {
      if let Some(captures) = catalog_regex.captures(version) {
        let catalog_name = captures.name("name");

        let target_catalog = if let Some(name) = catalog_name {
          self.catalogs.entry(name.as_str().to_string()).or_default()
        } else {
          &mut self.catalog
        };

        let version = get_latest_version(name)
          .await
          .unwrap_or_else(|_| "latest".to_string());
        let range = range_kind.create(version);

        target_catalog.insert(name.to_string(), range);
      }
    }
  }
}
