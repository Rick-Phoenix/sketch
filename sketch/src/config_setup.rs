use std::{
  fs::File,
  path::{Path, PathBuf},
};

use figment::{
  providers::{Format, Json, Toml, Yaml},
  Figment,
};

use crate::{
  get_abs_path,
  paths::{create_parent_dirs, get_parent_dir},
  Config, GenError,
};

pub(crate) fn extract_config_from_file(config_file_abs: &Path) -> Result<Config, GenError> {
  File::open(config_file_abs).map_err(|e| GenError::ReadError {
    path: config_file_abs.to_path_buf(),
    source: e,
  })?;

  let extension = config_file_abs.extension().unwrap_or_else(|| {
    panic!(
      "Config file '{}' has no extension.",
      config_file_abs.display()
    )
  });

  let figment = if extension == "yaml" || extension == "yml" {
    Figment::from(Yaml::file(&config_file_abs))
  } else if extension == "toml" {
    Figment::from(Toml::file(&config_file_abs))
  } else if extension == "json" {
    Figment::from(Json::file(&config_file_abs))
  } else {
    return Err(GenError::InvalidConfigFormat {
      file: config_file_abs.to_path_buf(),
    });
  };

  let mut config: Config = figment
    .extract()
    .map_err(|e| GenError::ConfigParsing { source: e })?;

  config.config_file = Some(config_file_abs.to_path_buf());

  let parent_dir = &get_parent_dir(config_file_abs);

  if let Some(templates_dir) = &config.templates_dir {
    let templates_dir = parent_dir.join(templates_dir);

    create_parent_dirs(&templates_dir)?;

    config.templates_dir = Some(get_abs_path(&templates_dir)?);
  }

  if let Some(root_dir) = &config.root_dir {
    let root_dir = parent_dir.join(root_dir);

    create_parent_dirs(&root_dir)?;

    config.root_dir = Some(get_abs_path(&root_dir)?);
  }

  Ok(config)
}

impl Config {
  pub fn from_file<T: Into<PathBuf> + Clone>(config_file: T) -> Result<Self, GenError> {
    let config_file_path = config_file.into();

    let config_file_abs: PathBuf = get_abs_path(&config_file_path)?;

    let mut config = extract_config_from_file(&config_file_abs)?;

    if !config.extends.is_empty() {
      config = config.merge_config_files()?;
    }

    Ok(config)
  }
}
