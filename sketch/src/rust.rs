pub(crate) use rust_manifest::*;

use crate::{TemplatingPresetRef, init_repo::gitignore::GitIgnorePresetRef, licenses::License, *};
use toml_edit::{Array, Decor, DocumentMut, Item, Table};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Merge, Default)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct RustConfig {
	/// A map that contains presets for `Cargo.toml` files.
	pub manifest_presets: IndexMap<String, CargoTomlPreset>,

	pub crate_presets: IndexMap<String, CratePreset>,
}

#[derive(Args, Clone, Debug, Serialize, Deserialize, PartialEq, Merge, Default)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[group(id = "input")]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct CratePreset {
	#[arg(skip)]
	#[merge(with = overwrite_always)]
	pub manifest: CargoTomlPresetRef,

	#[arg(long)]
	/// Settings for the gitignore file.
	pub gitignore: Option<GitIgnorePresetRef>,

	#[arg(long)]
	/// A license file to generate for the new repo.
	pub license: Option<License>,

	#[arg(short = 't', long = "template", value_name = "PRESET_ID")]
	pub with_templates: Vec<TemplatingPresetRef>,
}

impl CratePreset {
	pub fn generate(
		self,
		dir: &PathBuf,
		name: Option<String>,
		config: &Config,
	) -> Result<(), AppError> {
		create_all_dirs(dir)?;

		let mut manifest = match self.manifest {
			CargoTomlPresetRef::Preset(CargoTomlPreset { config, .. }) => config,
			CargoTomlPresetRef::PresetId(id) => {
				return Err(anyhow!("Unresolved manifest preset with id `{id}`").into());
			}
		};

		let manifest_is_virtual = manifest.workspace.is_some();

		if !manifest_is_virtual {
			let name = name.unwrap_or_else(|| {
				dir.file_name()
					.expect("Empty path")
					.to_string_lossy()
					.to_string()
			});
			manifest.package.get_or_insert_default().name = Some(name);
		}

		let workspace_manifest_path = get_parent_dir(dir)?.join("Cargo.toml");

		let workspace_manifest = if !manifest_is_virtual && workspace_manifest_path.exists() {
			let workspace_manifest_raw = read_to_string(&workspace_manifest_path).map_err(|e| {
				AppError::DeserializationError {
					file: workspace_manifest_path.clone(),
					error: e.to_string(),
				}
			})?;

			let mut workspace_manifest_content = workspace_manifest_raw
				.parse::<DocumentMut>()
				.map_err(|e| AppError::DeserializationError {
					file: workspace_manifest_path.clone(),
					error: e.to_string(),
				})?;

			let workspace_entry = workspace_manifest_content
				.entry("workspace")
				.or_insert_with(|| Item::Table(Table::new()));

			let members = workspace_entry
				.as_table_mut()
				.unwrap()
				.entry("members")
				.or_insert_with(|| Item::Value(toml_edit::Value::Array(Array::new())))
				.as_array_mut()
				.unwrap();

			let new_member: toml_edit::Value = dir
				.file_name()
				.unwrap()
				.to_string_lossy()
				.to_string()
				.into();

			members.push(new_member);

			for member in members.iter_mut() {
				*member.decor_mut() = Decor::new("\n\t", "");
			}

			members.set_trailing("\n");
			members.set_trailing_comma(true);

			write_file(
				&workspace_manifest_path,
				&workspace_manifest_content.to_string(),
				true,
			)?;

			let workspace_manifest_full: Manifest = toml::from_str(&workspace_manifest_raw)
				.map_err(|e| AppError::DeserializationError {
					file: workspace_manifest_path.clone(),
					error: e.to_string(),
				})?;

			workspace_manifest_full.workspace
		} else {
			None
		};

		if let Some(workspace_manifest) = workspace_manifest {
			if workspace_manifest.lints.is_some() && manifest.lints.is_none() {
				manifest.lints = Some(Inheritable::Workspace { workspace: true });
			}

			if let Some(workspace_package_config) = &workspace_manifest.package {
				let package_config = manifest.package.get_or_insert_default();

				macro_rules! inherit_opt {
					($($name:ident),*) => {
						$(
							if workspace_package_config.$name.is_some() && package_config.$name.is_none() {
								package_config.$name = Some(Inheritable::Workspace {
									workspace: true,
								});
							}
						)*
					};
				}

				inherit_opt!(
					edition,
					license,
					homepage,
					rust_version,
					description,
					readme,
					documentation,
					publish,
					version,
					repository
				);

				macro_rules! inherit_list_opt {
					($($name:ident),*) => {
						$(
							if !workspace_package_config.$name.is_empty() && package_config.$name.is_default() {
								package_config.$name = Inheritable::Workspace {
									workspace: true,
								};
							}
						)*
					};
				}

				inherit_list_opt!(keywords, categories, exclude, include);
			}
		}

		write_file(
			&dir.join("Cargo.toml"),
			&manifest.as_document().to_string(),
			true,
		)?;

		if let Some(GitIgnorePresetRef::Preset(gitignore)) = self.gitignore {
			write_file(
				&dir.join(".gitignore"),
				&gitignore.content.to_string(),
				true,
			)?;
		}

		if let Some(license) = self.license {
			write_file(&dir.join("LICENSE"), license.get_content(), true)?;
		}

		if !self.with_templates.is_empty() {
			config.generate_templates(dir, self.with_templates, &Default::default())?;
		}

		Ok(())
	}
}

