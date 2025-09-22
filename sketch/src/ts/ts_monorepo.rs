use std::fs::create_dir_all;

use maplit::btreeset;

use crate::{
  fs::{create_all_dirs, get_cwd, serialize_json, serialize_yaml},
  ts::{
    oxlint::OxlintConfigSetting,
    package_json::PackageJsonData,
    ts_config::{tsconfig_defaults::get_default_root_tsconfig, TsConfig, TsConfigKind},
    PackageManager,
  },
  Config, GenError, Preset,
};

impl Config {
  pub async fn create_ts_monorepo(self) -> Result<(), GenError> {
    let typescript = self.typescript.clone().unwrap_or_default();

    let package_json_presets = &typescript.package_json_presets;

    let out_dir = self.out_dir.clone().unwrap_or_else(|| get_cwd());

    let package_manager = typescript.package_manager.unwrap_or_default();
    let version_ranges = typescript.version_range.unwrap_or_default();
    let root_package = typescript.root_package.unwrap_or_default();

    create_all_dirs(&out_dir)?;

    let (package_json_id, package_json_preset) = match root_package.package_json.unwrap_or_default()
    {
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
        .convert_latest_to_range(version_ranges)
        .await?;
    }

    let root_package_name = root_package.name.unwrap_or_else(|| "root".to_string());

    package_json_data.name = Some(root_package_name.clone());

    serialize_json(&package_json_data, &out_dir.join("package.json"))?;

    let mut tsconfig_files: Vec<(String, TsConfig)> = Default::default();
    let tsconfig_presets = &typescript.ts_config_presets;

    if let Some(root_tsconfigs) = root_package.ts_config {
      for directive in root_tsconfigs {
        let (id, tsconfig_data) = match directive.config.unwrap_or_default() {
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
          TsConfigKind::Config(data) => {
            (format!("__inlined_definition_{}", root_package_name), data)
          }
        };

        let tsconfig_data = tsconfig_data.process_data(id.as_str(), tsconfig_presets)?;

        tsconfig_files.push((
          directive
            .output
            .unwrap_or_else(|| "tsconfig.json".to_string()),
          tsconfig_data,
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
      serialize_json(&tsconfig, &out_dir.join(file))?;
    }

    if matches!(package_manager, PackageManager::Pnpm) {
      let mut pnpm_data = typescript.pnpm.unwrap_or_default();

      for dir in &pnpm_data.packages {
        let dir = dir.strip_suffix("/*").unwrap_or(dir);
        create_dir_all(out_dir.join(dir)).map_err(|e| GenError::DirCreation {
          path: out_dir.to_path_buf(),
          source: e,
        })?;
      }

      pnpm_data
        .add_dependencies_to_catalog(version_ranges, &package_json_data)
        .await;

      serialize_yaml(&pnpm_data, &out_dir.join("pnpm-workspace.yaml"))?;
    }

    if let Some(oxlint_config) = root_package.oxlint && !matches!(oxlint_config, OxlintConfigSetting::Bool(false)) {
      serialize_json(&oxlint_config, &out_dir.join(".oxlintrc.json"))?;
    }

    if let Some(templates) = root_package.with_templates && !templates.is_empty() {
      self.generate_templates(&out_dir, templates)?;
    }

    Ok(())
  }
}
