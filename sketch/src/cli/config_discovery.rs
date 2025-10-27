use std::{env, fs::exists, path::PathBuf};

use merge::Merge;

use crate::{
  cli::{Commands, ConfigOverrides},
  Config, GenError,
};

pub(crate) async fn get_config_from_cli(
  overrides: ConfigOverrides,
  command: &Commands,
) -> Result<Config, GenError> {
  let ConfigOverrides {
    templates_dir,
    no_overwrite,
    config: config_path,
    ignore_config,
  } = overrides;

  let mut config = Config::default();

  let config_path = if let Some(path) = config_path {
    Some(path)
  } else if !ignore_config {
    get_config_path_from_defaults()
  } else {
    None
  };

  if let Some(config_path) = config_path {
    config.merge(Config::from_file(&config_path)?);
  }

  if let Some(templates_dir) = templates_dir {
    config.templates_dir = Some(templates_dir);
  }

  if no_overwrite {
    config.no_overwrite = Some(true);
  }

  if let Commands::Ts {
    typescript_overrides,
    ..
  } = command
  {
    let typescript = config.typescript.get_or_insert_default();

    if let Some(typescript_overrides) = typescript_overrides {
      typescript.merge(typescript_overrides.clone());
    }
  };

  Ok(config)
}

const DEFAULT_CONFIG_NAMES: [&str; 3] = ["sketch.yaml", "sketch.toml", "sketch.json"];

fn get_config_path_from_defaults() -> Option<PathBuf> {
  for name in DEFAULT_CONFIG_NAMES {
    if exists(name).is_ok_and(|exists| exists) {
      return Some(PathBuf::from(name));
    }
  }

  // Try xdg path if nothing else was found
  get_config_from_xdg()
}

fn get_config_from_xdg() -> Option<PathBuf> {
  let xdg_config = if let Ok(env_val) = env::var("XDG_CONFIG_HOME") {
    Some(PathBuf::from(env_val))
  } else {
    env::home_dir().map(|home| home.join(".config"))
  };

  if let Some(xdg_config) = xdg_config {
    let config_dir = xdg_config.join("sketch");

    if config_dir.is_dir() {
      for name in DEFAULT_CONFIG_NAMES {
        let config_path = config_dir.join(name);
        if exists(&config_path).is_ok_and(|exists| exists) {
          return Some(config_path);
        }
      }
    }
  }
  None
}
