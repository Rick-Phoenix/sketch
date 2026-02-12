use super::*;

pub(crate) fn extract_config_from_file(config_file_abs_path: &Path) -> Result<Config, AppError> {
	let extension = get_extension(config_file_abs_path)?;

	let mut config: Config = if extension == "yaml" || extension == "yml" {
		deserialize_yaml(config_file_abs_path)?
	} else if extension == "toml" {
		deserialize_toml(config_file_abs_path)?
	} else if extension == "json" {
		deserialize_json(config_file_abs_path)?
	} else {
		return Err(AppError::DeserializationError {
			file: config_file_abs_path.to_path_buf(),
			error: format!(
				"Invalid config format for `{}`. Allowed formats are: yaml, toml, json",
				config_file_abs_path.display()
			),
		});
	};

	config.config_file = Some(config_file_abs_path.to_path_buf());

	let config_parent_dir = get_parent_dir(config_file_abs_path)?;

	if let Some(templates_dir) = &config.templates_dir {
		let templates_dir = config_parent_dir.join(templates_dir);

		create_all_dirs(&templates_dir)?;

		// Convert to absolute path
		config.templates_dir = Some(get_abs_path(&templates_dir)?);
	}

	Ok(config)
}

impl Config {
	/// Extracts a config from a file.
	pub fn from_file<T: Into<PathBuf> + Clone>(config_file: T) -> Result<Self, AppError> {
		let config_file_path = config_file.into();

		let config_file_abs = get_abs_path(&config_file_path)?;

		let mut config = extract_config_from_file(&config_file_abs)?;

		if !config.extends.is_empty() {
			config = config.merge_config_files()?;
		}

		Ok(config)
	}
}
