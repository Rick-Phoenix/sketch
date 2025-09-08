#![allow(clippy::result_large_err)]

use askama::Template;
use figment::{
  providers::{Format, Toml, Yaml},
  Figment,
};
use indexmap::IndexMap;
use maplit::btreeset;
use merge::Merge;
pub(crate) use package_json::*;
use serde_json::Value;
pub(crate) use ts_config::*;

use crate::{
  moon::{MoonConfig, MoonConfigKind},
  tsconfig_defaults::get_default_root_tsconfig,
  versions::get_latest_version,
  Config, OxlintConfig, PackageManager,
};

pub(crate) mod rendering;
use std::{
  collections::BTreeMap,
  fs::{create_dir_all, File},
  path::{Path, PathBuf},
};

pub(crate) use config::*;
pub use errors::GenError;
pub(crate) use merging_strategies::*;
pub(crate) use rendering::*;

use crate::pnpm::PnpmWorkspace;

#[macro_use]
mod macros;
pub mod cli;
pub mod config;
pub mod errors;
pub(crate) mod merging_strategies;
pub mod moon;
pub mod package;
pub mod package_json;
pub(crate) mod paths;
pub mod pnpm;
pub mod ts_config;
pub mod versions;

pub(crate) type StringBTreeMap = BTreeMap<String, String>;
pub(crate) type JsonValueBTreeMap = BTreeMap<String, Value>;

pub(crate) type OrderedMap = IndexMap<String, Value>;

/// The kinds of presets that can be stored in the global config, along with a name key.
#[derive(Debug, Clone, Copy)]
pub enum Preset {
  Vitest,
  PackageJson,
  Package,
  TsConfig,
}

#[derive(Debug, Template)]
#[template(ext = "txt", source = "{{ text }}")]
pub(crate) struct GenericTemplate {
  pub(crate) text: String,
}

pub(crate) fn convert_btreemap_to_json<T>(map: BTreeMap<String, T>) -> Value
where
  T: Into<Value>,
{
  map.into_iter().collect()
}

pub(crate) const DEFAULT_DEPS: [&str; 3] = ["typescript", "vitest", "oxlint"];

pub(crate) fn merge_path(mut figment: Figment, path: &Path) -> Result<Figment, GenError> {
  let extension = path
    .extension()
    .unwrap_or_else(|| panic!("Config file '{}' has no extension.", path.display()));

  if extension == "yaml" || extension == "yml" {
    figment = figment.merge(Yaml::file(path));
  } else if extension == "toml" {
    figment = figment.merge(Toml::file(path));
  } else {
    return Err(GenError::InvalidConfigFormat {
      file: path.to_path_buf(),
    });
  }

  Ok(figment)
}

impl Config {
  pub fn init(config_file: PathBuf) -> Result<Self, GenError> {
    let config_figment = merge_path(Config::figment(), &config_file)?;

    let mut config: Config = config_figment
      .extract()
      .map_err(|e| GenError::ConfigParsing { source: e })?;

    if !config.extends.is_empty() {
      config = config.merge_configs(&config_file.canonicalize().map_err(|e| {
        GenError::PathCanonicalization {
          path: config_file,
          source: e,
        }
      })?)?;
    }

    Ok(config)
  }
}

