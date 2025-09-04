use std::io::Write;
pub mod vitest;

use std::{
  fs::{create_dir_all, File},
  path::PathBuf,
};

use figment::{
  providers::Toml,
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

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct PackageConfig {
  pub name: String,
  pub kind: PackageKind,
  pub package_json: PackageJsonData,
  pub moonrepo: Option<MoonDotYml>,
  pub dir: String,
  pub vitest: VitestConfig,
  pub ts_config: Vec<(String, TsConfig)>,
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

pub async fn build_package(name: &str) -> Result<(), Box<dyn std::error::Error>> {
  let global_config: Config = Config::figment()
    .merge(Toml::file("scaffolder/config.toml"))
    .extract()?;

  let mut package_json_presets = global_config.package_json_presets;

  let root_dir = PathBuf::from(global_config.root_dir);

  let mut packages = global_config.package_presets;
  let config = packages
    .remove(name)
    .unwrap_or_else(|| panic!("Could not find the package preset for '{}'", name));

  let output = root_dir.join(config.dir);

  create_dir_all(&output).map_err(|e| TemplateError::DirCreation {
    path: output.to_owned(),
    source: e,
  })?;

  macro_rules! write_file {
    ($data:expr, $suffix:expr) => {
      let path = output.join($suffix);
      let mut file = File::create(&path).map_err(|e| TemplateError::FileCreation {
        path: path.to_owned(),
        source: e,
      })?;
      $data.write_into(&mut file)?;
    };
  }

  let mut package_json_data = match config.package_json {
    PackageJsonData::Named(name) => package_json_presets
      .remove(&name)
      .expect("Package json not found in store"),
    PackageJsonData::Definition(package_json) => package_json,
  };

  for id in package_json_data.extends.clone() {
    let target_to_extend = package_json_presets
      .get(&id)
      .unwrap_or_else(|| panic!("Could not find the package json preset for '{}'", id))
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
    let pnpm_workspace_file =
      File::open(root_dir.join("pnpm-workspace.yaml")).expect("pnpm-workspace not found");

    let mut pnpm_workspace: PnpmWorkspace = serde_yaml_ng::from_reader(&pnpm_workspace_file)
      .expect("Could not deserialize pnpm-workspace");

    pnpm_workspace
      .add_dependencies_to_catalog(global_config.version_ranges, &package_json_data)
      .await;

    pnpm_workspace.write_into(&mut File::create(root_dir.join("pnpm-workspace.yaml"))?)?;
  }

  write_file!(package_json_data, "package.json");

  if let Some(moon_config) = config.moonrepo {
    write_file!(moon_config, "moon.yml");
  }

  let mut tsconfig_files = config.ts_config;

  let rel_path_to_root_dir = get_relative_path(&output, &PathBuf::from(&root_dir))?;
  let is_app = matches!(config.kind, PackageKind::App);
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

  let root_tsconfig_path = PathBuf::from(&root_dir).join("tsconfig.json");
  let root_tsconfig_file = File::open(&root_tsconfig_path)?;

  let mut root_tsconfig: TsConfig = serde_json::from_reader(root_tsconfig_file)?;

  let path_to_new_tsconfig = output.join("tsconfig.json").to_string_lossy().to_string();

  if let Some(root_tsconfig_references) = root_tsconfig.references.as_mut() && !root_tsconfig_references.iter().any(|p| p.path == path_to_new_tsconfig)
  {

    root_tsconfig_references.push(ts_config::TsConfigReference { path: path_to_new_tsconfig });
    root_tsconfig.write_into(&mut File::create(root_tsconfig_path)?)?;
  }

  let out_dir = get_relative_path(&output, &root_dir.join(global_config.out_dir))?
    .join(PathBuf::from(&config.name))
    .to_string_lossy()
    .to_string();

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
      ts_build_info_file: Some(format!("{}/{}/.tsBuildInfoSrc", out_dir, config.name)),
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
        ts_build_info_file: Some(format!("{}/{}/.tsBuildInfoDev", out_dir, config.name)),
        ..Default::default()
      }),
      ..Default::default()
    };

    tsconfig_files.push((global_config.dev_tsconfig_name.clone(), dev_tsconfig));
  }

  for (file, content) in tsconfig_files {
    content.write_into(&mut File::create(PathBuf::from(file))?)?;
  }

  create_dir_all(output.join("tests/setup"))?;

  let vitest_config = match config.vitest {
    VitestConfig::Boolean(v) => v.then(VitestConfigStruct::default),
    VitestConfig::Named(n) => {
      let mut vitest_presets = global_config.vitest_presets;
      Some(vitest_presets.remove(&n).expect("Vitest preset not found"))
    }
    VitestConfig::Definition(vitest_config_struct) => Some(vitest_config_struct),
  };

  if let Some(vitest) = vitest_config {
    write_file!(vitest, "tests/vitest.config.ts");
    write_file!(TestsSetupFile {}, "tests/setup/tests_setup.ts");
  }

  create_dir_all(output.join("src"))?;

  let mut index_file = File::create(output.join("src/index.ts"))?;

  write!(
    index_file,
    "console.log(`they're taking the hobbits to isengard!`)"
  )?;

  Ok(())
}

#[cfg(test)]
mod test {
  use crate::package::build_package;

  #[tokio::test]
  async fn package_test() -> Result<(), Box<dyn std::error::Error>> {
    build_package("alt2").await
  }

