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
      .map(|c| -> Result<Person, GenError> {
        match c {
          Person::Workspace(name) => Ok(Person::Data(
            $config
              .people
              .get(&name)
              .ok_or(GenError::PersonNotFound { name })?
              .clone(),
          )),
          Person::Data(person) => Ok(Person::Data(person)),
        }
      })
      .collect::<Result<std::collections::BTreeSet<Person>, GenError>>()?
  };
}

macro_rules! write_file {
  ($output:expr, $data:expr, $suffix:expr) => {
    let path = $output.join($suffix);
    let mut file = File::create(&path).map_err(|e| GenError::FileCreation {
      path: path.to_owned(),
      source: e,
    })?;
    $data
      .write_into(&mut file)
      .map_err(|e| GenError::WriteError {
        path: path.clone(),
        source: e,
      })?;
  };
}

#[macro_use]
pub mod config;
pub mod moon;
pub mod package;
pub mod package_json;
pub(crate) mod paths;
pub mod pnpm;
pub(crate) mod tera;
pub mod ts_config;
pub mod versions;

use std::fs::File;

use askama::Template;
use figment::providers::{Format, Toml};

use crate::{
  moon::{MoonTasks, MoonToolchain},
  versions::get_latest_version,
  Config, OxlintConfig, PackageManager,
};

pub(crate) mod rendering;
use std::{collections::BTreeMap, fs::create_dir_all, io, path::PathBuf};

pub(crate) use config::*;
pub(crate) use rendering::*;
use thiserror::Error;

pub(crate) type StringKeyVal = BTreeMap<String, String>;
pub(crate) type StringKeyValMap = BTreeMap<String, Value>;

#[derive(Debug, Clone, Copy)]
pub enum Preset {
  Vitest,
  PackageJson,
  Package,
  TsConfig,
}

#[derive(Debug, Error)]
pub enum GenError {
  #[error("Could not create the dir '{path}': {source}")]
  DirCreation { path: PathBuf, source: io::Error },
  #[error("Could not create the file '{path}': {source}")]
  FileCreation { path: PathBuf, source: io::Error },
  #[error("Failed to parse the configuration: {source}")]
  ConfigParsing { source: figment::Error },
  #[error("{kind:?} preset '{name}' not found")]
  PresetNotFound { kind: Preset, name: String },
  #[error("Failed to parse the template '{template}': {source}")]
  TemplateParsing {
    template: String,
    source: ::tera::Error,
  },
  #[error("Failed to read the templates directory: {source}")]
  TemplateDirLoading { source: ::tera::Error },
  #[error("Failed to parse the templating context: {source}")]
  TemplateContextParsing { source: ::tera::Error },
  #[error("Could not create the parent directory for '{path}': {source}")]
  ParentDirCreation { path: PathBuf, source: io::Error },
  #[error("Failed to render the template '{template}': {source}")]
  TemplateRendering {
    template: String,
    source: ::tera::Error,
  },
  #[error("Failed to write to the file '{path}': {source}")]
  WriteError { path: PathBuf, source: io::Error },
  #[error("Person '{name}' not found")]
  PersonNotFound { name: String },
  #[error("Could not read the contents of '{path}': {source}")]
  ReadError { path: PathBuf, source: io::Error },
  #[error("Could not deserialize '{path}': {source}")]
  YamlDeserialization {
    path: PathBuf,
    source: serde_yaml_ng::Error,
  },
  #[error("Could not deserialize '{path}': {source}")]
  JsonDeserialization {
    path: PathBuf,
    source: serde_json::Error,
  },
  #[error("Failed to canonicalize the path '{path}': {source}")]
  PathCanonicalization { path: PathBuf, source: io::Error },
}

pub(crate) const DEFAULT_DEPS: [&str; 3] = ["typescript", "vitest", "oxlint"];

