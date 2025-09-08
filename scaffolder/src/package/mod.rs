pub mod vitest;

use std::{
  fs::{create_dir_all, File},
  path::PathBuf,
};

use figment::{
  value::{Dict, Map},
  Error, Figment, Metadata, Profile, Provider,
};
use serde::{Deserialize, Serialize};

use crate::{
  moon::{MoonDotYml, MoonDotYmlKind},
  package::vitest::{TestsSetupFile, VitestConfig, VitestConfigStruct},
  package_json::{PackageJson, PackageJsonKind},
  paths::get_relative_path,
  pnpm::PnpmWorkspace,
  tera::TemplateOutput,
  ts_config::{tsconfig_defaults::*, TsConfig, TsConfigDirective, TsConfigKind},
  versions::get_latest_version,
  *,
};

#[derive(Debug, Deserialize, Serialize, Default, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum PackageKind {
  #[default]
  Library,
  App,
}

/// The configuration struct that is used to generate new packages.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct PackageConfig {
  /// The name of the package. It will be set as the name field in its package.json file.
  pub name: String,
  /// The kind of package (i.e. library of app).
  pub kind: PackageKind,
  /// The key to the package.json preset to use.
  pub package_json: Option<PackageJsonKind>,
  /// The configuration for the moon.yml file that will be generated for this package.
  pub moonrepo: Option<MoonDotYmlKind>,
  /// The directory for this package. This path will be joined to the `root_dir` setting in the global config.
  pub dir: String,
  /// The configuration for this package's vitest setup.
  pub vitest: VitestConfig,
  pub oxlint: Option<OxlintConfig>,
  /// The keys for the tsconfig files to generate for this package. If `use_default_tsconfigs` is set to true, the defaults will be appended to this list.
  pub ts_config: Option<Vec<TsConfigDirective>>,
  /// The out_dir for this package's tsconfig. Ignored if the default tsconfigs are not used.
  /// If it's unset and the shared_out_dir is set for the global config, it will resolve to the shared_out_dir, joined with a directory with this package's name.
  /// So if the shared_out_dir is ".out" and the name of the package is "my_pkg", the out_dir's default value will be `.out/my_pkg`.
  pub ts_out_dir: Option<String>,
  /// The templates to generate when this package is created.
  /// The paths specified for these templates' outputs will be joined to the package's directory.
  pub generate_templates: Vec<TemplateOutput>,
  /// If true, the root tsconfig.json file will be updated when this package is created, by adding the new tsconfig file to its list of references.
  pub update_root_tsconfig: bool,
}

impl Default for PackageConfig {
  fn default() -> Self {
    Self {
      name: "my-awesome-package".to_string(),
      kind: Default::default(),
      package_json: None,
      moonrepo: None,
      dir: ".".to_string(),
      vitest: Default::default(),
      ts_config: None,
      generate_templates: Default::default(),
      ts_out_dir: None,
      update_root_tsconfig: true,
      oxlint: None,
    }
  }
}

impl PackageConfig {
  // Allow the configuration to be extracted from any `Provider`.
  pub fn from<T: Provider>(provider: T) -> Result<PackageConfig, Error> {
    Figment::from(provider).extract()
  }

  // Provide a default provider, a `Figment`.
  pub fn figment() -> Figment {
    Figment::from(PackageConfig::default())
  }
}

// Make `Config` a provider itself for composability.
impl Provider for PackageConfig {
  fn metadata(&self) -> Metadata {
    Metadata::named("Package Generation Config")
  }

  fn data(&self) -> Result<Map<Profile, Dict>, Error> {
    figment::providers::Serialized::defaults(PackageConfig::default()).data()
  }
}

