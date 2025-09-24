use std::{
  env,
  fs::{exists, read_dir},
  path::PathBuf,
};

use merge::Merge;

use crate::{
  cli::{Cli, Commands},
  Config, GenError,
};

fn get_config_file_path(cli_arg: Option<PathBuf>) -> Option<PathBuf> {
  if let Some(cli_arg) = cli_arg {
    Some(cli_arg)
  } else if exists("sketch.yaml").is_ok_and(|exists| exists) {
    Some(PathBuf::from("sketch.yaml"))
  } else if exists("sketch.toml").is_ok_and(|exists| exists) {
    Some(PathBuf::from("sketch.toml"))
  } else if exists("sketch.json").is_ok_and(|exists| exists) {
    Some(PathBuf::from("sketch.json"))
  } else {
    None
  }
}

fn get_config_from_xdg() -> Option<PathBuf> {
  let xdg_config = if let Ok(env_val) = env::var("XDG_CONFIG_HOME") {
    Some(PathBuf::from(env_val))
  } else if let Some(home) = env::home_dir() {
    Some(home.join(".config"))
  } else {
    None
  };

  if let Some(xdg_config) = xdg_config {
    let config_dir = PathBuf::from(xdg_config).join("sketch");

    if config_dir.is_dir() {
      if let Ok(dir_contents) = read_dir(&config_dir) {
        for item in dir_contents {
          if let Ok(item) = item {
            if item.file_name() == "sketch.toml"
              || item.file_name() == "sketch.yaml"
              || item.file_name() == "sketch.json"
            {
              return Some(item.path());
            }
          }
        }
      }
    }
  }
  None
}

pub(crate) async fn get_config_from_cli(cli: Cli) -> Result<Config, GenError> {
  let mut config = Config::default();

  if !cli.ignore_config {
    let config_path = if let Some(config_file) = get_config_file_path(cli.config) {
      Some(config_file)
    } else if let Some(config_from_xdg) = get_config_from_xdg() {
      Some(config_from_xdg)
    } else {
      None
    };

    if let Some(config_path) = config_path {
      if config.debug {
        eprintln!("Found config file `{}`", config_path.display());
      }
      config.merge(Config::from_file(&config_path)?);
    }
  } else if config.debug {
    eprintln!("`ignore_config` detected");
  }

  if let Some(overrides) = cli.overrides {
    config.merge(overrides);
  }

  if let Some(vars) = cli.templates_vars {
    config.vars.extend(vars);
  }

  match cli.command {
    Commands::Ts {
      typescript_overrides,
      ..
    } => {
      let typescript = config.typescript.get_or_insert_default();

      if let Some(typescript_overrides) = typescript_overrides {
        typescript.merge(typescript_overrides);
      }
    }
    _ => {}
  };

  Ok(config)
}
