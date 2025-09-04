#![allow(dead_code)]
#![allow(clippy::result_large_err)]

use merge::Merge;
pub use package_json::*;
use serde_json::Value;
pub use ts_config::*;

use crate::pnpm::PnpmWorkspace;

macro_rules! get_contributors {
  ($data:ident, $config:ident, $list_name:ident) => {
    $data.$list_name = $data
      .$list_name
      .into_iter()
      .map(|c| match c {
        Person::Workspace(name) => Person::Data(
          $config
            .people
            .get(&name)
            .expect("Contributor not found")
            .clone(),
        ),
        Person::Data(person) => Person::Data(person),
      })
      .collect()
  };
}

#[macro_use]
pub mod config;
pub mod moon;
pub mod package;
pub mod package_json;
pub(crate) mod paths;
pub mod pnpm;
pub mod ts_config;
pub mod versions;

pub(crate) mod rendering;
use std::{collections::BTreeMap, fs::create_dir_all, io, path::PathBuf};

pub(crate) use config::*;
pub(crate) use rendering::*;
use thiserror::Error;

pub(crate) type StringKeyVal = BTreeMap<String, String>;
pub(crate) type StringKeyValMap = BTreeMap<String, Value>;

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
  moon::{MoonTasks, MoonToolchain},
  versions::get_latest_version,
  Config, OxlintConfig, PackageManager,
};

pub(crate) const DEFAULT_DEPS: [&str; 3] = ["typescript", "vitest", "oxlint"];

pub async fn build_repo() -> Result<(), Box<dyn std::error::Error>> {
  let config: Config = Config::figment()
    .merge(Toml::file("scaffolder/config.toml"))
    .extract()?;

  let mut package_json_templates = config.package_json_presets;
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

  write_file!(config.gitignore, ".gitignore");

  let mut package_json_data = match config.root_package_json {
    PackageJsonData::Named(name) => package_json_templates
      .remove(&name)
      .expect("Package json template not found"),
    PackageJsonData::Definition(package_json) => package_json,
  };

  for preset in package_json_data.extends.clone() {
    let target = package_json_templates
      .get(&preset)
      .unwrap_or_else(|| panic!("Could not find package.json preset '{}'", preset))
      .clone();

    package_json_data.merge(target);
  }

  get_contributors!(package_json_data, config, contributors);
  get_contributors!(package_json_data, config, maintainers);

  write_file!(package_json_data, "package.json");

  write_file!(RootTsConfig {}, config.root_tsconfig_name.clone());

  let root_tsconfig = TsConfig {
    root_tsconfig_path: config.root_tsconfig_name.clone(),
    references: Default::default(),
  };
  write_file!(root_tsconfig, "tsconfig.json");

  if package_json_data.default_deps {
    for dep in DEFAULT_DEPS {
      let version = if config.catalog {
        "catalog:".to_string()
      } else {
        get_latest_version(dep).await.unwrap_or_else(|_| {
          println!(
            "Could not get the latest valid version range for '{}'. Falling back to 'latest'...",
            dep
          );
          "latest".to_string()
        })
      };

      let range = config.version_ranges.create(version);
      package_json_data
        .dev_dependencies
        .insert(dep.to_string(), range);
    }
  }

  if matches!(config.package_manager, PackageManager::Pnpm) {
    let mut pnpm_data = PnpmWorkspace {
      catalog: Default::default(),
      packages: config.package_dirs,
      extra: config.pnpm_config,
      catalogs: Default::default(),
    };

    pnpm_data
      .add_dependencies_to_catalog(config.version_ranges, &package_json_data)
      .await;

    write_file!(pnpm_data, "pnpm-workspace.yaml");
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

    let moon_tasks = MoonTasks {
      tasks: moon_config.tasks.unwrap_or_default(),
      config: moon_config.tasks_config.unwrap_or_default(),
      project_tsconfig_name: config.project_tsconfig_name.clone(),
      root_tsconfig_name: config.root_tsconfig_name.clone(),
      out_dir: config.out_dir.clone(),
    };

    write_file!(moon_tasks, ".moon/tasks.yml");
  }

  write_file!(config.pre_commit, ".pre-commit-config.yaml");

  write_file!(OxlintConfig {}, ".oxlintrc.json");

  create_dir_all(output.join("packages")).map_err(|e| TemplateError::DirCreation {
    path: output.to_owned(),
    source: e,
  })?;

  create_dir_all(output.join(config.out_dir))?;

  Ok(())
}

#[cfg(test)]
mod test {
  use crate::build_repo;

  #[tokio::test]
  async fn repo_test() -> Result<(), Box<dyn std::error::Error>> {
    build_repo().await
  }
}
