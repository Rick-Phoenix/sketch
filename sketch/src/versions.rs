use std::sync::LazyLock;

use reqwest::Client;
use thiserror::Error;
use url::{ParseError, Url};

#[derive(Debug, serde::Deserialize)]
struct NpmApiResponse {
  version: String,
}

/// Errors occurring when fetching the latest version for a package.
#[derive(Debug, Error)]
pub enum GetVersionError {
  #[error("An invalid url was used: {source}")]
  InvalidUrl { source: ParseError },
  #[error(transparent)]
  ReqwestError(#[from] reqwest::Error),
}

static CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

/// A helper to get the latest version of an npm package.
pub async fn get_latest_version(package_name: &str) -> Result<String, GetVersionError> {
  let url_str = format!("https://registry.npmjs.org/{}/latest", package_name);
  let url = Url::parse(&url_str).map_err(|e| GetVersionError::InvalidUrl { source: e })?;

  let response = CLIENT
    .get(url)
    .send()
    .await?
    .json::<NpmApiResponse>()
    .await?;

  Ok(response.version)
}