  #[test]
  fn test_tsconfig_generation() {
    // let ts_config = TsConfig {
    //   compiler_options: Some(CompilerOptions {
    //     out_dir: Some("out".to_string()),
    //     allow_js: Some(true),
    //     check_js: Some(true),
    //     composite: Some(true),
    //     declaration: Some(true),
    //     declaration_map: Some(true),
    //     downlevel_iteration: Some(true),
    //     import_helpers: Some(true),
    //     incremental: Some(true),
    //     isolated_modules: Some(true),
    //     no_emit: Some(true),
    //     remove_comments: Some(true),
    //     source_map: Some(true),
    //     always_strict: Some(true),
    //     no_implicit_any: Some(true),
    //     no_implicit_this: Some(true),
    //     strict_bind_call_apply: Some(true),
    //     strict_function_types: Some(true),
    //     strict: Some(true),
    //     strict_null_checks: Some(true),
    //     strict_property_initialization: Some(true),
    //     allow_synthetic_default_imports: Some(true),
    //     allow_umd_global_access: Some(true),
    //     es_module_interop: Some(true),
    //     preserve_symlinks: Some(true),
    //     inline_source_map: Some(true),
    //     inline_sources: Some(true),
    //     no_fallthrough_cases_in_switch: Some(true),
    //     no_implicit_returns: Some(true),
    //     no_unchecked_indexed_access: Some(true),
    //     no_unused_locals: Some(true),
    //     emit_decorator_metadata: Some(true),
    //     experimental_decorators: Some(true),
    //     allow_unreachable_code: Some(true),
    //     allow_unused_labels: Some(true),
    //     assume_changes_only_affect_direct_dependencies: Some(true),
    //     disable_referenced_project_load: Some(true),
    //     disable_size_limit: Some(true),
    //     max_node_module_js_depth: Some(15),
    //     disable_solution_searching: Some(true),
    //     disable_source_of_project_reference_redirect: Some(true),
    //     emit_declaration_only: Some(true),
    //     emit_bom: Some(true),
    //     explain_files: Some(true),
    //     extended_diagnostics: Some(true),
    //     force_consistent_casing_in_file_names: Some(true),
    //     keyof_strings_only: Some(true),
    //     list_emitted_files: Some(true),
    //     pretty: Some(true),
    //     list_files: Some(true),
    //     no_emit_helpers: Some(true),
    //     no_emit_on_error: Some(true),
    //     no_error_truncation: Some(true),
    //     no_implicit_use_strict: Some(true),
    //     no_lib: Some(true),
    //     no_resolve: Some(true),
    //     no_strict_generic_checks: Some(true),
    //     preserve_const_enums: Some(true),
    //     resolve_json_module: Some(true),
    //     skip_default_lib_check: Some(true),
    //     skip_lib_check: Some(true),
    //     strip_internal: Some(true),
    //     suppress_excess_property_errors: Some(true),
    //     suppress_implicit_any_index_errors: Some(true),
    //     trace_resolution: Some(true),
    //     use_define_for_class_fields: Some(true),
    //     preserve_watch_output: Some(true),
    //     no_property_access_from_index_signature: Some(true),
    //     map_root: Some("abc".to_string()),
    //     source_root: Some("abc".to_string()),
    //     declaration_dir: Some("abc".to_string()),
    //     imports_not_used_as_values: Some("abc".to_string()),
    //     jsx_factory: Some("abc".to_string()),
    //     jsx_fragment_factory: Some("abc".to_string()),
    //     jsx_import_source: Some("abc".to_string()),
    //     react_namespace: Some("abc".to_string()),
    //     out_file: Some("abc".to_string()),
    //     root_dir: Some("abc".to_string()),
    //     ts_build_info_file: Some("abc".to_string()),
    //     base_url: Some("abc".to_string()),
    //     fallback_polling: Some("abc".to_string()),
    //     watch_directory: Some("abc".to_string()),
    //     watch_file: Some("abc".to_string()),
    //     target: Some(Target::EsNext),
    //     module: Some(Module::EsNext),
    //     jsx: Some(Jsx::React),
    //     module_detection: Some(ModuleDetectionMode::Force),
    //     module_resolution: Some(ModuleResolutionMode::NodeNext),
    //     root_dirs: Some(vec!["abc".to_string(), "abc".to_string()]),
    //     types: Some(vec!["abc".to_string(), "abc".to_string()]),
    //     type_roots: Some(vec!["abc".to_string(), "abc".to_string()]),
    //     lib: Some(vec![Lib::Dom, Lib::EsNext]),
    //     paths: Some(
    //       btreemap! { "@".to_string() => vec!["src/".to_string()], "@components".to_string() => vec!["src/components".to_string()] },
    //     ),
    //     ..Default::default()
    //   }),
    //   extends: Some(root_tsconfig_path.to_string_lossy().to_string()),
    //   files: Some(vec!["*.ts".to_string(), "*.js".to_string()]),
    //   include: Some(vec!["*.ts".to_string(), "*.js".to_string()]),
    //   exclude: Some(vec!["*.ts".to_string(), "*.js".to_string()]),
    //   references: Some(vec![
    //     TsConfigReference {
    //       path: "abc.json".to_string(),
    //     },
    //     TsConfigReference {
    //       path: "abc.json".to_string(),
    //     },
    //   ]),
    //   type_acquisition: Some(TypeAcquisition::Object {
    //     enable: true,
    //     include: Some(vec!["*.ts".to_string(), "*.js".to_string()]),
    //     exclude: Some(vec!["*.ts".to_string(), "*.js".to_string()]),
    //     disable_filename_based_type_acquisition: Some(true),
    //   }),
    // };
  }
}
