use futures::{StreamExt, stream};
use reqwest::Client;
use url::Url;

use crate::{ts::package_json::JsDepKind, *};

/// The kinds of version ranges for a dependency with semantic versioning.
#[derive(Debug, Deserialize, Serialize, Default, Clone, Copy, ValueEnum, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum VersionRange {
	// Only allow updates within the same semVer `minor` range (i.e. 1.x.x)
	#[default]
	Minor,

	// Only allow updates within the same semVer `patch` range (i.e. 1.0.x)
	Patch,

	// Use the exact version given
	Exact,
}

impl VersionRange {
	/// Takes a version and appends the range prefix to it.
	pub fn create(&self, version: String) -> String {
		if version.starts_with("catalog:") || version == "latest" {
			return version;
		}
		match self {
			Self::Patch => format!("~{version}"),
			Self::Minor => format!("^{version}"),
			Self::Exact => version,
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
	let url_str = format!("https://registry.npmjs.org/{package_name}/latest");
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
		.map_err(|e| {
			generic_error!("Could not get the latest version for `{package_name}`: {e}")
		})?;

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