impl Config {
  /// Generate a new typescript package.
  pub async fn build_package(self, name: &str) -> Result<(), GenError> {
    let global_config = self;

    let package_json_presets = &global_config.package_json_presets;

    let root_dir = PathBuf::from(&global_config.root_dir);

    let config = global_config
      .package_presets
      .get(name)
      .ok_or(GenError::PresetNotFound {
        kind: Preset::Package,
        name: name.to_string(),
      })?
      .clone();

    let output = root_dir.join(&config.dir);

    create_dir_all(&output).map_err(|e| GenError::DirCreation {
      path: output.to_owned(),
      source: e,
    })?;

    macro_rules! write_to_output {
    ($($tokens:tt)*) => {
      write_file!(output, global_config.overwrite, $($tokens) *)
    };
  }

    let (package_json_id, mut package_json_data) = if let Some(package_json_config) =
      config.package_json.clone()
    {
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
        PackageJsonKind::Config(package_json_config) => (config.name.clone(), *package_json_config),
      }
    } else {
      ("__default".to_string(), PackageJson::default())
    };

    package_json_data = package_json_data.merge_configs(&package_json_id, package_json_presets)?;

    if package_json_data.package_manager.is_none() {
      package_json_data.package_manager = Some(global_config.package_manager.to_string());
    }

    get_contributors!(package_json_data, global_config, contributors);
    get_contributors!(package_json_data, global_config, maintainers);

    if package_json_data.use_default_deps {
      for dep in DEFAULT_DEPS {
        let version = if global_config.catalog {
          "catalog:".to_string()
        } else {
          let version = get_latest_version(dep).await.unwrap_or_else(|e| {
            println!(
              "Could not get the latest valid version range for '{}' due to the following error: {}.\nFalling back to 'latest'...",
              e,
              dep
            );
            "latest".to_string()
          });
          global_config.version_ranges.create(version)
        };

        package_json_data
          .dev_dependencies
          .insert(dep.to_string(), version);
      }
    }

    package_json_data.name = config.name.clone();

    if global_config.get_latest_version_range {
      package_json_data
        .get_latest_version_range(global_config.version_ranges)
        .await?;
    }

    write_to_output!(package_json_data, "package.json");

    if global_config.catalog && matches!(global_config.package_manager, PackageManager::Pnpm) {
      let pnpm_workspace_path = root_dir.join("pnpm-workspace.yaml");
      let pnpm_workspace_file = File::open(&pnpm_workspace_path)
        .map_err(|e| GenError::PnpmWorkspaceUpdate(e.to_string()))?;

      let mut pnpm_workspace: PnpmWorkspace = serde_yaml_ng::from_reader(&pnpm_workspace_file)
        .map_err(|e| GenError::PnpmWorkspaceUpdate(e.to_string()))?;

      pnpm_workspace
        .add_dependencies_to_catalog(global_config.version_ranges, &package_json_data)
        .await;

      pnpm_workspace
        .write_into(
          &mut File::create(root_dir.join("pnpm-workspace.yaml"))
            .map_err(|e| GenError::PnpmWorkspaceUpdate(e.to_string()))?,
        )
        .map_err(|e| GenError::PnpmWorkspaceUpdate(e.to_string()))?;
    }

    if let Some(ref moon_config_kind) = config.moonrepo && !matches!(moon_config_kind, MoonDotYmlKind::Bool(false)) {
      let moon_config = match moon_config_kind {
        MoonDotYmlKind::Bool(_) => MoonDotYml::default(),
        MoonDotYmlKind::Config(moon_dot_yml) => moon_dot_yml.clone(),
      };

      write_to_output!(moon_config, "moon.yml");
    }

    let mut tsconfig_files: Vec<(String, TsConfig)> = Default::default();

    let tsconfig_presets = &global_config.tsconfig_presets;

