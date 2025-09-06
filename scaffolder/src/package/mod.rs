pub mod vitest;

use std::{
  fs::{create_dir_all, File},
  path::PathBuf,
};

use figment::{
  value::{Dict, Map},
  Error, Figment, Metadata, Profile, Provider,
};
use merge::Merge;
use serde::{Deserialize, Serialize};

use crate::{
  moon::MoonDotYml,
  package::vitest::{TestsSetupFile, VitestConfig, VitestConfigStruct},
  paths::get_relative_path,
  pnpm::PnpmWorkspace,
  tera::TemplateOutput,
  versions::get_latest_version,
  PackageJsonData, *,
};

#[derive(Debug, Deserialize, Serialize, Default, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum PackageKind {
  #[default]
  Library,
  App,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct PackageConfig {
  pub name: String,
  pub kind: PackageKind,
  pub package_json: PackageJsonData,
  pub moonrepo: Option<MoonDotYml>,
  pub dir: String,
  pub vitest: VitestConfig,
  pub ts_config: Vec<TsConfigDirective>,
  pub ts_out_dir: Option<String>,
  pub use_default_tsconfigs: bool,
  pub generate_templates: Vec<TemplateOutput>,
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
  fn from<T: Provider>(provider: T) -> Result<PackageConfig, Error> {
    Figment::from(provider).extract()
  }

  // Provide a default provider, a `Figment`.
  pub fn figment() -> Figment {
    use figment::providers::Env;

    // In reality, whatever the library desires.
    Figment::from(PackageConfig::default()).merge(Env::prefixed("APP_"))
  }
}

// Make `Config` a provider itself for composability.
impl Provider for PackageConfig {
  fn metadata(&self) -> Metadata {
    Metadata::named("Library Config")
  }

  fn data(&self) -> Result<Map<Profile, Dict>, Error> {
    figment::providers::Serialized::defaults(PackageConfig::default()).data()
  }
}

impl Config {
  pub async fn build_package(mut self, name: &str) -> Result<(), GenError> {
    self.merge_configs()?;

    let global_config = self;

    let package_json_presets = global_config.package_json_presets.clone();

    let root_dir = PathBuf::from(&global_config.root_dir);

    let config = &global_config
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

    let mut package_json_data = match &config.package_json {
      PackageJsonData::Named(name) => package_json_presets
        .get(name)
        .ok_or(GenError::PresetNotFound {
          kind: Preset::PackageJson,
          name: name.clone(),
        })?
        .clone(),
      PackageJsonData::Definition(package_json) => package_json.clone(),
    };

    for id in package_json_data.extends.clone() {
      let target_to_extend = package_json_presets
        .get(&id)
        .ok_or(GenError::PresetNotFound {
          kind: Preset::PackageJson,
          name: id.clone(),
        })?
        .clone();

      package_json_data.merge(target_to_extend);
    }

    if package_json_data.package_manager.is_none() {
      package_json_data.package_manager = Some(global_config.package_manager.to_string());
    }

    get_contributors!(package_json_data, global_config, contributors);
    get_contributors!(package_json_data, global_config, maintainers);

    if package_json_data.default_deps {
      for dep in DEFAULT_DEPS {
        let version = if global_config.catalog {
          "catalog:".to_string()
        } else {
          let version = get_latest_version(dep).await.unwrap_or_else(|_| {
            println!(
              "Could not get the latest valid version range for '{}'. Falling back to 'latest'...",
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

    if global_config.catalog {
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

    package_json_data.package_name = config.name.clone();

    write_to_output!(package_json_data, "package.json");

    if let Some(ref moon_config) = config.moonrepo {
      write_to_output!(moon_config, "moon.yml");
    }

    let mut tsconfig_files = config.ts_config.clone();

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

      tsconfig_files.push(TsConfigDirective::Definition {
        file_name: "tsconfig.json".to_string(),
        config: base_tsconfig,
      });

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

      tsconfig_files.push(TsConfigDirective::Definition {
        file_name: global_config.project_tsconfig_name.clone(),
        config: src_ts_config,
      });

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

        tsconfig_files.push(TsConfigDirective::Definition {
          file_name: global_config.dev_tsconfig_name.clone(),
          config: dev_tsconfig,
        });
      }
    }

    if config.update_root_tsconfig {
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

    for directive in tsconfig_files {
      let output_file = &directive.get_file_name().to_string();

      let content = match directive {
        TsConfigDirective::Preset { preset_id, .. } => {
          let presets_store = &global_config.tsconfig_presets;
          presets_store
            .get(&preset_id)
            .ok_or(GenError::PresetNotFound {
              kind: Preset::TsConfig,
              name: preset_id,
            })?
            .clone()
        }
        TsConfigDirective::Definition { config, .. } => config,
        TsConfigDirective::Merged {
          extends, config, ..
        } => {
          let presets_store = &global_config.tsconfig_presets;
          let mut config = config.unwrap_or_default();

          for preset in extends {
            let target_to_extend = presets_store
              .get(&preset)
              .ok_or(GenError::PresetNotFound {
                kind: Preset::TsConfig,
                name: preset,
              })?
              .clone();

            config.merge(target_to_extend);
          }

          config
        }
      };

      write_to_output!(content, output_file);
    }

    let tests_setup_dir = output.join("tests/setup");
    create_dir_all(&tests_setup_dir).map_err(|e| GenError::DirCreation {
      path: tests_setup_dir,
      source: e,
    })?;

    let vitest_config = match config.vitest.clone() {
      VitestConfig::Boolean(v) => v.then(VitestConfigStruct::default),
      VitestConfig::Named(n) => {
        let mut vitest_presets = global_config.vitest_presets.clone();
        Some(vitest_presets.remove(&n).expect("Vitest preset not found"))
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
        text: "console.log(`they're taking the hobbits to isengard!`)".to_string()
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
  use crate::{config::Config, GenError};

  #[tokio::test]
  async fn package_test() -> Result<(), GenError> {
    let config: Config = Config::figment()
      .extract()
      .map_err(|e| GenError::ConfigParsing { source: e })?;

    config.build_package("alt2").await
  }
}