impl RustConfig {
	pub fn get_crate_preset(&self, id: &str) -> AppResult<CratePreset> {
		Ok(self
			.crate_presets
			.get(id)
			.ok_or_else(|| AppError::PresetNotFound {
				kind: PresetKind::RustCrate,
				name: id.to_string(),
			})?
			.clone())
	}

	pub fn get_cargo_toml_preset(&self, id: &str) -> AppResult<CargoTomlPreset> {
		self.manifest_presets
			.get(id)
			.ok_or_else(|| AppError::PresetNotFound {
				kind: PresetKind::CargoToml,
				name: id.to_string(),
			})?
			.clone()
			.merge_presets(id, &self.manifest_presets)
	}
}

impl CratePreset {
	pub fn process_data(mut self, config: &Config) -> Result<Self, AppError> {
		if !self.manifest.is_preset_id()
			&& self
				.gitignore
				.as_ref()
				.is_none_or(|g| !g.is_preset_id())
		{
			return Ok(self);
		}

		self.manifest = {
			let preset = match self.manifest {
				CargoTomlPresetRef::PresetId(id) => config.rust.get_cargo_toml_preset(&id)?,
				CargoTomlPresetRef::Preset(preset) => {
					preset.merge_presets("__inlined", &config.rust.manifest_presets)?
				}
			};

			CargoTomlPresetRef::Preset(preset)
		};

		if let Some(preset_ref) = self.gitignore {
			let preset = match preset_ref {
				GitIgnorePresetRef::PresetId(id) => config.get_gitignore_preset(&id)?,
				GitIgnorePresetRef::Preset(preset) => {
					preset.merge_presets("__inlined", &config.gitignore_presets)?
				}
			};

			self.gitignore = Some(GitIgnorePresetRef::Preset(preset));
		}

		Ok(self)
	}
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
pub enum CargoTomlPresetRef {
	PresetId(String),
	Preset(CargoTomlPreset),
}

impl CargoTomlPresetRef {
	/// Returns `true` if the cargo toml preset ref is [`PresetId`].
	///
	/// [`PresetId`]: CargoTomlPresetRef::PresetId
	#[must_use]
	pub const fn is_preset_id(&self) -> bool {
		matches!(self, Self::PresetId(..))
	}
}

impl Default for CargoTomlPresetRef {
	fn default() -> Self {
		Self::Preset(CargoTomlPreset::default())
	}
}

/// A preset for a `Cargo.toml` file.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default, Merge)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default)]
pub struct CargoTomlPreset {
	/// The list of extended presets.
	#[merge(skip)]
	pub extends_presets: IndexSet<String>,

	#[serde(flatten)]
	pub config: Manifest,
}

impl ExtensiblePreset for CargoTomlPreset {
	fn kind() -> PresetKind {
		PresetKind::CargoToml
	}

	fn extended_ids(&mut self) -> &mut IndexSet<String> {
		&mut self.extends_presets
	}
}
