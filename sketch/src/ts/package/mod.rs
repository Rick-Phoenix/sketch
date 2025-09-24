use std::path::PathBuf;

use askama::Template;
use clap::Parser;
use merge::Merge;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{
  package_json::PackageJsonData,
  pnpm::PnpmWorkspace,
  ts_config::{tsconfig_defaults::*, TsConfig, TsConfigDirective, TsConfigKind},
  vitest::{TestsSetupFile, VitestConfig, VitestConfigKind},
};
use crate::{
  custom_templating::TemplateOutput,
  fs::{
    create_all_dirs, deserialize_json, deserialize_yaml, find_file_up, get_abs_path,
    get_relative_path, open_file_if_overwriting, serialize_json, serialize_yaml,
  },
  merge_if_not_default, merge_optional_vecs, overwrite_if_some,
  ts::{
    oxlint::{OxlintConfigSetting, OxlintPreset},
    ts_config, PackageManager,
  },
  Config, GenError, Preset,
};

/// The kind of ts package. Only relevant when using defaults.
#[derive(Debug, Deserialize, Serialize, Default, Clone, Copy, PartialEq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum PackageKind {
  #[default]
  Library,
  App,
}

/// The configuration struct that is used to generate new packages.
#[derive(Clone, Debug, Deserialize, Serialize, Parser, Merge, PartialEq, JsonSchema)]
#[merge(strategy = overwrite_if_some)]
#[serde(default)]
pub struct PackageConfig {
  /// The name of the new package. It defaults to the name of its parent directory.
  #[arg(short, long)]
  pub name: Option<String>,

  /// A list of [`TsConfigDirective`]s for this package. They can be preset ids or literal configurations. If unset, defaults are used.
  #[arg(short, long, value_parser = TsConfigDirective::from_cli)]
  #[arg(
    help = "One or many tsconfig presets (with their output path) to use for this package. If unset, defaults are used",
    value_name = "id=ID,output=PATH"
  )]
  #[merge(strategy = merge_optional_vecs)]
  pub ts_config: Option<Vec<TsConfigDirective>>,

  /// The [`PackageJsonData`] to use for this package. It can be a preset id or a literal definition (or nothing, to use defaults).
  #[arg(long, value_parser = PackageJsonData::from_cli)]
  #[arg(
    help = "The id of the package.json preset to use for this package",
    value_name = "ID"
  )]
  pub package_json: Option<PackageJsonData>,

  /// The templates to generate when this package is created.
  /// Relative output paths will be joined to the package's root directory.
  #[arg(long = "with-template", value_parser = TemplateOutput::from_cli, value_name = "id=TEMPLATE_ID,output=PATH")]
  #[merge(strategy = merge_optional_vecs)]
  pub with_templates: Option<Vec<TemplateOutput>>,

  /// The configuration for this package's vitest setup. It can be set to `false` to be disabled, or to a literal configuration.
  #[arg(skip)]
  #[merge(strategy = merge_if_not_default)]
  pub vitest: VitestConfigKind,

  /// The configuration for this package's oxlint setup. It can be set to `true` to use a basic default config, to a preset id, or to a literal configuration.
  #[arg(skip)]
  pub oxlint: Option<OxlintConfigSetting>,
}

impl Default for PackageConfig {
  fn default() -> Self {
    Self {
      name: None,
      package_json: None,
      vitest: Default::default(),
      ts_config: None,
      with_templates: Default::default(),
      oxlint: None,
    }
  }
}

/// The kinds of Ts package data. Either an id pointing to a stored preset, or a custom configuration.
pub enum PackageData {
  Preset(String),
  Config(PackageConfig),
}

