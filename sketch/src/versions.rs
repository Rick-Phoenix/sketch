use std::sync::LazyLock;

use clap::ValueEnum;
use futures::{stream, StreamExt};
use reqwest::Client;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{ts::package_json::JsDepKind, GenError};

/// The kinds of version ranges for a dependency with semantic versioning.
#[derive(Debug, Deserialize, Serialize, Default, Clone, Copy, ValueEnum, PartialEq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum VersionRange {
  Patch,
  #[default]
  Minor,
  Exact,
}

impl VersionRange {
  /// Takes a version and appends the range prefix to it.
  pub fn create(&self, version: String) -> String {
    if version.starts_with("catalog:") || version == "latest" {
      return version;
    }
    match self {
      VersionRange::Patch => format!("~{}", version),
      VersionRange::Minor => format!("^{}", version),
      VersionRange::Exact => version,
    }
  }
}

#[derive(Debug, serde::Deserialize)]
struct NpmApiResponse {
  version: String,
}

static CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

/// A helper to get the latest version of an npm package.
pub async fn get_latest_npm_version(package_name: &str) -> Result<String, GenError> {
  let url_str = format!("https://registry.npmjs.org/{}/latest", package_name);
  let url = Url::parse(&url_str).map_err(|e| {
    generic_error!(
      "Could not get the latest version for `{package_name}` due to an invalid URL: {e}"
    )
  })?;

  let response = CLIENT
    .get(url)
    .send()
    .await
    .map_err(|e| generic_error!("Could not get the latest version for `{package_name}`: {e}"))?
    .json::<NpmApiResponse>()
    .await
    .map_err(|e| generic_error!("Could not get the latest version for `{package_name}`: {e}"))?;

  Ok(response.version)
}

pub async fn get_batch_latest_npm_versions(
  deps: Vec<(JsDepKind, String)>,
) -> Vec<Result<(JsDepKind, String, String), GenError>> {
  let handles = deps.into_iter().map(|(kind, name)| async move {
    let actual_latest = get_latest_npm_version(&name).await?;

    Ok((kind, name, actual_latest))
  });

  let stream = stream::iter(handles).buffer_unordered(10);

  let results: Vec<Result<(JsDepKind, String, String), GenError>> = stream.collect().await;

  results
}
