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
  config::PackageManager, json_files::PackageJson, moon::MoonDotYml, paths::get_relative_path, *,
};

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PackageJsonData {
  Named(String),
  Definition(PackageJson),
}

impl Default for PackageJsonData {
  fn default() -> Self {
    Self::Definition(PackageJson::default())
  }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct PackageConfig {
  pub name: String,
  pub package_json: PackageJsonData,
  pub root_tsconfig_name: String,
  pub project_tsconfig_name: String,
  pub package_manager: PackageManager,
  pub moonrepo: Option<MoonDotYml>,
  pub dir: String,
}

impl Default for PackageConfig {
  fn default() -> Self {
    Self {
      name: "my-awesome-package".to_string(),
      package_json: Default::default(),
      package_manager: Default::default(),
      root_tsconfig_name: "tsconfig.options".to_string(),
      project_tsconfig_name: "tsconfig.dev".to_string(),
      moonrepo: None,
      dir: ".".to_string(),
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

pub fn build_package(_name: &str) -> Result<(), Box<dyn std::error::Error>> {
  let global_config: Config = Config::figment()
    .merge(Toml::file("scaffolder/config.toml"))
    .extract()?;

  let packages_dir = global_config.packages_dir;
  let mut package_json_templates = global_config.package_json;

  let root_dir = global_config.root_dir;

  let mut packages = global_config.package;
  let config = packages.remove("alt2").unwrap();

  let output = PathBuf::new().join(&root_dir).join(packages_dir);

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

  let package_json_data = match config.package_json {
    PackageJsonData::Named(name) => package_json_templates
      .remove(&name)
      .expect("Package json not found in store"),
    PackageJsonData::Definition(package_json) => package_json,
  };

  write_file!(package_json_data, "package.json");

  if let Some(moon_config) = config.moonrepo {
    write_file!(moon_config, "moon.yml");
  }

  let rel_path_to_root_dir = get_relative_path(&output, &PathBuf::from(&root_dir))?;
  write_file!(
    TsConfig {
      root_tsconfig_path: rel_path_to_root_dir
        .join(format!("{}.json", global_config.root_tsconfig_name))
        .to_string_lossy()
        .to_string(),
    },
    "tsconfig.json"
  );

  Ok(())
}

#[cfg(test)]
mod test {
  use crate::package::build_package;

  #[test]
  fn package_test() -> std::result::Result<(), std::boxed::Box<(dyn std::error::Error + 'static)>> {
    build_package("default")
  }
}
