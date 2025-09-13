pub mod vitest;

use std::{
  fs::{create_dir_all, File},
  path::PathBuf,
};

use clap::Parser;
use indexmap::IndexSet;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
  custom_templating::TemplateOutput,
  moon::{MoonDotYml, MoonDotYmlKind},
  package::vitest::{TestsSetupFile, VitestConfig, VitestConfigKind},
  package_json::{PackageJson, PackageJsonKind},
  paths::{get_cwd, get_relative_path},
  pnpm::PnpmWorkspace,
  ts_config::{tsconfig_defaults::*, TsConfig, TsConfigDirective, TsConfigKind},
  *,
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
#[merge(strategy = overwrite_option)]
#[serde(default)]
pub struct PackageConfig {
  /// A list of [`PackageConfig`]s to extend, in the given order.
  #[arg(skip)]
  #[merge(strategy = merge_index_sets)]
  extends: IndexSet<String>,

  /// The new package's directory, starting from the [`Config::root_dir`]. Defaults to the name of the package.
  #[arg(
    value_name = "DIR",
    help = "The new package's directory, starting from the `root_dir`. Defaults to the name of the package"
  )]
  pub dir: Option<PathBuf>,

  /// The name of the package. If `dir` is set, it defaults to the last segment of it.
  #[arg(skip)]
  pub name: Option<String>,

  /// A list of [`TsConfigDirective`]s for this package. They can be preset ids or literal configurations. If unset, defaults are used.
  #[arg(short, long, value_parser = TsConfigDirective::from_cli)]
  #[arg(
    help = "One or many tsconfig files for this package. If unset, defaults are used",
    value_name = "output=PATH,id=ID"
  )]
  #[merge(strategy = merge_optional_vecs)]
  pub ts_config: Option<Vec<TsConfigDirective>>,

  /// The out_dir for this package's tsconfig. Ignored if the default tsconfigs are not used.
  /// If it's unset and the shared_out_dir is set for the global config, it will resolve to the shared_out_dir, joined with a directory with this package's name.
  /// So if the shared_out_dir is 'root_dir/.out' and the name of the package is "my_pkg" (situated in root_dir/my_pkg), the out_dir's default value will be '../.out/my_pkg'.
  #[arg(long)]
  #[arg(
    help = "The out_dir for this package's tsconfig. Ignored if the default tsconfigs are not used",
    value_name = "DIR"
  )]
  pub ts_out_dir: Option<PathBuf>,

  /// The [`PackageJsonKind`] to use for this package. It can be a preset id or a literal definition.
  #[arg(short, long, value_parser = PackageJsonKind::from_cli)]
  #[arg(
    help = "The id of the package.json preset to use for this package",
    value_name = "ID"
  )]
  pub package_json: Option<PackageJsonKind>,

  /// The templates to generate when this package is created.
  /// The paths specified for these templates' output paths will be joined to the package's directory.
  #[arg(skip)]
  #[merge(strategy = merge_optional_vecs)]
  pub generate_templates: Option<Vec<TemplateOutput>>,

  /// The kind of package [default: 'library'].
  #[arg(skip)]
  pub kind: Option<PackageKind>,

  /// Configuration for the moon.yml file. It can be a boolean, to use defaults, or a full configuration.
  #[arg(skip)]
  pub moonrepo: Option<MoonDotYmlKind>,

  /// The configuration for this package's vitest setup. It can be set to false (to disable it), an id (to use a preset) or a literal configuration.
  #[arg(skip)]
  #[merge(strategy = merge_if_not_default)]
  pub vitest: VitestConfigKind,

  /// The configuration for this package's oxlint setup. It can be set to true (to use defaults), or to a literal value.
  #[arg(skip)]
  pub oxlint: Option<OxlintConfig>,

  /// Adds the new package to the references in the root tsconfig.
  #[arg(short = 'u', long)]
  #[merge(strategy = merge::bool::overwrite_false)]
  pub update_root_tsconfig: bool,
}

