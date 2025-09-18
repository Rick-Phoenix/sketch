use std::{
  collections::{BTreeMap, BTreeSet},
  sync::LazyLock,
};

use askama::Template;
use maplit::btreeset;
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
  versions::{get_latest_npm_version, VersionRange},
  OrderedMap, PackageJson, StringBTreeMap,
};

/// A struct representing a pnpm-workspace.yaml config. Some widely used fields such as catalog, catalogs and packages are included directly. Everything else is flattened in the `extra` field.
#[derive(Clone, Debug, Template, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(default)]
#[template(path = "pnpm-workspace.yaml.j2")]
pub struct PnpmWorkspace {
  pub catalog: StringBTreeMap,
  pub catalogs: BTreeMap<String, StringBTreeMap>,
  pub packages: BTreeSet<String>,
  #[serde(flatten)]
  pub extra: OrderedMap,
}

impl Default for PnpmWorkspace {
  fn default() -> Self {
    Self {
      catalog: Default::default(),
      catalogs: Default::default(),
      packages: btreeset! { "packages/*".to_string() },
      extra: Default::default(),
    }
  }
}

static CATALOG_REGEX: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r"^catalog:(?<name>\w+)?$").expect("Failed to initialize the catalog regex")
});

impl PnpmWorkspace {
  /// A helper to add all dependencies listed in a [`PackageJson`] (dev, optional, peer, etc) to a catalog in this configuration.
  pub async fn add_dependencies_to_catalog(
    &mut self,
    range_kind: VersionRange,
    package_json: &PackageJson,
  ) {
    let names_to_add: Vec<(String, Option<String>)> = package_json
      .dependencies
      .iter()
      .chain(package_json.dev_dependencies.iter())
      .chain(package_json.peer_dependencies.iter())
      .chain(package_json.bundle_dependencies.iter())
      .chain(package_json.optional_dependencies.iter())
      .filter_map(|(name, version)| match CATALOG_REGEX.captures(version) {
        Some(captures) => {
          let catalog_name = captures.name("name");
          Some((name.clone(), catalog_name.map(|n| n.as_str().to_string())))
        }
        None => None,
      })
      .collect();

    self.add_names_to_catalog(range_kind, names_to_add).await
  }

  /// A helper to add several dependencies to one of this config's catalog.
  pub async fn add_names_to_catalog(
    &mut self,
    range_kind: VersionRange,
    entries: Vec<(String, Option<String>)>,
  ) {
    for (name, catalog_name) in entries {
      let target_catalog = if let Some(name) = catalog_name {
        self.catalogs.entry(name.as_str().to_string()).or_default()
      } else {
        &mut self.catalog
      };

      let version = get_latest_npm_version(&name)
          .await
          .unwrap_or_else(|e| {
            eprintln!(
              "Could not get the latest valid version range for '{}' due to the following error: {}.\nFalling back to 'latest'...",
              name,
              e,
            );
            "latest".to_string()
          });
      let range = range_kind.create(version);

      target_catalog.insert(name.to_string(), range);
    }
  }

  /// A helper to add several dependencies to one of this config's catalog.
  pub async fn add_dependencies_map_to_catalog(
    &mut self,
    range_kind: VersionRange,
    entries: &BTreeMap<String, String>,
  ) {
    for (name, version) in entries {
      if let Some(captures) = CATALOG_REGEX.captures(version) {
        let catalog_name = captures.name("name");

        let target_catalog = if let Some(name) = catalog_name {
          self.catalogs.entry(name.as_str().to_string()).or_default()
        } else {
          &mut self.catalog
        };

        let version = get_latest_npm_version(name)
          .await
          .unwrap_or_else(|e| {
            eprintln!(
              "Could not get the latest valid version range for '{}' due to the following error: {}.\nFalling back to 'latest'...",
              name,
              e,
            );
            "latest".to_string()
          });
        let range = range_kind.create(version);

        target_catalog.insert(name.to_string(), range);
      }
    }
  }
}