    if let Some(tsconfig_directives) = config.ts_config.clone() {
      for directive in tsconfig_directives {
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
          TsConfigKind::Config(ts_config) => (format!("__{}", config.name), *ts_config),
        };

        if !tsconfig.extend_presets.is_empty() {
          tsconfig = tsconfig.merge_configs(&id, tsconfig_presets)?;
        }

        tsconfig_files.push((directive.file_name, tsconfig));
      }
    } else {
      let is_app = matches!(config.kind, PackageKind::App);
      let out_dir = if let Some(ref ts_out_dir) = config.ts_out_dir {
        get_relative_path(&output, &PathBuf::from(ts_out_dir))?
      } else {
        get_relative_path(
          &output,
          &root_dir.join(
            global_config
              .shared_out_dir
              .get_name()
              .unwrap_or(".out".to_string()),
          ),
        )?
        .join(&config.name)
      }
      .to_string_lossy()
      .to_string();

      let rel_path_to_root_dir = get_relative_path(&output, &PathBuf::from(&root_dir))?
        .to_string_lossy()
        .to_string();
      let base_tsconfig = get_default_package_tsconfig(
        rel_path_to_root_dir,
        &global_config.project_tsconfig_name,
        is_app.then_some(&global_config.dev_tsconfig_name),
      );

      tsconfig_files.push(("tsconfig.json".to_string(), base_tsconfig));

      let src_tsconfig = get_default_src_tsconfig(is_app, &out_dir);

      tsconfig_files.push((global_config.project_tsconfig_name.clone(), src_tsconfig));

      if !is_app {
        let dev_tsconfig = get_default_dev_tsconfig(&global_config.project_tsconfig_name, &out_dir);

        tsconfig_files.push((global_config.dev_tsconfig_name.clone(), dev_tsconfig));
      }
    }

    if config.update_root_tsconfig {
      let root_tsconfig_path = PathBuf::from(&root_dir).join("tsconfig.json");
      let root_tsconfig_file =
        File::open(&root_tsconfig_path).map_err(|e| GenError::RootTsConfigUpdate(e.to_string()))?;

      let mut root_tsconfig: TsConfig = serde_json::from_reader(root_tsconfig_file)
        .map_err(|e| GenError::RootTsConfigUpdate(e.to_string()))?;

      let path_to_new_tsconfig = output.join("tsconfig.json").to_string_lossy().to_string();

      if let Some(root_tsconfig_references) = root_tsconfig.references.as_mut()
         && !root_tsconfig_references.iter().any(|p| p.path == path_to_new_tsconfig)
      {
        root_tsconfig_references.insert(ts_config::TsConfigReference { path: path_to_new_tsconfig });
        root_tsconfig.write_into(&mut File::create(&root_tsconfig_path)
          .map_err(|e| GenError::RootTsConfigUpdate(e.to_string()))?)
          .map_err(|e| GenError::RootTsConfigUpdate(e.to_string()))?;
      }
    }

    for (file, tsconfig) in tsconfig_files {
      write_to_output!(tsconfig, file);
    }

    let tests_setup_dir = output.join("tests/setup");
    create_dir_all(&tests_setup_dir).map_err(|e| GenError::DirCreation {
      path: tests_setup_dir,
      source: e,
    })?;

    let vitest_config = match config.vitest {
      VitestConfig::Boolean(v) => v.then(VitestConfigStruct::default),
      VitestConfig::Id(n) => {
        let vitest_presets = &global_config.vitest_presets;
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
      VitestConfig::Config(vitest_config_struct) => Some(vitest_config_struct),
    };

    if let Some(vitest) = vitest_config {
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

    if !config.generate_templates.is_empty() {
      global_config
        .generate_templates(&output.to_string_lossy(), config.generate_templates.clone())?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod test {
  use std::path::PathBuf;

  use crate::{config::Config, GenError};

  #[tokio::test]
  async fn package_test() -> Result<(), GenError> {
    let config = Config::init(PathBuf::from("config.toml"))?;

    config.build_package("alt2").await
  }

  #[tokio::test]
  async fn circular_package_json() -> Result<(), GenError> {
    let config = Config::init(PathBuf::from("tests/circular_package_json/config.toml"))?;

    let result = config.build_package("circular_package_json").await;

    match result {
      Ok(_) => panic!("Circular package json test did not fail as expected"),
      Err(e) => {
        if matches!(e, GenError::CircularDependency(_)) {
          Ok(())
        } else {
          panic!("Circular package json test returned wrong kind of error")
        }
      }
    }
  }

  #[tokio::test]
  async fn circular_tsconfig() -> Result<(), GenError> {
    let config = Config::init(PathBuf::from("tests/circular_tsconfigs/config.toml"))?;

    let result = config.build_package("circular_tsconfigs").await;

    match result {
      Ok(_) => panic!("Circular tsconfig test did not fail as expected"),
      Err(e) => {
        if matches!(e, GenError::CircularDependency(_)) {
          Ok(())
        } else {
          panic!("Circular tsconfig test returned wrong kind of error: {}", e)
        }
      }
    }
  }
}