impl Config {
  pub async fn build_repo(self) -> Result<(), GenError> {
    let package_json_presets = &self.package_json_presets;
    let output = PathBuf::from(&self.root_dir);

    create_dir_all(&output).map_err(|e| GenError::DirCreation {
      path: output.to_owned(),
      source: e,
    })?;

    macro_rules! write_to_output {
    ($($tokens:tt)*) => {
      write_file!(output, self.overwrite, $($tokens)*)
    };
  }

    write_to_output!(self.gitignore, ".gitignore");

    let mut package_json_data = match package_json_presets.get("root") {
      Some(v) => v.clone(),
      None => PackageJson::default(),
    };

    for preset in package_json_data.extends.clone() {
      let target = package_json_presets
        .get(&preset)
        .ok_or(GenError::PresetNotFound {
          kind: Preset::PackageJson,
          name: preset,
        })?
        .clone();

      package_json_data.merge(target);
    }

    if package_json_data.package_manager.is_none() {
      package_json_data.package_manager = Some(self.package_manager.to_string());
    }

    get_contributors!(package_json_data, self, contributors);
    get_contributors!(package_json_data, self, maintainers);

    if package_json_data.use_default_deps {
      for dep in ["typescript", "oxlint"] {
        let version = if self.catalog {
          "catalog:".to_string()
        } else {
          get_latest_version(dep).await.unwrap_or_else(|e| {
            println!(
              "Could not get the latest valid version range for '{}' due to the following error: {}.\nFalling back to 'latest'...",
              e,
              dep
            );
            "latest".to_string()
          })
        };

        let range = self.version_ranges.create(version);
        package_json_data
          .dev_dependencies
          .insert(dep.to_string(), range);
      }
    }

    if self.get_latest_version_range {
      package_json_data
        .get_latest_version_range(self.version_ranges)
        .await?;
    }

    package_json_data.name = self.root_package.name.clone();

    write_to_output!(package_json_data, "package.json");

    let mut tsconfig_files: Vec<(String, TsConfig)> = Default::default();
    let tsconfig_presets = &self.tsconfig_presets;

    if let Some(root_tsconfigs) = self.root_package.ts_configs.clone() {
      for directive in root_tsconfigs {
        let (id, mut tsconfig) = match directive.config {
          TsConfigKind::Id(id) => {
            let tsconfig = tsconfig_presets
              .get(&id)
              .ok_or(GenError::PresetNotFound {
                kind: Preset::TsConfig,
                name: id.clone(),
              })?
              .clone();

            (id, tsconfig)
          }
          TsConfigKind::Config(ts_config) => ("__root".to_string(), *ts_config),
        };

        if !tsconfig.extend_presets.is_empty() {
          tsconfig = tsconfig.merge_configs(&id, tsconfig_presets)?;
        }

        tsconfig_files.push((directive.file_name, tsconfig));
      }
    } else {
      let tsconfig_options = get_default_root_tsconfig();

      tsconfig_files.push((self.root_tsconfig_name.clone(), tsconfig_options));

      let root_tsconfig = TsConfig {
        files: Some(btreeset![]),
        references: Some(btreeset![]),
        ..Default::default()
      };

      tsconfig_files.push(("tsconfig.json".to_string(), root_tsconfig));
    }

    for (file, tsconfig) in tsconfig_files {
      write_to_output!(tsconfig, file);
    }

    if matches!(self.package_manager, PackageManager::Pnpm) {
      let mut pnpm_data = PnpmWorkspace {
        catalog: Default::default(),
        packages: self.packages_dirs.clone(),
        extra: self.pnpm_config.clone(),
        catalogs: Default::default(),
      };

      pnpm_data
        .add_dependencies_to_catalog(self.version_ranges, &package_json_data)
        .await;

      write_to_output!(pnpm_data, "pnpm-workspace.yaml");
    }

    if let Some(ref moon_config_kind) = self.moonrepo && !matches!(moon_config_kind, MoonConfigKind::Bool(false)) {
      let moon_config = match moon_config_kind.clone() {
        MoonConfigKind::Bool(_) => MoonConfig::default(),
        MoonConfigKind::Config(c) => *c
      };

      let moon_dir = output.join(".moon");

      create_dir_all(&moon_dir).map_err(|e| GenError::DirCreation {
        path: moon_dir.to_path_buf(),
        source: e,
      })?;

      let moon_toolchain = moon_config.toolchain.unwrap_or_default();

      write_to_output!(
        moon_toolchain,
        ".moon/toolchain.yml"
      );

      let moon_tasks = moon_config.tasks.unwrap_or_default();

      write_to_output!(moon_tasks, ".moon/tasks.yml");
    }

    let pre_commit_config = match &self.pre_commit {
      PreCommitSetting::Bool(val) => {
        if *val {
          Some(&PreCommitConfig::default())
        } else {
          None
        }
      }
      PreCommitSetting::Config(conf) => Some(conf),
    };

    if let Some(pre_commit) = pre_commit_config {
      write_to_output!(pre_commit, ".pre-commit-config.yaml");
    }

    if !matches!(self.root_package.oxlint, OxlintConfig::Bool(false)) {
      write_to_output!(self.root_package.oxlint, ".oxlintrc.json");
    }

    for dir in &self.packages_dirs {
      create_dir_all(output.join(dir)).map_err(|e| GenError::DirCreation {
        path: output.to_path_buf(),
        source: e,
      })?;
    }

    if let Some(shared_out_dir) = self.shared_out_dir.get_name() {
      create_dir_all(output.join(shared_out_dir)).map_err(|e| GenError::DirCreation {
        path: output.to_path_buf(),
        source: e,
      })?;
    }

    if !self.root_package.generate_templates.is_empty() {
      let templates = self.root_package.generate_templates.clone();
      let base_path = self.root_dir.clone();
      self.generate_templates(&base_path, templates)?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod test {
  use std::path::PathBuf;

  use crate::{config::Config, GenError};

  #[tokio::test]
  async fn repo_test() -> Result<(), GenError> {
    let config = Config::init(PathBuf::from("config.toml"))?;

    config.build_repo().await
  }

  #[tokio::test]
  async fn circular_configs() -> Result<(), GenError> {
    let config = Config::init(PathBuf::from("tests/circular_configs/config.toml"));

    match config {
      Ok(_) => panic!("Circular configs test did not fail as expected"),
      Err(e) => {
        if matches!(e, GenError::CircularDependency(_)) {
          Ok(())
        } else {
          panic!("Circular configs test returned wrong kind of error")
        }
      }
    }
  }
}
