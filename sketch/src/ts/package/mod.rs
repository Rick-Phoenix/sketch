use std::{
  fs::{create_dir_all, File},
  path::PathBuf,
};

use askama::Template;
use clap::Parser;
use merge::Merge;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{
  package_json::{PackageJson, PackageJsonKind},
  pnpm::PnpmWorkspace,
  ts_config::{tsconfig_defaults::*, TsConfig, TsConfigDirective, TsConfigKind},
  vitest::{TestsSetupFile, VitestConfig, VitestConfigKind},
};
use crate::{
  custom_templating::TemplateOutput,
  merge_if_not_default, overwrite_option,
  paths::{create_parent_dirs, get_abs_path, get_cwd, get_relative_path},
  ts::{package_json::Person, ts_config, OxlintConfig, PackageManager},
  Config, GenError, Preset,
};

/// The kind of ts package.
#[derive(Debug, Deserialize, Serialize, Default, Clone, Copy, PartialEq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum PackageKind {
  #[default]
  Library,
  App,
}

#[derive(Debug, Clone, Serialize, Deserialize, Parser, Merge, PartialEq, JsonSchema)]
#[merge(strategy = overwrite_option)]
#[serde(default)]
pub struct RootPackage {
  /// The name of the root package [default: "root"].
  #[arg(short, long)]
  pub name: Option<String>,

  /// Oxlint configuration for the root package.
  /// Can be set to true (to use defaults) or false (to disable it) or to a string defining the configuration to generate.
  #[arg(skip)]
  pub oxlint: Option<OxlintConfig>,

  /// A list of [`TsConfigDirective`]s for the root package. They can be preset ids or literal configurations. If unset, defaults are used.
  #[arg(help = "One or many tsconfig files for the root package. If unset, defaults are used")]
  #[arg(short, long, value_parser = TsConfigDirective::from_cli, value_name = "output=PATH,id=ID")]
  pub ts_config: Option<Vec<TsConfigDirective>>,

  /// The [`PackageJsonKind`] to use for the root package. It can be a preset id or a literal definition.
  #[arg(short, long, value_parser = PackageJsonKind::from_cli)]
  #[arg(
    help = "The id of the package.json preset to use for the root package",
    value_name = "ID"
  )]
  pub package_json: Option<PackageJsonKind>,

  /// The templates to generate when the root package is generated.
  #[arg(skip)]
  pub with_templates: Option<Vec<TemplateOutput>>,
}

impl Default for RootPackage {
  fn default() -> Self {
    Self {
      name: None,
      oxlint: Some(Default::default()),
      ts_config: Default::default(),
      with_templates: Default::default(),
      package_json: Default::default(),
    }
  }
}

/// The configuration struct that is used to generate new packages.
#[derive(Clone, Debug, Deserialize, Serialize, Parser, Merge, PartialEq, JsonSchema)]
#[merge(strategy = overwrite_option)]
#[serde(default)]
pub struct PackageConfig {
  /// The new package's directory, starting from the [`Config::root_dir`]. Defaults to the name of the package.
  #[arg(
    value_name = "DIR",
    help = "The new package's directory, starting from the `root_dir`. Defaults to the name of the package"
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

  /// The [`PackageJsonKind`] to use for this package. It can be a preset id or a literal definition.
  #[arg(long, value_parser = PackageJsonKind::from_cli)]
  #[arg(
    help = "The id of the package.json preset to use for this package",
    value_name = "ID"
  )]
  pub package_json: Option<PackageJsonKind>,

  /// The templates to generate when this package is created.
  /// The paths specified for these templates' output paths will be joined to the package's directory.
  #[arg(skip)]
  pub with_templates: Option<Vec<TemplateOutput>>,

  /// The kind of package [default: 'library'].
  #[arg(skip)]
  pub kind: Option<PackageKind>,

  /// The configuration for this package's vitest setup. It can be set to false (to disable it), an id (to use a preset) or a literal configuration.
  #[arg(skip)]
  #[merge(strategy = merge_if_not_default)]
  pub vitest: VitestConfigKind,

  /// The configuration for this package's oxlint setup. It can be set to true (to use defaults), or to a literal value.
  #[arg(skip)]
  pub oxlint: Option<OxlintConfig>,
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