impl Default for PackageConfig {
  fn default() -> Self {
    Self {
      extends: Default::default(),
      name: None,
      kind: Default::default(),
      package_json: None,
      moonrepo: None,
      dir: Default::default(),
      vitest: Default::default(),
      ts_config: None,
      generate_templates: Default::default(),
      ts_out_dir: None,
      update_root_tsconfig: false,
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
  pub async fn build_package(self, data: PackageData) -> Result<(), GenError> {
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

    let package_name = if let Some(name) = config.name {
      name
    } else if let Some(dir) = config.dir.as_ref() && let Some(base) = dir.file_name() {
      base.to_string_lossy().to_string()
    } else {
      "my-awesome-package".to_string()
    };

    let output = root_dir.join(config.dir.unwrap_or_else(|| package_name.clone().into()));

    create_dir_all(&output).map_err(|e| GenError::DirCreation {
      path: output.to_owned(),
      source: e,
    })?;

    let package_manager = typescript.package_manager.unwrap_or_default();
    let version_ranges = typescript.version_range.unwrap_or_default();

    macro_rules! write_to_output {
      ($($tokens:tt)*) => {
        write_file!(output, !self.no_overwrite, $($tokens)*)
      };
    }

    let (package_json_id, mut package_json_data) =
      if let Some(package_json_config) = config.package_json {
        match package_json_config {
          PackageJsonKind::Id(id) => {
            let config = package_json_presets
              .get(&id)
              .ok_or(GenError::PresetNotFound {
                kind: Preset::PackageJson,
                name: id.clone(),
              })?
              .clone();

            (id, config)
          }
          PackageJsonKind::Config(package_json_config) => {
            (package_name.clone(), *package_json_config)
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

    if package_json_data.use_default_deps {
      for dep in DEFAULT_DEPS {
        let version = if typescript.no_catalog {
          "latest".to_string()
        } else {
          "catalog:".to_string()
        };

        package_json_data
          .dev_dependencies
          .insert(dep.to_string(), version);
      }
    }

    package_json_data.name = package_name.clone();

    if !typescript.no_convert_latest_to_range {
      package_json_data
        .get_latest_version_range(version_ranges)
        .await?;
    }

    write_to_output!(package_json_data, "package.json");

    if !typescript.no_catalog && matches!(package_manager, PackageManager::Pnpm) {
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

    if let Some(moon_config_kind) = config.moonrepo && !matches!(moon_config_kind, MoonDotYmlKind::Bool(false)) {
      let moon_config = match moon_config_kind {
        MoonDotYmlKind::Bool(_) => MoonDotYml::default(),
        MoonDotYmlKind::Config(moon_dot_yml) => moon_dot_yml,
      };

      write_to_output!(moon_config, "moon.yml");
    }

    let mut tsconfig_files: Vec<(String, TsConfig)> = Default::default();

    let tsconfig_presets = &typescript.tsconfig_presets;

    if let Some(tsconfig_directives) = config.ts_config {
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

            (id, tsconfig)
          }
          TsConfigKind::Config(ts_config) => (format!("__{}", package_name), *ts_config),
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
      let out_dir = if let Some(ts_out_dir) = config.ts_out_dir {
        output.join(ts_out_dir)
      } else {
        let rel_path_to_root = get_relative_path(&output, &root_dir)?;

        let root_out_dir = rel_path_to_root.join(
          typescript
            .shared_out_dir
            .get_name()
            .unwrap_or(".out".to_string()),
        );

        root_out_dir.join(&package_name)
      }
      .to_string_lossy()
      .to_string();

      let path_to_root_tsconfig = get_relative_path(&output, &root_dir)?
        .join(
          typescript
            .root_tsconfig_name
            .unwrap_or_else(|| "tsconfig.options.json".to_string()),
        )
        .to_string_lossy()
        .to_string();

      let base_tsconfig = get_default_package_tsconfig(
        path_to_root_tsconfig,
        typescript
          .project_tsconfig_name
          .as_ref()
          .map_or("tsconfig.src.json", |v| v),
        (!is_app).then_some(
          typescript
            .dev_tsconfig_name
            .as_ref()
            .map_or("tsconfig.dev.json", |v| v),
        ),
      );

      tsconfig_files.push(("tsconfig.json".to_string(), base_tsconfig));

      let src_tsconfig = get_default_src_tsconfig(is_app, &out_dir);

      tsconfig_files.push((
        typescript
          .project_tsconfig_name
          .clone()
          .unwrap_or_else(|| "tsconfig.src.json".to_string()),
        src_tsconfig,
      ));

      if !is_app {
        let dev_tsconfig = get_default_dev_tsconfig(
          typescript
            .project_tsconfig_name
            .as_ref()
            .map_or("tsconfig.src.json", |v| v),
          &out_dir,
        );

        tsconfig_files.push((
          typescript
            .dev_tsconfig_name
            .unwrap_or_else(|| "tsconfig.dev.json".to_string()),
          dev_tsconfig,
        ));
      }
    }

    for (file, tsconfig) in tsconfig_files {
      write_to_output!(tsconfig, file);
    }

    if config.update_root_tsconfig {
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

    let vitest_config = match config.vitest {
      VitestConfigKind::Bool(v) => v.then(VitestConfig::default),
      VitestConfigKind::Id(n) => {
        let vitest_presets = &typescript.vitest_presets;
        Some(
          vitest_presets
            .get(&n)
            .ok_or(GenError::PresetNotFound {
              kind: Preset::Vitest,
              name: n,
            })?
            .clone(),
        )
      }
      VitestConfigKind::Config(vitest_config_struct) => Some(vitest_config_struct),
    };

    if let Some(vitest) = vitest_config {
      let tests_setup_dir = output.join("tests/setup");
      create_dir_all(&tests_setup_dir).map_err(|e| GenError::DirCreation {
        path: tests_setup_dir,
        source: e,
      })?;

      write_to_output!(vitest, "tests/vitest.config.ts");
      write_to_output!(TestsSetupFile {}, "tests/setup/tests_setup.ts");
    }

    let src_dir = output.join("src");
    create_dir_all(&src_dir).map_err(|e| GenError::DirCreation {
      path: src_dir,
      source: e,
    })?;

    write_to_output!(
      GenericTemplate {
        text: "console.log(\"They're taking the hobbits to Isengard!\");".to_string()
      },
      "src/index.ts"
    );

    if let Some(oxlint_config) = config.oxlint.clone() {
      write_to_output!(oxlint_config, ".oxlintrc.json");
    }

    if let Some(templates) = config.generate_templates && !templates.is_empty() {
      self
        .generate_templates(&output, templates)?;
    }

    Ok(())
  }
}

impl PackageConfig {
  fn merge_configs_recursive(
    &mut self,
    store: &IndexMap<String, PackageConfig>,
    processed_ids: &mut IndexSet<String>,
  ) -> Result<(), GenError> {
    for id in self.extends.clone() {
      let was_absent = processed_ids.insert(id.clone());

      if !was_absent {
        let chain: Vec<&str> = processed_ids.iter().map(|s| s.as_str()).collect();

        return Err(GenError::CircularDependency(format!(
          "Found circular dependency for package preset '{}'. The full processed chain is: {}",
          id,
          chain.join(" -> ")
        )));
      }

      let mut target = store
        .get(id.as_str())
        .ok_or(GenError::PresetNotFound {
          kind: Preset::Package,
          name: id.to_string(),
        })?
        .clone();

      target.merge_configs_recursive(store, processed_ids)?;

      self.merge(target);
    }

    Ok(())
  }

  pub fn merge_configs(
    mut self,
    initial_id: &str,
    store: &IndexMap<String, PackageConfig>,
  ) -> Result<PackageConfig, GenError> {
    let mut processed_ids: IndexSet<String> = Default::default();

    processed_ids.insert(initial_id.to_string());

    self.merge_configs_recursive(store, &mut processed_ids)?;

    Ok(self)
  }
}
