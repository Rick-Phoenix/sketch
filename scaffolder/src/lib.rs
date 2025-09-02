#![allow(dead_code)]
#![allow(clippy::result_large_err)]

#[macro_use]
pub mod config;
pub mod json_files;
pub mod moon;
pub mod package;
pub(crate) mod paths;

pub(crate) mod rendering;
use std::{fs::create_dir_all, io, path::PathBuf};

pub(crate) use config::*;
pub(crate) use rendering::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TemplateError {
  #[error("Could not create the dir '{path}': {source}")]
  DirCreation { path: PathBuf, source: io::Error },
  #[error("Could not create the file '{path}': {source}")]
  FileCreation { path: PathBuf, source: io::Error },
}

use std::fs::{self, File};

use askama::Template;
use figment::providers::{Format, Toml};

use crate::{
  json_files::{RootTsConfig, TsConfig},
  moon::{MoonTasks, MoonToolchain},
  Config, GitIgnore, OxlintConfig, PackageManager, PnpmWorkspace,
};

pub fn build_repo() -> Result<(), Box<dyn std::error::Error>> {
  let config: Config = Config::figment()
    .merge(Toml::file("scaffolder/config.toml"))
    .extract()?;

  let gitignore_data = if let Some(replacement) = config.gitignore_replacement {
    GitIgnore::Replacement(replacement)
  } else {
    GitIgnore::Additions(config.gitignore_additions)
  };

  let mut package_json_templates = config.package_json;
  let output = PathBuf::from(config.root_dir);

  create_dir_all(&output).map_err(|e| TemplateError::DirCreation {
    path: output.to_owned(),
    source: e,
  })?;

  macro_rules! write_file {
    ($data:expr, $suffix:expr) => {
      let path = output.join($suffix);
      let mut file = File::create(&path).map_err(|e| TemplateError::FileCreation {
        path: path.to_owned(),
        source: e,
      })?;
      $data.write_into(&mut file)?;
    };
  }

  write_file!(gitignore_data, ".gitignore");

  let package_json_data = match config.root_package_json {
    package::PackageJsonData::Named(name) => package_json_templates
      .remove(&name)
      .expect("Package json template not found"),
    package::PackageJsonData::Definition(package_json) => package_json,
  };

  write_file!(package_json_data, "package.json");
  write_file!(
    RootTsConfig {},
    format!("{}.json", config.root_tsconfig_name)
  );

  let root_tsconfig = TsConfig {
    root_tsconfig_path: format!("{}.json", config.root_tsconfig_name),
  };
  write_file!(root_tsconfig, "tsconfig.json");

  if matches!(config.package_manager, PackageManager::Pnpm) {
    write_file!(PnpmWorkspace {}, "pnpm-workspace.yaml");
  }

  if let Some(moon_config) = config.moonrepo {
    let moon_dir = output.join(".moon");
    fs::create_dir_all(&moon_dir).map_err(|e| TemplateError::DirCreation {
      path: moon_dir.to_owned(),
      source: e,
    })?;
    write_file!(
      MoonToolchain {
        package_manager: config.package_manager.clone(),
        root_tsconfig_name: config.root_tsconfig_name.clone(),
        project_tsconfig_name: config.project_tsconfig_name.clone(),
        config: moon_config.toolchain.unwrap_or_default(),
      },
      ".moon/toolchain.yml"
    );

    write_file!(
      MoonTasks {
        root_tsconfig_name: config.root_tsconfig_name.clone(),
        tasks: moon_config.tasks.unwrap_or_default(),
        config: moon_config.tasks_config.unwrap_or_default()
      },
      ".moon/tasks.yml"
    );
  }

  write_file!(config.pre_commit, ".pre-commit-config.yaml");

  write_file!(OxlintConfig {}, ".oxlintrc.json");

  fs::create_dir_all(output.join("packages")).map_err(|e| TemplateError::DirCreation {
    path: output.to_owned(),
    source: e,
  })?;

  Ok(())
}

#[cfg(test)]
mod test {
  use crate::build_repo;

  #[test]
  fn main_test() -> Result<(), Box<dyn std::error::Error>> {
    build_repo()
  }
}
