pub mod vitest;

use std::{
  fs::{create_dir_all, File},
  path::PathBuf,
};

use clap::Parser;
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
      name: None,
      kind: Default::default(),
      package_json: None,
      moonrepo: None,
      dir: Default::default(),
      vitest: Default::default(),
      ts_config: None,
      generate_templates: Default::default(),
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

    let package_name = if let Some(name) = config.name.as_ref() {
      name.clone()
    } else if let Some(dir) = config.dir.as_ref() && let Some(base) = dir.file_name() {
      base.to_string_lossy().to_string()
    } else {
      "my-awesome-package".to_string()
    };

    let output = root_dir.join(
      config
        .dir
        .clone()
        .unwrap_or_else(|| package_name.clone().into()),
    );

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

    if package_json_data.use_default_deps {
      for dep in DEFAULT_DEPS {
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

    if let Some(moon_config_kind) = config.moonrepo.as_ref() && !matches!(moon_config_kind, MoonDotYmlKind::Bool(false)) {
      let moon_config = match moon_config_kind {
        MoonDotYmlKind::Bool(_) => MoonDotYml::default(),
        MoonDotYmlKind::Config(moon_dot_yml) => moon_dot_yml.clone(),
      };

      write_to_output!(moon_config, "moon.yml");
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

    if self.debug {
      eprintln!("DEBUG:");
      eprintln!("  package {:#?}", config);
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
