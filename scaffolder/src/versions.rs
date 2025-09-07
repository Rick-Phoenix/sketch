use std::{collections::BTreeMap, sync::LazyLock};

use futures::future::join_all;
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

/// A helper to get the latest version of multiple npm packages.
pub async fn get_latest_versions(names: Vec<String>) -> BTreeMap<String, String> {
  let mut tasks = Vec::new();

  for name in names {
    let name_clone = name.clone();

    let task = tokio::spawn(async move {
      let versions_result = get_latest_version(&name_clone).await;

      match versions_result {
        Ok(latest) => Ok((name_clone, latest)),
        Err(e) => {
          eprintln!("Error fetching versions for '{}': {:?}", name_clone, e);
          Ok((name_clone, "".to_string()))
        }
      }
    });
    tasks.push(task);
  }

  let results: Vec<Result<(String, String), String>> = join_all(tasks)
    .await
    .into_iter()
    .map(|res| res.map_err(|e| format!("Task join error: {}", e))?)
    .collect();

  let mut latest_versions = BTreeMap::new();
  for result in results {
    match result {
      Ok((name, version)) => {
        latest_versions.insert(name, version);
      }

      Err(e) => {
        eprintln!("Could not get latest version for package '{}':", e);
      }
    }
  }

  latest_versions
}
