use std::sync::LazyLock;

use clap::ValueEnum;
use reqwest::Client;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::{ParseError, Url};

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

/// Errors occurring when fetching the latest version for an npm package.
#[derive(Debug, Error)]
pub enum NpmVersionError {
  #[error("An invalid url was used: {source}")]
  InvalidUrl { source: ParseError },
  #[error(transparent)]
  ReqwestError(#[from] reqwest::Error),
}

static CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

/// A helper to get the latest version of an npm package.
pub async fn get_latest_npm_version(package_name: &str) -> Result<String, NpmVersionError> {
  let url_str = format!("https://registry.npmjs.org/{}/latest", package_name);
  let url = Url::parse(&url_str).map_err(|e| NpmVersionError::InvalidUrl { source: e })?;

  let response = CLIENT
    .get(url)
    .send()
    .await?
    .json::<NpmApiResponse>()
    .await?;

  Ok(response.version)
}
