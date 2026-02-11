pub(crate) use rust_manifest::*;

use crate::{
	custom_templating::TemplatingPresetReference,
	init_repo::gitignore::{GitIgnoreRef, GitignorePreset},
	licenses::License,
	*,
};
use toml_edit::{Array, Decor, DocumentMut, Item, Table};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Merge, Default)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default)]
pub struct RustPresets {
	/// A map that contains presets for `Cargo.toml` files.
	pub manifest: IndexMap<String, CargoTomlPreset>,

	#[serde(rename = "crate")]
	pub crate_: IndexMap<String, Crate>,
}

#[derive(Args, Clone, Debug, Serialize, Deserialize, PartialEq, Merge, Default)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[group(id = "crate_config")]
#[serde(default)]
pub struct Crate {
	#[arg(short, long, value_parser = CargoTomlPresetRef::from_cli, default_value_t = CargoTomlPresetRef::default())]
	#[merge(with = overwrite_always)]
	pub manifest: CargoTomlPresetRef,

	#[arg(long)]
	/// Settings for the gitignore file.
	pub gitignore: Option<GitIgnoreRef>,

	#[arg(long)]
	/// A license file to generate for the new repo.
	pub license: Option<License>,

	#[arg(short = 't', long = "template", value_name = "PRESET_ID")]
	pub with_templates: Vec<TemplatingPresetReference>,
}

pub fn format_array(arr: &mut Array) {
	const MAX_INLINE_ITEMS: usize = 4;
	const MAX_INLINE_CHARS: usize = 50;

	let count = arr.len();

	let total_chars: usize = arr
		.iter()
		.map(|item| item.to_string().len())
		.sum();

	let has_tables = arr.iter().any(|item| item.is_inline_table());

	let should_expand = count > MAX_INLINE_ITEMS || total_chars > MAX_INLINE_CHARS || has_tables;

	if should_expand {
		for item in arr.iter_mut() {
			item.decor_mut().set_prefix("\n\t");
		}

		arr.set_trailing_comma(true);

		arr.set_trailing("\n");
	} else {
		arr.fmt();
	}
}

impl Crate {
	pub fn generate(
		self,
		dir: &PathBuf,
		name: Option<String>,
		config: &Config,
	) -> Result<(), GenError> {
		if dir.exists() {
			panic!("Dir exists");
		}

		create_all_dirs(dir)?;

		let name = name.unwrap_or_else(|| {
			dir.file_name()
				.expect("Empty path")
				.to_string_lossy()
				.to_string()
		});

		let CargoTomlPresetRef::Config(CargoTomlPreset {
			config: mut manifest,
			..
		}) = self.manifest
		else {
			panic!("Unresolved manifest");
		};

		manifest.package.get_or_insert_default().name = Some(name);

		let manifest_is_virtual = manifest.workspace.is_some();

		let workspace_manifest_path = PathBuf::from("Cargo.toml");

		let workspace_manifest = if !manifest_is_virtual && workspace_manifest_path.exists() {
			let workspace_manifest_raw = read_to_string(&workspace_manifest_path).map_err(|e| {
				GenError::DeserializationError {
					file: workspace_manifest_path.clone(),
					error: e.to_string(),
				}
			})?;

			let mut workspace_manifest_content = workspace_manifest_raw
				.parse::<DocumentMut>()
				.map_err(|e| GenError::DeserializationError {
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

			members.set_trailing_comma(true);

			let decor = members
				.get(0)
				.map(|i| i.decor().clone())
				.unwrap_or_else(|| Decor::new("\n\t", ""));

			if decor
				.prefix()
				.is_some_and(|p| p.as_str().unwrap().contains("\n"))
			{
				members.set_trailing("\n");
			}

			let mut new_member: toml_edit::Value = dir.to_string_lossy().to_string().into();

			*new_member.decor_mut() = decor;

			members.push(new_member);

			write_file(
				&workspace_manifest_path,
				&workspace_manifest_content.to_string(),
				true,
			)?;

			let workspace_manifest_full: Manifest = toml::from_str(&workspace_manifest_raw)
				.map_err(|e| GenError::DeserializationError {
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

				inherit_opt!(edition, license, repository);

				if !workspace_package_config.keywords.is_empty()
					&& package_config.keywords.is_default()
				{
					package_config.keywords = Inheritable::Workspace { workspace: true };
				}
			}
		}

		write_file(
			&dir.join("Cargo.toml"),
			&manifest.as_document().to_string(),
			true,
		)?;

		if let Some(GitIgnoreRef::Config(gitignore)) = self.gitignore {
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

impl Crate {
	pub fn process_data(
		mut self,
		manifests_store: &IndexMap<String, CargoTomlPreset>,
		gitignore_store: &IndexMap<String, GitignorePreset>,
	) -> Result<Self, GenError> {
		let mut manifest_id: Option<String> = None;

		if let CargoTomlPresetRef::Id(id) = self.manifest {
			manifest_id = Some(id.clone());

			let data = manifests_store
				.get(&id)
				.ok_or_else(|| GenError::PresetNotFound {
					kind: Preset::CargoToml,
					name: id,
				})?
				.clone();

			self.manifest = CargoTomlPresetRef::Config(data);
		}

		if let CargoTomlPresetRef::Config(data) = self.manifest {
			self.manifest = CargoTomlPresetRef::Config(data.process_data(
				manifest_id.as_deref().unwrap_or("__inlined"),
				manifests_store,
			)?);
		}

		let mut gitignore_id: Option<String> = None;

		if let Some(GitIgnoreRef::Id(id)) = self.gitignore {
			gitignore_id = Some(id.clone());

			let data = gitignore_store
				.get(&id)
				.ok_or_else(|| GenError::PresetNotFound {
					kind: Preset::Gitignore,
					name: id,
				})?
				.clone();

			self.gitignore = Some(GitIgnoreRef::Config(data));
		}

		if let Some(GitIgnoreRef::Config(data)) = self.gitignore {
			let resolved = data.process_data(
				gitignore_id.as_deref().unwrap_or("__inlined"),
				gitignore_store,
			)?;

			self.gitignore = Some(GitIgnoreRef::Config(resolved));
		}

		Ok(self)
	}
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
pub enum CargoTomlPresetRef {
	Id(String),
	Config(CargoTomlPreset),
}

impl std::fmt::Display for CargoTomlPresetRef {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Id(id) => write!(f, "{id}"),
			Self::Config(_) => write!(f, "default config"),
		}
	}
}

impl Default for CargoTomlPresetRef {
	fn default() -> Self {
		Self::Config(CargoTomlPreset::default())
	}
}

impl CargoTomlPresetRef {
	pub fn from_cli(str: &str) -> Result<Self, String> {
		Ok(Self::Id(str.to_string()))
	}
}

/// A preset for a `Cargo.toml` file.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default, Merge)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default)]
pub struct CargoTomlPreset {
	/// The list of extended presets.
	pub extends_presets: IndexSet<String>,

	#[serde(flatten)]
	pub config: Manifest,
}

impl Extensible for CargoTomlPreset {
	fn get_extended(&self) -> &IndexSet<String> {
		&self.extends_presets
	}
}

impl CargoTomlPreset {
	pub fn process_data(self, id: &str, store: &IndexMap<String, Self>) -> Result<Self, GenError> {
		if self.extends_presets.is_empty() {
			return Ok(self);
		}

		let mut processed_ids: IndexSet<String> = IndexSet::new();

		let merged_preset = merge_presets(Preset::CargoToml, id, self, store, &mut processed_ids)?;

		Ok(merged_preset)
	}
}
