#![allow(clippy::result_large_err)]

#[doc = include_str!("../README.md")]
#[macro_use]
mod macros;

use std::path::Path;

use askama::Template;
use figment::{
  providers::{Format, Json, Toml, Yaml},
  Figment,
};
use indexmap::IndexMap;
use maplit::btreeset;
use merge::Merge;
use serde_json::Value;

use crate::{
  package_json::{PackageJsonKind, Person},
  paths::{get_abs_path, get_cwd, get_parent_dir},
};
pub mod config_elements;
pub use config::*;
pub use config_elements::*;
pub use errors::*;
pub mod commands;
pub(crate) mod init_repo;
pub(crate) mod serde_strategies;

use crate::{
  moon::{MoonConfig, MoonConfigKind},
  package_json::PackageJson,
  ts_config::{tsconfig_defaults::get_default_root_tsconfig, TsConfig, TsConfigKind},
};

pub(crate) mod templating;
use std::{
  collections::BTreeMap,
  fs::{create_dir_all, File},
  path::PathBuf,
};

pub(crate) use merging_strategies::*;
pub(crate) use templating::*;

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
  Templating,
}

pub(crate) const DEFAULT_DEPS: [&str; 3] = ["typescript", "vitest", "oxlint"];

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

  if let Some(templates_dir) = &config.templates_dir {
    config.templates_dir = Some(get_abs_path(
      &get_parent_dir(config_file_abs).join(templates_dir),
    )?);
  }

  if let Some(root_dir) = &config.root_dir {
    config.root_dir = Some(get_abs_path(
      &get_parent_dir(config_file_abs).join(root_dir),
    )?);
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

impl Config {
  pub async fn create_ts_monorepo(self) -> Result<(), GenError> {
    let typescript = self.typescript.clone().unwrap_or_default();

    let package_json_presets = &typescript.package_json_presets;

    let root_dir = self.root_dir.clone().unwrap_or_else(|| get_cwd());

    let package_manager = typescript.package_manager.unwrap_or_default();
    let version_ranges = typescript.version_range.unwrap_or_default();
    let root_package = typescript.root_package.unwrap_or_default();

    create_dir_all(&root_dir).map_err(|e| GenError::DirCreation {
      path: root_dir.to_owned(),
      source: e,
    })?;

    macro_rules! write_to_output {
      ($($tokens:tt)*) => {
        write_file!(root_dir, !self.no_overwrite, $($tokens)*)
      };
    }

    let mut package_json_data = match root_package.package_json.unwrap_or_default() {
      PackageJsonKind::Id(id) => package_json_presets
        .get(&id)
        .ok_or(GenError::PresetNotFound {
          kind: Preset::PackageJson,
          name: id,
        })?
        .clone(),
      PackageJsonKind::Config(package_json) => *package_json,
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
      package_json_data.package_manager = Some(package_manager.to_string());
    }

    get_contributors!(package_json_data, typescript, contributors);
    get_contributors!(package_json_data, typescript, maintainers);

    if package_json_data.use_default_deps {
      for dep in ["typescript", "oxlint"] {
        let version = if typescript.no_catalog {
          "latest".to_string()
        } else {
          "catalog:".to_string()
        };

        let range = version_ranges.create(version);
        package_json_data
          .dev_dependencies
          .insert(dep.to_string(), range);
      }
    }

    if !typescript.no_convert_latest_to_range {
      package_json_data
        .get_latest_version_range(version_ranges)
        .await?;
    }

    package_json_data.name = root_package
      .name
      .clone()
      .unwrap_or_else(|| "root".to_string());

    write_to_output!(package_json_data, "package.json");

    let mut tsconfig_files: Vec<(String, TsConfig)> = Default::default();
    let tsconfig_presets = &typescript.tsconfig_presets;

    if let Some(root_tsconfigs) = root_package.ts_config {
      for directive in root_tsconfigs {
        let (id, mut tsconfig) = match directive.config.unwrap_or_default() {
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

        tsconfig_files.push((
          directive
            .output
            .unwrap_or_else(|| "tsconfig.json".to_string()),
          tsconfig,
        ));
      }
    } else {
      let tsconfig_options = get_default_root_tsconfig();

      tsconfig_files.push((
        typescript
          .root_tsconfig_name
          .clone()
          .unwrap_or_else(|| "tsconfig.options.json".to_string()),
        tsconfig_options,
      ));

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

    if matches!(package_manager, PackageManager::Pnpm) {
      let mut pnpm_data = typescript.pnpm_config.unwrap_or_default();

      for dir in &pnpm_data.packages {
        create_dir_all(root_dir.join(dir)).map_err(|e| GenError::DirCreation {
          path: root_dir.to_path_buf(),
          source: e,
        })?;
      }

      pnpm_data
        .add_dependencies_to_catalog(version_ranges, &package_json_data)
        .await;

      write_to_output!(pnpm_data, "pnpm-workspace.yaml");
    }

    if let Some(moon_config_kind) = root_package.moonrepo && !matches!(moon_config_kind, MoonConfigKind::Bool(false)) {
      let moon_config = match moon_config_kind {
        MoonConfigKind::Bool(_) => MoonConfig::default(),
        MoonConfigKind::Config(c) => *c
      };

      let moon_dir = root_dir.join(".moon");

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

    if let Some(oxlint_config) = root_package.oxlint && !matches!(oxlint_config, OxlintConfig::Bool(false)) {
      write_to_output!(oxlint_config, ".oxlintrc.json");
    }

    if let Some(shared_out_dir) = typescript.shared_out_dir.get_name() {
      create_dir_all(root_dir.join(shared_out_dir)).map_err(|e| GenError::DirCreation {
        path: root_dir.to_path_buf(),
        source: e,
      })?;
    }

    if let Some(templates) = root_package.generate_templates && !templates.is_empty() {
      self.generate_templates(&root_dir, templates)?;
    }

    Ok(())
  }
}
