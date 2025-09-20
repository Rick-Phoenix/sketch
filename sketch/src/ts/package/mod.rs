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
  custom_templating::TemplateOutput, fs::{
    create_all_dirs, deserialize_json, deserialize_yaml, get_abs_path, get_cwd, get_relative_path,
    open_file_for_writing, serialize_json, serialize_yaml,
  }, merge_if_not_default,  overwrite_if_some, ts::{oxlint::{OxlintConfigSetting, OxlintPreset}, ts_config, PackageManager}, Config, GenError, Preset
};

/// The kind of ts package.
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
  /// The new package's directory, starting from the [`Config::out_dir`]. Defaults to the name of the package.
  #[arg(
    value_name = "DIR",
    help = "The new package's directory, starting from the `out_dir`. Defaults to the name of the package"
  )]
  pub dir: Option<PathBuf>,

  /// The name of the new package. If `dir` is set, it defaults to the last segment of it.
  #[arg(short, long)]
  pub name: Option<String>,

  /// A list of [`TsConfigDirective`]s for this package. They can be preset ids or literal configurations. If unset, defaults are used.
  #[arg(short, long, value_parser = TsConfigDirective::from_cli)]
  #[arg(
    help = "One or many tsconfig files for this package. If unset, defaults are used",
    value_name = "output=PATH,id=ID"
  )]
  pub ts_config: Option<Vec<TsConfigDirective>>,

  /// The [`PackageJsonKind`] to use for this package. It can be a preset id or a literal definition (or nothing, to use defaults).
  #[arg(long, value_parser = PackageJsonData::from_cli)]
  #[arg(
    help = "The id of the package.json preset to use for this package",
    value_name = "ID"
  )]
  pub package_json: Option<PackageJsonData>,

  /// The templates to generate when this package is created.
  /// Relative output paths will be joined to the package's root directory.
  #[arg(skip)]
  pub with_templates: Option<Vec<TemplateOutput>>,

  /// The kind of package [default: 'library'].
  #[arg(skip)]
  pub kind: Option<PackageKind>,

  /// The configuration for this package's vitest setup. It can be set to true/false (to use defaults or to disable it), or as a customized configuration.
  #[arg(skip)]
  #[merge(strategy = merge_if_not_default)]
  pub vitest: VitestConfigKind,

  /// The configuration for this package's oxlint setup. It can be set to true/false (to use defaults or to disable it), or to a literal configuration.
  #[arg(skip)]
  pub oxlint: Option<OxlintConfigSetting>,
}

impl Default for PackageConfig {
  fn default() -> Self {
    Self {
      name: None,
      kind: Default::default(),
      package_json: None,
      dir: Default::default(),
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
    update_root_tsconfig: bool,
  ) -> Result<(), GenError> {
    let typescript = self.typescript.clone().unwrap_or_default();

    let package_json_presets = &typescript.package_json_presets;

    let monorepo_root = self.out_dir.clone().unwrap_or_else(|| get_cwd());

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
    } else if let Some(dir) = config.dir.as_ref() && let Some(base) = dir.file_name() {
      base.to_string_lossy().to_string()
    } else {
      "my-awesome-package".to_string()
    };

    let mut pkg_root = monorepo_root.join(
      config
        .dir
        .clone()
        .unwrap_or_else(|| package_name.clone().into()),
    );

    create_all_dirs(&pkg_root)?;

    pkg_root = get_abs_path(&pkg_root)?;

    let package_manager = typescript.package_manager.unwrap_or_default();
    let version_ranges = typescript.version_range.unwrap_or_default();

    macro_rules! write_to_output {
      ($($tokens:tt)*) => {
        write_file!(pkg_root, self.no_overwrite, $($tokens)*)
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

    serialize_json(&package_json_data, &pkg_root.join("package.json"))?;

    if typescript.catalog && matches!(package_manager, PackageManager::Pnpm) {
      let pnpm_workspace_path = monorepo_root.join("pnpm-workspace.yaml");

      let mut pnpm_workspace: PnpmWorkspace = deserialize_yaml(&pnpm_workspace_path)?;

      pnpm_workspace
        .add_dependencies_to_catalog(version_ranges, &package_json_data)
        .await;

      serialize_yaml(&pnpm_workspace, &pnpm_workspace_path)?;
    }

    let mut tsconfig_files: Vec<(String, TsConfig)> = Default::default();

    let tsconfig_presets = &typescript.tsconfig_presets;

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
      let is_app = matches!(config.kind.unwrap_or_default(), PackageKind::App);
      let ts_out_dir = {
        let rel_path_to_root = get_relative_path(&pkg_root, &monorepo_root)?;

        let root_out_dir = rel_path_to_root.join(".out");

        root_out_dir.join(&package_name)
      }
      .to_string_lossy()
      .to_string();

      let path_to_root_tsconfig =
        get_relative_path(&pkg_root, &monorepo_root)?.join("tsconfig.options.json");

      let base_tsconfig =
        get_default_package_tsconfig(path_to_root_tsconfig.to_string_lossy().to_string(), is_app);

      tsconfig_files.push(("tsconfig.json".to_string(), base_tsconfig));

      let src_tsconfig = get_default_src_tsconfig(is_app, &ts_out_dir);

      tsconfig_files.push(("tsconfig.src.json".to_string(), src_tsconfig));

      if !is_app {
        let dev_tsconfig = get_default_dev_tsconfig(&ts_out_dir);

        tsconfig_files.push(("tsconfig.dev.json".to_string(), dev_tsconfig));
      }
    }

    for (file, tsconfig) in tsconfig_files {
      serialize_json(&tsconfig, &pkg_root.join(file))?;
    }

    if update_root_tsconfig {
      let root_tsconfig_path = monorepo_root.join("tsconfig.json");

      let mut root_tsconfig: TsConfig = deserialize_json(&root_tsconfig_path)?;

      let path_to_new_tsconfig =
        get_relative_path(&monorepo_root, &pkg_root.join("tsconfig.json"))?;

      let root_tsconfig_references = root_tsconfig.references.get_or_insert_default();

      root_tsconfig_references.insert(ts_config::TsConfigReference {
        path: path_to_new_tsconfig.to_string_lossy().to_string(),
      });

      serialize_json(&root_tsconfig, &root_tsconfig_path)?;
    }

    let src_dir = pkg_root.join("src");
    create_all_dirs(&src_dir)?;

    let _index_file = open_file_for_writing(&src_dir.join("index.ts"))?;

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

      write_to_output!(vitest, file_path);
      write_to_output!(TestsSetupFile, tests_setup_dir.join("tests_setup.ts"));
    }

    if let Some(oxlint_config) = config.oxlint && oxlint_config.is_enabled() {
      let (id, oxlint_config) = match oxlint_config {
        OxlintConfigSetting::Bool(_) => ("__default".to_string(), OxlintPreset::default()),
        OxlintConfigSetting::Id(id) => (id.clone(), typescript.oxlint_presets.get(id.as_str()).ok_or(GenError::PresetNotFound { kind: Preset::Oxlint, name: id.clone() })?.clone()), 
        OxlintConfigSetting::Config(oxlint_preset) => (format!("__inlined_definition_{}", package_name), oxlint_preset),
      };

      let merged_config = oxlint_config.process_data(id.as_str(), &typescript.oxlint_presets)?;

      serialize_json(&merged_config, &pkg_root.join(".oxlintrc.json"))?;
    }

    if let Some(templates) = config.with_templates && !templates.is_empty() {
      self
        .generate_templates(&pkg_root, templates)?;
    }

    Ok(())
  }
}