pub async fn build_repo() -> Result<(), GenError> {
  let config: Config = Config::figment()
    .merge(Toml::file("scaffolder/config.toml"))
    .extract()
    .map_err(|e| GenError::ConfigParsing { source: e })?;

  let package_json_templates = &config.package_json_presets;
  let output = PathBuf::from(&config.root_dir);

  create_dir_all(&output).map_err(|e| GenError::DirCreation {
    path: output.to_owned(),
    source: e,
  })?;

  macro_rules! write_to_output {
    ($($tokens:tt)*) => {
      write_file!(output, $($tokens) *)
    };
  }

  write_to_output!(config.gitignore, ".gitignore");

  let mut package_json_data = match &config.root_package_json {
    PackageJsonData::Named(name) => package_json_templates
      .get(name)
      .ok_or(GenError::PresetNotFound {
        kind: Preset::PackageJson,
        name: name.to_string(),
      })?
      .clone(),
    PackageJsonData::Definition(package_json) => package_json.clone(),
  };

  for preset in package_json_data.extends.clone() {
    let target = package_json_templates
      .get(&preset)
      .ok_or(GenError::PresetNotFound {
        kind: Preset::PackageJson,
        name: preset,
      })?
      .clone();

    package_json_data.merge(target);
  }

  if package_json_data.package_manager.is_none() {
    package_json_data.package_manager = Some(config.package_manager.to_string());
  }

  get_contributors!(package_json_data, config, contributors);
  get_contributors!(package_json_data, config, maintainers);

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

  write_to_output!(package_json_data, "package.json");

  let tsconfig_options = TsConfig {
    compiler_options: Some(CompilerOptions {
      lib: Some(vec![Lib::EsNext, Lib::Dom]),
      module_resolution: Some(ModuleResolution::NodeNext),
      module: Some(Module::EsNext),
      target: Some(Target::EsNext),
      module_detection: Some(ModuleDetection::Force),
      isolated_modules: Some(true),
      es_module_interop: Some(true),
      resolve_json_module: Some(true),
      declaration: Some(true),
      declaration_map: Some(true),
      composite: Some(true),
      no_emit_on_error: Some(true),
      incremental: Some(true),
      source_map: Some(true),
      strict: Some(true),
      strict_null_checks: Some(true),
      skip_lib_check: Some(true),
      force_consistent_casing_in_file_names: Some(true),
      no_unchecked_indexed_access: Some(true),
      allow_synthetic_default_imports: Some(true),
      ..Default::default()
    }),
    ..Default::default()
  };

  write_to_output!(tsconfig_options, config.root_tsconfig_name.clone());

  let root_tsconfig = TsConfig {
    extends: Some(config.root_tsconfig_name.clone()),
    files: Some(vec![]),
    references: Some(vec![]),
    ..Default::default()
  };

  write_to_output!(root_tsconfig, "tsconfig.json");

  if matches!(config.package_manager, PackageManager::Pnpm) {
    let mut pnpm_data = PnpmWorkspace {
      catalog: Default::default(),
      packages: config.package_dirs.clone(),
      extra: config.pnpm_config.clone(),
      catalogs: Default::default(),
    };

    pnpm_data
      .add_dependencies_to_catalog(config.version_ranges, &package_json_data)
      .await;

    write_to_output!(pnpm_data, "pnpm-workspace.yaml");
  }

  if let Some(ref moon_config) = config.moonrepo {
    let moon_dir = output.join(".moon");

    create_dir_all(&moon_dir).map_err(|e| GenError::DirCreation {
      path: moon_dir.to_owned(),
      source: e,
    })?;

    write_to_output!(
      MoonToolchain {
        package_manager: config.package_manager.clone(),
        root_tsconfig_name: config.root_tsconfig_name.clone(),
        project_tsconfig_name: config.project_tsconfig_name.clone(),
        config: moon_config.toolchain.clone().unwrap_or_default(),
      },
      ".moon/toolchain.yml"
    );

    let moon_tasks = MoonTasks {
      tasks: moon_config.tasks.clone().unwrap_or_default(),
      config: moon_config.tasks_config.clone().unwrap_or_default(),
      project_tsconfig_name: config.project_tsconfig_name.clone(),
      root_tsconfig_name: config.root_tsconfig_name.clone(),
      out_dir: config.out_dir.clone(),
    };

    write_to_output!(moon_tasks, ".moon/tasks.yml");
  }

  write_to_output!(config.pre_commit, ".pre-commit-config.yaml");

  write_to_output!(OxlintConfig {}, ".oxlintrc.json");

  create_dir_all(output.join("packages")).map_err(|e| GenError::DirCreation {
    path: output.to_owned(),
    source: e,
  })?;

  create_dir_all(output.join(&config.out_dir)).map_err(|e| GenError::DirCreation {
    path: output.to_owned(),
    source: e,
  })?;

  if !config.generate_root_templates.is_empty() {
    config.generate_templates(&config.root_dir, config.generate_root_templates.clone())?;
  }

  Ok(())
}

#[cfg(test)]
mod test {
  use crate::{build_repo, GenError};

  #[tokio::test]
  async fn repo_test() -> Result<(), GenError> {
    build_repo().await
  }
}
