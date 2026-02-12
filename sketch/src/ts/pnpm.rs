pub(crate) use pnpm_config::*;

use crate::{
	ts::{package_json::*, *},
	versions::*,
	*,
};

/// A preset for a `pnpm-workspace.yaml` file configuration.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default, Eq, Merge)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default)]
pub struct PnpmPreset {
	/// The list of extended presets.
	pub extends_presets: IndexSet<String>,

	#[serde(flatten)]
	pub config: PnpmWorkspace,
}

impl ExtensiblePreset for PnpmPreset {
	fn kind() -> PresetKind {
		PresetKind::PnpmWorkspace
	}

	fn get_extended_ids(&self) -> &IndexSet<String> {
		&self.extends_presets
	}
}

#[cfg(feature = "npm-version")]
/// A helper to add all [`PackageJson`] dependencies (dev, optional, peer, etc) marked with `catalog:` to the pnpm catalogs.
pub async fn add_dependencies_to_catalog(
	config: &mut PnpmWorkspace,
	range_kind: VersionRange,
	package_json: &PackageJson,
) -> Result<(), GenError> {
	let names_to_add: Vec<(String, Option<String>)> = package_json
		.dependencies
		.iter()
		.chain(package_json.dev_dependencies.iter())
		.chain(package_json.peer_dependencies.iter())
		.chain(package_json.optional_dependencies.iter())
		.filter_map(|(name, version)| match CATALOG_REGEX.captures(version) {
			Some(captures) => {
				let catalog_name = captures
					.name("name")
					.map(|n| n.as_str().to_string());
				Some((name.clone(), catalog_name))
			}
			None => None,
		})
		.collect();

	add_names_to_catalog(config, range_kind, names_to_add).await
}

#[cfg(feature = "npm-version")]
/// A helper to add several dependencies to one of this config's catalog.
async fn add_names_to_catalog(
	config: &mut PnpmWorkspace,
	range_kind: VersionRange,
	entries: Vec<(String, Option<String>)>,
) -> Result<(), GenError> {
	use futures::{StreamExt, stream};

	let handles = entries
		.into_iter()
		.map(|(name, catalog_name)| async move {
			let actual_latest = npm_version::get_latest_npm_version(&name).await?;

			Ok((name, catalog_name, actual_latest))
		});

	let stream = stream::iter(handles).buffer_unordered(10);

	#[allow(clippy::type_complexity)]
	let results: Vec<Result<(String, Option<String>, String), GenError>> = stream.collect().await;

	for result in results {
		match result {
			Ok((name, catalog_name, actual_latest)) => {
				let target_catalog = if let Some(catalog_name) = catalog_name {
					config
						.catalogs
						.entry(catalog_name.as_str().to_string())
						.or_default()
				} else {
					&mut config.catalog
				};

				let range = range_kind.create(actual_latest);

				target_catalog.insert(name.clone(), range);
			}
			Err(e) => return Err(e),
		};
	}

	Ok(())
}