pub enum PackageData {
  Preset(String),
  Config(PackageConfig),
}

impl Config {
  /// Generate a new typescript package.
  pub async fn build_package(
    self,
    data: PackageData,
    update_root_tsconfig: bool,
  ) -> Result<(), GenError> {
    let typescript = self.typescript.clone().unwrap_or_default();

    let package_json_presets = &typescript.package_json_presets;

    let root_dir = self.root_dir.clone().unwrap_or_else(|| get_cwd());

    let config = match data {
      PackageData::Config(conf) => conf,
      PackageData::Preset(id) => typescript
        .package_presets
        .get(&id)
        .ok_or(GenError::PresetNotFound {
          kind: Preset::Package,
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

    let mut output = root_dir.join(
      config
        .dir
        .clone()
        .unwrap_or_else(|| package_name.clone().into()),
    );

    create_dir_all(&output).map_err(|e| GenError::DirCreation {
      path: output.to_owned(),
      source: e,
    })?;

    output = get_abs_path(&output)?;

    let package_manager = typescript.package_manager.unwrap_or_default();
    let version_ranges = typescript.version_range.unwrap_or_default();

    macro_rules! write_to_output {
      ($($tokens:tt)*) => {
        write_file!(output, !self.no_overwrite, $($tokens)*)
      };
    }

    let (package_json_id, mut package_json_data) =
      if let Some(package_json_config) = config.package_json.as_ref() {
        match package_json_config {
          PackageJsonKind::Id(id) => {
            let config = package_json_presets
              .get(id)
              .ok_or(GenError::PresetNotFound {
                kind: Preset::PackageJson,
                name: id.clone(),
              })?
              .clone();

            (id.to_string(), config)
          }
          PackageJsonKind::Config(package_json_config) => {
            (package_name.clone(), *package_json_config.clone())
          }
        }
      } else {
        ("__default".to_string(), PackageJson::default())
      };

    package_json_data = package_json_data.merge_configs(&package_json_id, package_json_presets)?;

    if package_json_data.package_manager.is_none() {
      package_json_data.package_manager = Some(package_manager.to_string());
    }

    get_contributors!(package_json_data, typescript, contributors);
    get_contributors!(package_json_data, typescript, maintainers);

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

    package_json_data.name = package_name.clone();

    if !typescript.no_convert_latest_to_range {
      package_json_data
        .get_latest_version_range(version_ranges)
        .await?;
    }

    write_to_output!(package_json_data, "package.json");

    if typescript.catalog && matches!(package_manager, PackageManager::Pnpm) {
      let pnpm_workspace_path = root_dir.join("pnpm-workspace.yaml");
      let pnpm_workspace_file = File::open(&pnpm_workspace_path)
        .map_err(|e| GenError::PnpmWorkspaceUpdate(e.to_string()))?;

      let mut pnpm_workspace: PnpmWorkspace = serde_yaml_ng::from_reader(&pnpm_workspace_file)
        .map_err(|e| GenError::PnpmWorkspaceUpdate(e.to_string()))?;

      pnpm_workspace
        .add_dependencies_to_catalog(version_ranges, &package_json_data)
        .await;

      pnpm_workspace
        .write_into(
          &mut File::create(root_dir.join("pnpm-workspace.yaml"))
            .map_err(|e| GenError::PnpmWorkspaceUpdate(e.to_string()))?,
        )
        .map_err(|e| GenError::PnpmWorkspaceUpdate(e.to_string()))?;
    }

    let mut tsconfig_files: Vec<(String, TsConfig)> = Default::default();

    let tsconfig_presets = &typescript.tsconfig_presets;

    if let Some(tsconfig_directives) = config.ts_config.clone() {
      for directive in tsconfig_directives {
        let (id, mut tsconfig) = match directive.config.unwrap_or_default() {
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
          TsConfigKind::Config(ts_config) => (format!("__{}", package_name), *ts_config.clone()),
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
      let is_app = matches!(config.kind.unwrap_or_default(), PackageKind::App);
      let out_dir = {
        let rel_path_to_root = get_relative_path(&output, &root_dir)?;

        let root_out_dir = rel_path_to_root.join(".out");

        root_out_dir.join(&package_name)
      }
      .to_string_lossy()
      .to_string();

      let path_to_root_tsconfig = get_relative_path(&output, &root_dir)?
        .join("tsconfig.options.json")
        .to_string_lossy()
        .to_string();

      let base_tsconfig = get_default_package_tsconfig(path_to_root_tsconfig, is_app);

      tsconfig_files.push(("tsconfig.json".to_string(), base_tsconfig));

      let src_tsconfig = get_default_src_tsconfig(is_app, &out_dir);

      tsconfig_files.push(("tsconfig.src.json".to_string(), src_tsconfig));

      if !is_app {
        let dev_tsconfig = get_default_dev_tsconfig(&out_dir);

        tsconfig_files.push(("tsconfig.dev.json".to_string(), dev_tsconfig));
      }
    }

    if self.debug {
      eprintln!("DEBUG:");
      eprintln!("  package {:#?}", config);
    }

    for (file, tsconfig) in tsconfig_files {
      write_to_output!(tsconfig, file);
    }

    if update_root_tsconfig {
      let root_tsconfig_path = root_dir.join("tsconfig.json");

      let root_tsconfig_file =
        File::open(&root_tsconfig_path).map_err(|e| GenError::RootTsConfigUpdate(e.to_string()))?;

      let mut root_tsconfig: TsConfig = serde_json::from_reader(root_tsconfig_file)
        .map_err(|e| GenError::RootTsConfigUpdate(e.to_string()))?;

      let path_to_new_tsconfig = get_relative_path(&root_dir, &output.join("tsconfig.json"))?;

      let root_tsconfig_references = root_tsconfig.references.get_or_insert_default();

      root_tsconfig_references.insert(ts_config::TsConfigReference {
        path: path_to_new_tsconfig.to_string_lossy().to_string(),
      });

      root_tsconfig
        .write_into(
          &mut File::create(&root_tsconfig_path)
            .map_err(|e| GenError::RootTsConfigUpdate(e.to_string()))?,
        )
        .map_err(|e| GenError::RootTsConfigUpdate(e.to_string()))?;
    }

    let src_dir = output.join("src");
    create_dir_all(&src_dir).map_err(|e| GenError::DirCreation {
      path: src_dir.clone(),
      source: e,
    })?;

    File::create(src_dir.join("index.ts")).map_err(|e| GenError::FileCreation {
      path: src_dir.join("index.ts"),
      source: e,
    })?;

    let vitest_config = match config.vitest {
      VitestConfigKind::Bool(v) => v.then(VitestConfig::default),
      VitestConfigKind::Config(vitest_config_struct) => Some(vitest_config_struct),
    };

    if let Some(mut vitest) = vitest_config.clone() {
      let tests_dir = output.join(&vitest.tests_dir);
      let tests_setup_dir = tests_dir.join(&vitest.setup_dir);

      create_parent_dirs(&tests_dir)?;
      create_parent_dirs(&tests_setup_dir)?;

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
      write_to_output!(TestsSetupFile {}, tests_setup_dir.join("tests_setup.ts"));
    }

    if let Some(oxlint_config) = config.oxlint.clone() {
      write_to_output!(oxlint_config, ".oxlintrc.json");
    }

    if let Some(templates) = config.with_templates && !templates.is_empty() {
      self
        .generate_templates(&output, templates)?;
    }

    Ok(())
  }
}
