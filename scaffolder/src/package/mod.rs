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
  moon::MoonDotYml,
  package::vitest::{TestsSetupFile, VitestConfig, VitestConfigStruct},
  paths::get_relative_path,
  pnpm::PnpmWorkspace,
  tera::TemplateOutput,
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
  pub package_json: String,
  /// The configuration for the moon.yml file that will be generated for this package.
  pub moonrepo: Option<MoonDotYml>,
  /// The directory for this package. This path will be joined to the `root_dir` setting in the global config.
  pub dir: String,
  /// The configuration for this package's vitest setup.
  pub vitest: VitestConfig,
  /// The keys for the tsconfig files to generate for this package. If `use_default_tsconfigs` is set to true, the defaults will be appended to this list.
  pub ts_config: Vec<TsConfigDirective>,
  /// The out_dir for this package's tsconfig. Ignored if the default tsconfigs are not used.
  /// If it's unset and the shared_out_dir is set for the global config, it will resolve to the shared_out_dir, joined with a directory with this package's name.
  /// So if the shared_out_dir is ".out" and the name of the package is "my_pkg", the out_dir's default value will be `.out/my_pkg`.
  pub ts_out_dir: Option<String>,
  /// Use the default tsconfigs.
  pub use_default_tsconfigs: bool,
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
      package_json: Default::default(),
      moonrepo: None,
      dir: ".".to_string(),
      vitest: Default::default(),
      ts_config: Default::default(),
      use_default_tsconfigs: true,
      generate_templates: Default::default(),
      ts_out_dir: None,
      update_root_tsconfig: true,
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

    let mut package_json_data = package_json_presets
      .get(&config.package_json)
      .ok_or(GenError::PresetNotFound {
        kind: Preset::PackageJson,
        name: config.package_json.clone(),
      })?
      .clone();

    package_json_data =
      package_json_data.merge_configs(&config.package_json, package_json_presets)?;

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

    package_json_data.package_name = config.name.clone();

    write_to_output!(package_json_data, "package.json");

    if global_config.catalog && matches!(global_config.package_manager, PackageManager::Pnpm) {
      println!("Detected catalog = true. Dependencies marked with 'catalog:' will be added to pnpm-workspace.yaml if they are not already listed in their target catalog");

      let pnpm_workspace_path = root_dir.join("pnpm-workspace.yaml");
      let pnpm_workspace_file =
        File::open(&pnpm_workspace_path).map_err(|e| GenError::ReadError {
          path: pnpm_workspace_path.clone(),
          source: e,
        })?;

      let mut pnpm_workspace: PnpmWorkspace = serde_yaml_ng::from_reader(&pnpm_workspace_file)
        .map_err(|e| GenError::YamlDeserialization {
          path: pnpm_workspace_path.clone(),
          source: e,
        })?;

      pnpm_workspace
        .add_dependencies_to_catalog(global_config.version_ranges, &package_json_data)
        .await;

      pnpm_workspace
        .write_into(
          &mut File::create(root_dir.join("pnpm-workspace.yaml")).map_err(|e| {
            GenError::WriteError {
              path: pnpm_workspace_path.clone(),
              source: e,
            }
          })?,
        )
        .map_err(|e| GenError::WriteError {
          path: pnpm_workspace_path.clone(),
          source: e,
        })?;
    }

    if let Some(ref moon_config) = config.moonrepo {
      write_to_output!(moon_config, "moon.yml");
    }

    let mut tsconfig_files: Vec<(String, TsConfig)> = Default::default();

    if config.use_default_tsconfigs {
      let is_app = matches!(config.kind, PackageKind::App);
      let rel_path_to_root_dir = get_relative_path(&output, &PathBuf::from(&root_dir))?;
      let base_tsconfig = TsConfig {
        extends: Some(
          rel_path_to_root_dir
            .join(&global_config.root_tsconfig_name)
            .to_string_lossy()
            .to_string(),
        ),
        files: Some(vec![]),
        references: {
          let mut references = vec![TsConfigReference {
            path: global_config.project_tsconfig_name.clone(),
          }];
          if !is_app {
            references.push(TsConfigReference {
              path: global_config.dev_tsconfig_name.clone(),
            });
          }
          Some(references)
        },
        ..Default::default()
      };

      tsconfig_files.push(("tsconfig.json".to_string(), base_tsconfig));

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
      };

      let out_dir = out_dir.to_string_lossy().to_string();

      let src_ts_config = TsConfig {
        extends: Some("./tsconfig.json".to_string()),
        references: Some(vec![]),
        include: if is_app {
          Some(vec![
            "src".to_string(),
            "*.ts".to_string(),
            "tests".to_string(),
            "scripts".to_string(),
          ])
        } else {
          Some(vec!["src".to_string()])
        },
        compiler_options: Some(CompilerOptions {
          root_dir: Some("src".to_string()),
          out_dir: Some(out_dir.clone()),
          ts_build_info_file: Some(format!("{}/.tsBuildInfoSrc", out_dir)),
          no_emit: is_app.then_some(true),
          emit_declaration_only: (!is_app).then_some(true),
          ..Default::default()
        }),
        ..Default::default()
      };

      tsconfig_files.push((global_config.project_tsconfig_name.clone(), src_ts_config));

      if !is_app {
        let dev_tsconfig = TsConfig {
          extends: Some(global_config.project_tsconfig_name.clone()),
          include: Some(vec![
            "*.ts".to_string(),
            "tests".to_string(),
            "scripts".to_string(),
            "src".to_string(),
          ]),
          references: Some(vec![TsConfigReference {
            path: global_config.project_tsconfig_name.clone(),
          }]),
          compiler_options: Some(CompilerOptions {
            root_dir: Some(".".to_string()),
            no_emit: Some(true),
            ts_build_info_file: Some(format!("{}/.tsBuildInfoDev", out_dir)),
            ..Default::default()
          }),
          ..Default::default()
        };

        tsconfig_files.push((global_config.dev_tsconfig_name.clone(), dev_tsconfig));
      }
    }

    if config.update_root_tsconfig {
      println!("Detected update_root_tsconfig = true. The new tsconfig will be added as a reference to the tsconfig file in the root of the monorepo.");

      let root_tsconfig_path = PathBuf::from(&root_dir).join("tsconfig.json");
      let root_tsconfig_file =
        File::open(&root_tsconfig_path).map_err(|e| GenError::ReadError {
          path: root_tsconfig_path.clone(),
          source: e,
        })?;

      let mut root_tsconfig: TsConfig =
        serde_json::from_reader(root_tsconfig_file).map_err(|e| GenError::JsonDeserialization {
          path: root_tsconfig_path.clone(),
          source: e,
        })?;

      let path_to_new_tsconfig = output.join("tsconfig.json").to_string_lossy().to_string();

      if let Some(root_tsconfig_references) = root_tsconfig.references.as_mut()
         && !root_tsconfig_references.iter().any(|p| p.path == path_to_new_tsconfig)
      {
        root_tsconfig_references.push(ts_config::TsConfigReference { path: path_to_new_tsconfig });
        root_tsconfig.write_into(&mut File::create(&root_tsconfig_path)
          .map_err(|e| GenError::WriteError { path: root_tsconfig_path.clone(), source: e })?)
          .map_err(|e| GenError::WriteError { path: root_tsconfig_path.clone(), source: e })?;
      }
    }

    let tsconfig_presets = &global_config.tsconfig_presets;

    for directive in config.ts_config.clone() {
      let mut preset = tsconfig_presets
        .get(&directive.id)
        .ok_or(GenError::PresetNotFound {
          kind: Preset::TsConfig,
          name: directive.id.clone(),
        })?
        .clone();

      if !preset.extend_presets.is_empty() {
        preset = preset.merge_configs(&directive.id, tsconfig_presets)?;
      }

      tsconfig_files.push((directive.file_name, preset));
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
      VitestConfig::Named(n) => {
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
      VitestConfig::Definition(vitest_config_struct) => Some(vitest_config_struct),
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