impl Config {
  /// Generates a new typescript package.
  pub async fn build_package(
    self,
    data: PackageData,
    pkg_root: PathBuf,
    tsconfig_files_to_update: Option<Vec<PathBuf>>,
  ) -> Result<(), GenError> {
    let overwrite = !self.no_overwrite;
    let typescript = self.typescript.clone().unwrap_or_default();

    let package_json_presets = &typescript.package_json_presets;

    let config = match data {
      PackageData::Config(conf) => conf,
      PackageData::Preset(id) => typescript
        .package_presets
        .get(&id)
        .ok_or(GenError::PresetNotFound {
          kind: Preset::TsPackage,
          name: id.clone(),
        })?
        .clone(),
    };

    let package_name = if let Some(name) = config.name.as_ref() {
      name.clone()
    } else if let Some(dir_name) = pkg_root.file_name() {
      dir_name.to_string_lossy().to_string()
    } else {
      "my-awesome-package".to_string()
    };

    let pkg_root = get_abs_path(&pkg_root)?;

    create_all_dirs(&pkg_root)?;

    let package_manager = typescript.package_manager.unwrap_or_default();
    let version_ranges = typescript.version_range.unwrap_or_default();

    macro_rules! write_pkg_template {
      ($($tokens:tt)*) => {
        write_template!(pkg_root, overwrite, $($tokens)*)
      };
    }

    let (package_json_id, package_json_preset) = match config.package_json.unwrap_or_default() {
      PackageJsonData::Id(id) => (
        id.clone(),
        package_json_presets
          .get(&id)
          .ok_or(GenError::PresetNotFound {
            kind: Preset::PackageJson,
            name: id,
          })?
          .clone(),
      ),
      PackageJsonData::Config(package_json) => ("__inlined_definition".to_string(), package_json),
    };

    let mut package_json_data = package_json_preset.process_data(
      package_json_id.as_str(),
      package_json_presets,
      &typescript.people,
    )?;

    if package_json_data.package_manager.is_none() {
      package_json_data.package_manager = Some(package_manager.to_string());
    }

    if !typescript.no_default_deps {
      let mut default_deps = vec!["typescript", "oxlint"];

      if config.vitest.is_enabled() {
        default_deps.push("vitest")
      }

      for dep in default_deps {
        if !package_json_data.dev_dependencies.contains_key(dep) {
          let version = if typescript.catalog {
            "catalog:".to_string()
          } else {
            "latest".to_string()
          };

          package_json_data
            .dev_dependencies
            .insert(dep.to_string(), version);
        }
      }
    }

    package_json_data.name = Some(package_name.clone());

    if !typescript.no_convert_latest_to_range {
      package_json_data
        .convert_latest_to_range(version_ranges)
        .await?;
    }

    serialize_json(
      &package_json_data,
      &pkg_root.join("package.json"),
      overwrite,
    )?;

    if typescript.catalog && matches!(package_manager, PackageManager::Pnpm) {
      let pnpm_workspace_path =
        find_file_up(&pkg_root, "pnpm-workspace.yaml").ok_or(GenError::Custom(format!(
          "Could not find a `pnpm-workspace.yaml` file while searching upwards from `{}`",
          pkg_root.display()
        )))?;

      let mut pnpm_workspace: PnpmWorkspace = deserialize_yaml(&pnpm_workspace_path)?;

      pnpm_workspace
        .add_dependencies_to_catalog(version_ranges, &package_json_data)
        .await;

      serialize_yaml(&pnpm_workspace, &pnpm_workspace_path, overwrite)?;
    }

    let mut tsconfig_files: Vec<(String, TsConfig)> = Default::default();

    let tsconfig_presets = &typescript.ts_config_presets;

    if let Some(tsconfig_directives) = config.ts_config.clone() {
      for directive in tsconfig_directives {
        let (id, tsconfig) = match directive.config.unwrap_or_default() {
          TsConfigKind::Id(id) => {
            let tsconfig = tsconfig_presets
              .get(&id)
              .ok_or(GenError::PresetNotFound {
                kind: Preset::TsConfig,
                name: id.clone(),
              })?
              .clone();

            (id.to_string(), tsconfig)
          }
          TsConfigKind::Config(ts_config) => {
            (format!("__inlined_config_{}", package_name), ts_config)
          }
        };

        let tsconfig = tsconfig.process_data(id.as_str(), tsconfig_presets)?;

        tsconfig_files.push((
          directive
            .output
            .unwrap_or_else(|| "tsconfig.json".to_string()),
          tsconfig,
        ));
      }
    } else {
      tsconfig_files.push(("tsconfig.json".to_string(), get_default_package_tsconfig()));
    }

    for (file, tsconfig) in tsconfig_files {
      serialize_json(&tsconfig, &pkg_root.join(file), overwrite)?;
    }

    if let Some(tsconfig_paths) = tsconfig_files_to_update {
      for path in tsconfig_paths {
        let mut tsconfig: TsConfig = deserialize_json(&path)?;

        let path_to_new_tsconfig = get_relative_path(&path, &pkg_root.join("tsconfig.json"))?;

        let root_tsconfig_references = tsconfig.references.get_or_insert_default();

        root_tsconfig_references.insert(ts_config::TsConfigReference {
          path: path_to_new_tsconfig.to_string_lossy().to_string(),
        });

        serialize_json(&tsconfig, &path, true)?;
      }
    }

    let src_dir = pkg_root.join("src");
    create_all_dirs(&src_dir)?;

    let _index_file = open_file_if_overwriting(overwrite, &src_dir.join("index.ts"))?;

    let vitest_config = match config.vitest {
      VitestConfigKind::Bool(v) => v.then(VitestConfig::default),
      VitestConfigKind::Config(vitest_config_struct) => Some(vitest_config_struct),
    };

    if let Some(mut vitest) = vitest_config {
      let tests_dir = pkg_root.join(&vitest.tests_dir);
      let tests_setup_dir = tests_dir.join(&vitest.setup_dir);

      create_all_dirs(&tests_dir)?;
      create_all_dirs(&tests_setup_dir)?;

      let file_parent_dir = vitest
        .out_dir
        .as_deref()
        .unwrap_or_else(|| tests_dir.as_path());

      let file_path = file_parent_dir.join("vitest.config.ts");

      let src_rel_path = get_relative_path(&file_parent_dir, &src_dir)?;

      vitest.src_rel_path = src_rel_path.to_string_lossy().to_string();

      vitest.setup_dir = get_relative_path(&file_parent_dir, &tests_setup_dir)?
        .to_string_lossy()
        .to_string();

      write_pkg_template!(vitest, file_path);
      write_pkg_template!(TestsSetupFile, tests_setup_dir.join("tests_setup.ts"));
    }

    if let Some(oxlint_config) = config.oxlint && oxlint_config.is_enabled() {
      let (id, oxlint_config) = match oxlint_config {
        OxlintConfigSetting::Bool(_) => ("__default".to_string(), OxlintPreset::default()),
        OxlintConfigSetting::Id(id) => (id.clone(), typescript.oxlint_presets.get(id.as_str()).ok_or(GenError::PresetNotFound { kind: Preset::Oxlint, name: id.clone() })?.clone()),
        OxlintConfigSetting::Config(oxlint_preset) => (format!("__inlined_definition_{}", package_name), oxlint_preset),
      };

      let merged_config = oxlint_config.process_data(id.as_str(), &typescript.oxlint_presets)?;

      serialize_json(&merged_config, &pkg_root.join(".oxlintrc.json"), overwrite)?;
    }

    if let Some(templates) = config.with_templates && !templates.is_empty() {
      self
        .generate_templates(&pkg_root, templates)?;
    }

    Ok(())
  }
}
