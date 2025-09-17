use std::fs::{create_dir_all, File};

use askama::Template;
use maplit::btreeset;
use merge::Merge;

use crate::{
  package_json::{PackageJsonKind, Person},
  paths::get_cwd,
  ts_config::{tsconfig_defaults::get_default_root_tsconfig, TsConfig, TsConfigKind},
  Config, GenError, OxlintConfig, PackageManager, Preset,
};

impl Config {
  pub async fn create_ts_monorepo(self) -> Result<(), GenError> {
    let typescript = self.typescript.clone().unwrap_or_default();

    let package_json_presets = &typescript.package_json_presets;

    let root_dir = self.root_dir.clone().unwrap_or_else(|| get_cwd());

    let package_manager = typescript.package_manager.unwrap_or_default();
    let version_ranges = typescript.version_range.unwrap_or_default();
    let root_package = typescript.root_package.unwrap_or_default();

    create_dir_all(&root_dir).map_err(|e| GenError::DirCreation {
      path: root_dir.to_owned(),
      source: e,
    })?;

    macro_rules! write_to_output {
      ($($tokens:tt)*) => {
        write_file!(root_dir, !self.no_overwrite, $($tokens)*)
      };
    }

    let mut package_json_data = match root_package.package_json.unwrap_or_default() {
      PackageJsonKind::Id(id) => package_json_presets
        .get(&id)
        .ok_or(GenError::PresetNotFound {
          kind: Preset::PackageJson,
          name: id,
        })?
        .clone(),
      PackageJsonKind::Config(package_json) => *package_json,
    };

    for preset in package_json_data.extends.clone() {
      let target = package_json_presets
        .get(&preset)
        .ok_or(GenError::PresetNotFound {
          kind: Preset::PackageJson,
          name: preset,
        })?
        .clone();

      package_json_data.merge(target);
    }

    if package_json_data.package_manager.is_none() {
      package_json_data.package_manager = Some(package_manager.to_string());
    }

    get_contributors!(package_json_data, typescript, contributors);
    get_contributors!(package_json_data, typescript, maintainers);

    if !typescript.no_default_deps {
      for dep in ["typescript", "oxlint"] {
        if !package_json_data.dev_dependencies.contains_key(dep) {
          let version = if typescript.catalog {
            "catalog:".to_string()
          } else {
            "latest".to_string()
          };

          let range = version_ranges.create(version);
          package_json_data
            .dev_dependencies
            .insert(dep.to_string(), range);
        }
      }
    }

    if !typescript.no_convert_latest_to_range {
      package_json_data
        .get_latest_version_range(version_ranges)
        .await?;
    }

    package_json_data.name = root_package
      .name
      .clone()
      .unwrap_or_else(|| "root".to_string());

    write_to_output!(package_json_data, "package.json");

    let mut tsconfig_files: Vec<(String, TsConfig)> = Default::default();
    let tsconfig_presets = &typescript.tsconfig_presets;

    if let Some(root_tsconfigs) = root_package.ts_config {
      for directive in root_tsconfigs {
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
          TsConfigKind::Config(ts_config) => ("__root".to_string(), *ts_config),
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
      let root_tsconfig_name = "tsconfig.options.json".to_string();

      let root_tsconfig = TsConfig {
        extends: Some(root_tsconfig_name.clone()),
        files: Some(btreeset![]),
        references: Some(btreeset![]),
        ..Default::default()
      };

      tsconfig_files.push(("tsconfig.json".to_string(), root_tsconfig));

      let tsconfig_options = get_default_root_tsconfig();

      tsconfig_files.push((root_tsconfig_name, tsconfig_options));
    }

    for (file, tsconfig) in tsconfig_files {
      write_to_output!(tsconfig, file);
    }

    if matches!(package_manager, PackageManager::Pnpm) {
      let mut pnpm_data = typescript.pnpm_config.unwrap_or_default();

      for dir in &pnpm_data.packages {
        let dir = dir.strip_suffix("/*").unwrap_or(dir);
        create_dir_all(root_dir.join(dir)).map_err(|e| GenError::DirCreation {
          path: root_dir.to_path_buf(),
          source: e,
        })?;
      }

      pnpm_data
        .add_dependencies_to_catalog(version_ranges, &package_json_data)
        .await;

      write_to_output!(pnpm_data, "pnpm-workspace.yaml");
    }

    if let Some(oxlint_config) = root_package.oxlint && !matches!(oxlint_config, OxlintConfig::Bool(false)) {
      write_to_output!(oxlint_config, ".oxlintrc.json");
    }

    if let Some(templates) = root_package.generate_templates && !templates.is_empty() {
      self.generate_templates(&root_dir, templates)?;
    }

    Ok(())
  }
}
