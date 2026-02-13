use super::*;
pub(crate) use ::package_json::*;

impl ExtensiblePreset for PackageJsonPreset {
	fn kind() -> PresetKind {
		PresetKind::PackageJson
	}

	fn extended_ids(&mut self) -> &mut IndexSet<String> {
		&mut self.extends_presets
	}
}

/// A [`PackageJson`] preset.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Merge, Default)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default)]
pub struct PackageJsonPreset {
	/// The list of extended presets.
	#[merge(skip)]
	pub extends_presets: IndexSet<String>,
	#[serde(flatten)]
	pub config: PackageJson,
}

/// Ways of indicating [`PackageJson`] data. It can be an id, pointing to a preset, or a literal configuration.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
pub enum PackageJsonPresetRef {
	PresetId(String),
	Preset(PackageJsonPreset),
}

impl Default for PackageJsonPresetRef {
	fn default() -> Self {
		Self::Preset(PackageJsonPreset::default())
	}
}

impl PackageJsonPresetRef {
	pub(crate) fn from_cli(s: &str) -> Result<Self, String> {
		Ok(Self::PresetId(s.trim().to_string()))
	}
}

#[cfg(feature = "npm-version")]
/// Converts dependencies marked with `latest` into a version range starting from the latest version fetched with the npm API.
pub async fn process_package_json_dependencies(
	config: &mut PackageJson,
	package_manager: PackageManager,
	convert_latest: bool,
	range_kind: VersionRange,
) -> Result<(), AppError> {
	let is_bun = package_manager.is_bun();

	if !convert_latest && !is_bun {
		return Ok(());
	}

	let mut names_to_update: Vec<(JsDepKind, String)> = Vec::new();

	macro_rules! get_latest {
		($list:ident, $kind:ident) => {
			for (name, version) in &config.$list {
				if convert_latest && version == "latest" {
					names_to_update.push((JsDepKind::$kind, name.clone()));
				} else if is_bun && let Some(captures) = CATALOG_REGEX.captures(version) {
					let catalog_name = captures
						.name("name")
						.map(|n| n.as_str().to_string());

					match catalog_name {
						Some(catalog_name) => {
							if !config
								.catalogs
								.get(&catalog_name)
								.is_some_and(|c| c.contains_key(name))
							{
								names_to_update.push((
									JsDepKind::CatalogDependency(Some(catalog_name)),
									name.clone(),
								));
							}
						}
						None => {
							if !config.catalog.contains_key(name) {
								names_to_update
									.push((JsDepKind::CatalogDependency(None), name.clone()));
							}
						}
					};
				}
			}
		};
	}

	get_latest!(dependencies, Dependency);
	get_latest!(dev_dependencies, DevDependency);
	get_latest!(optional_dependencies, OptionalDependency);
	get_latest!(peer_dependencies, PeerDependency);

	let results = npm_version::get_batch_latest_npm_versions(names_to_update).await;

	for result in results {
		match result {
			Ok((kind, name, actual_latest)) => {
				let new_version_range = range_kind.create(actual_latest);

				let target = match kind {
					JsDepKind::CatalogDependency(catalog_name) => {
						if let Some(catalog_name) = catalog_name {
							config.catalogs.entry(catalog_name).or_default()
						} else {
							&mut config.catalog
						}
					}
					JsDepKind::Dependency => &mut config.dependencies,
					JsDepKind::DevDependency => &mut config.dev_dependencies,
					JsDepKind::OptionalDependency => &mut config.optional_dependencies,
					JsDepKind::PeerDependency => &mut config.peer_dependencies,
				};

				target.insert(name, new_version_range);
			}
			Err(task_error) => return Err(task_error),
		};
	}

	Ok(())
}
