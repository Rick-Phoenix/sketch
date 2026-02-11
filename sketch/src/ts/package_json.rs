use super::*;
pub(crate) use ::package_json::*;

impl Extensible for PackageJsonPreset {
	fn get_extended(&self) -> &IndexSet<String> {
		&self.extends_presets
	}
}

fn get_person_data(id: &str, store: &IndexMap<String, PersonData>) -> Option<PersonData> {
	store.get(id).cloned()
}

impl PackageJsonPreset {
	pub fn process_data(
		self,
		current_id: &str,
		store: &IndexMap<String, Self>,
		people: &IndexMap<String, PersonData>,
	) -> Result<PackageJson, GenError> {
		let merged_preset = if self.extends_presets.is_empty() {
			self
		} else {
			let mut processed_ids: IndexSet<String> = IndexSet::new();
			merge_presets(
				Preset::PackageJson,
				current_id,
				self,
				store,
				&mut processed_ids,
			)?
		};

		let mut package_json = merged_preset.config;

		package_json.contributors = package_json
			.contributors
			.into_iter()
			.map(|person| {
				if let Person::Id(ref id) = person
					&& let Some(data) = get_person_data(id, people)
				{
					Person::Data(data)
				} else {
					person
				}
			})
			.collect();

		package_json.maintainers = package_json
			.maintainers
			.into_iter()
			.map(|person| {
				if let Person::Id(ref id) = person
					&& let Some(data) = get_person_data(id, people)
				{
					Person::Data(data)
				} else {
					person
				}
			})
			.collect();

		if let Some(author) = package_json.author.as_mut()
			&& let Person::Id(id) = author
			&& let Some(data) = get_person_data(id.as_str(), people)
		{
			*author = Person::Data(data);
		};

		Ok(package_json)
	}
}

/// A [`PackageJson`] preset.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Merge, Default)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default)]
pub struct PackageJsonPreset {
	/// The list of extended presets.
	pub extends_presets: IndexSet<String>,
	#[serde(flatten)]
	pub config: PackageJson,
}

/// Ways of indicating [`PackageJson`] data. It can be an id, pointing to a preset, or a literal configuration.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
pub enum PackageJsonData {
	Id(String),
	Config(PackageJsonPreset),
}

impl Default for PackageJsonData {
	fn default() -> Self {
		Self::Config(PackageJsonPreset::default())
	}
}

impl PackageJsonData {
	pub(crate) fn from_cli(s: &str) -> Result<Self, String> {
		Ok(Self::Id(s.trim().to_string()))
	}
}

#[cfg(feature = "npm-version")]
/// Converts dependencies marked with `latest` into a version range starting from the latest version fetched with the npm API.
pub async fn process_package_json_dependencies(
	config: &mut PackageJson,
	package_manager: PackageManager,
	convert_latest: bool,
	range_kind: VersionRange,
) -> Result<(), GenError> {
	let is_bun = matches!(package_manager, PackageManager::Bun);

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

				match kind {
					JsDepKind::CatalogDependency(catalog_name) => {
						let target_catalog = if let Some(catalog_name) = catalog_name {
							config
								.catalogs
								.entry(catalog_name.as_str().to_string())
								.or_default()
						} else {
							&mut config.catalog
						};

						target_catalog.insert(name, new_version_range);
					}
					JsDepKind::Dependency => {
						config
							.dependencies
							.insert(name, new_version_range);
					}
					JsDepKind::DevDependency => {
						config
							.dev_dependencies
							.insert(name, new_version_range);
					}
					JsDepKind::OptionalDependency => {
						config
							.optional_dependencies
							.insert(name, new_version_range);
					}
					JsDepKind::PeerDependency => {
						config
							.peer_dependencies
							.insert(name, new_version_range);
					}
				}
			}
			Err(task_error) => return Err(task_error),
		};
	}

	Ok(())
}
