use std::{
  fs::File,
  io::Read,
  path::{Path, PathBuf},
};

use crate::{
  get_abs_path,
  paths::{create_parent_dirs, get_parent_dir},
  Config, GenError,
};

pub(crate) fn extract_config_from_file(config_file_abs_path: &Path) -> Result<Config, GenError> {
  let mut config_file = File::open(config_file_abs_path).map_err(|e| GenError::ReadError {
    path: config_file_abs_path.to_path_buf(),
    source: e,
  })?;

  let extension = config_file_abs_path.extension().unwrap_or_else(|| {
    panic!(
      "Config file '{}' has no extension.",
      config_file_abs_path.display()
    )
  });

  let mut config: Config = if extension == "yaml" || extension == "yml" {
    serde_yaml_ng::from_reader(&config_file).map_err(|e| GenError::ConfigParsing(e.to_string()))?
  } else if extension == "toml" {
    let mut contents = String::new();
    config_file
      .read_to_string(&mut contents)
      .map_err(|e| GenError::ReadError {
        path: config_file_abs_path.to_path_buf(),
        source: e,
      })?;
    toml::from_str(&contents).map_err(|e| GenError::ConfigParsing(e.to_string()))?
  } else if extension == "json" {
    serde_json::from_reader(&config_file).map_err(|e| GenError::ConfigParsing(e.to_string()))?
  } else {
    return Err(GenError::InvalidConfigFormat {
      file: config_file_abs_path.to_path_buf(),
    });
  };

  config.config_file = Some(config_file_abs_path.to_path_buf());

  let parent_dir = &get_parent_dir(config_file_abs_path);

  if let Some(templates_dir) = &config.templates_dir {
    let templates_dir = parent_dir.join(templates_dir);

    create_parent_dirs(&templates_dir)?;

    config.templates_dir = Some(get_abs_path(&templates_dir)?);
  }

  if let Some(root_dir) = &config.out_dir {
    let root_dir = parent_dir.join(root_dir);

    create_parent_dirs(&root_dir)?;

    config.out_dir = Some(get_abs_path(&root_dir)?);
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
