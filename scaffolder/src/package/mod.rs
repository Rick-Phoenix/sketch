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
use serde::{Deserialize, Serialize};

use crate::{
  moon::MoonDotYml,
  package::vitest::{TestsSetupFile, VitestConfig, VitestConfigStruct},
  paths::get_relative_path,
  pnpm::PnpmWorkspace,
  versions::get_latest_version,
  DevTsConfig, PackageJsonData, SrcTsConfig, TsConfigData, *,
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

  let mut package_json_templates = global_config.package_json_presets;

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
    PackageJsonData::Named(name) => package_json_templates
      .remove(&name)
      .expect("Package json not found in store"),
    PackageJsonData::Definition(package_json) => package_json,
  };

  if matches!(package_json_data.repository, Repository::Workspace { workspace: true }) && let Some(repo) = global_config.repository {
    package_json_data.repository = repo;
  }

  macro_rules! add_package_json_val_to_data {
    ($($tokens:tt)*) => {
      add_package_json_val!(global_config, package_json_data, $($tokens)*)
    };
  }

  add_package_json_val_to_data!(optional_in_config, optional_in_package_json, description);
  add_package_json_val_to_data!(keywords);
  add_package_json_val_to_data!(homepage);
  add_package_json_val_to_data!(optional_in_config, optional_in_package_json, bugs);
  add_package_json_val_to_data!(license);
  add_package_json_val_to_data!(author);
  add_package_json_val_to_data!(files);
  add_package_json_val_to_data!(exports);
  add_package_json_val_to_data!(engines);
  add_package_json_val_to_data!(os);
  add_package_json_val_to_data!(cpu);
  add_package_json_val_to_data!(main);
  add_package_json_val_to_data!(optional_in_config, optional_in_package_json, browser);

  get_contributors!(package_json_data, global_config, contributors);
  get_contributors!(package_json_data, global_config, maintainers);

  if let PackageManagerJson::Workspace { workspace: true } = package_json_data.package_manager {
    package_json_data.package_manager =
      PackageManagerJson::Data(global_config.package_manager.to_string());
  }

  for dep in DEFAULT_DEPS {
    let version = if global_config.catalog {
      "catalog:".to_string()
    } else {
      let version = get_latest_version(dep)
        .await
        .unwrap_or_else(|_| "latest".to_string());
      global_config.version_ranges.create(version)
    };

    package_json_data
      .dev_dependencies
      .insert(dep.to_string(), version);
  }

  for preset in package_json_data.dependencies_presets.clone() {
    let preset_data = global_config
      .dependencies_presets
      .get(&preset)
      .unwrap_or_else(|| panic!("Dependencies preset {} not found.", preset))
      .clone();

    package_json_data.merge_dependencies_preset(preset_data);
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

  let is_app = matches!(config.kind, PackageKind::App);
  let rel_path_to_root_dir = get_relative_path(&output, &PathBuf::from(&root_dir))?;
  write_file!(
    TsConfig {
      root_tsconfig_path: rel_path_to_root_dir
        .join(global_config.root_tsconfig_name.clone())
        .to_string_lossy()
        .to_string(),
      references: {
        let mut refs = vec![global_config.project_tsconfig_name.clone()];
        if !is_app {
          refs.push(global_config.dev_tsconfig_name.clone());
        }
        refs
      },
    },
    "tsconfig.json"
  );

  let root_tsconfig_path = PathBuf::from(&root_dir).join("tsconfig.json");
  let root_tsconfig_file = File::open(&root_tsconfig_path)?;

  let root_tsconfig: TsConfigData = serde_json::from_reader(root_tsconfig_file)?;

  let path_to_new_tsconfig = output.join("tsconfig.json").to_string_lossy().to_string();

  if !root_tsconfig
    .references
    .iter()
    .any(|r| r.path == path_to_new_tsconfig)
  {
    let mut references: Vec<String> = root_tsconfig
      .references
      .into_iter()
      .map(|r| r.path)
      .collect();
    references.push(path_to_new_tsconfig);

    let new_root_tsconfig = TsConfig {
      references,
      root_tsconfig_path: global_config.root_tsconfig_name.clone(),
    };

    new_root_tsconfig.write_into(&mut File::create(root_tsconfig_path)?)?;
  }

  let out_dir = get_relative_path(&output, &root_dir.join(global_config.out_dir))?
    .join(PathBuf::from(config.name))
    .to_string_lossy()
    .to_string();

  write_file!(
    SrcTsConfig {
      kind: config.kind.into(),
      out_dir: out_dir.clone(),
    },
    global_config.project_tsconfig_name.clone()
  );

  if !is_app {
    write_file!(
      DevTsConfig {
        project_tsconfig_name: global_config.project_tsconfig_name,
        out_dir: out_dir.clone(),
      },
      global_config.dev_tsconfig_name
    );
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
}
