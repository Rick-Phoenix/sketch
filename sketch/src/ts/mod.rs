pub mod package;
pub mod package_json;
pub mod pnpm;
pub mod ts_config;
mod ts_monorepo;
pub mod vitest;

use std::fmt::Display;

use askama::Template;
use clap::{Parser, ValueEnum};
use indexmap::IndexMap;
use merge::Merge;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
  merge_index_maps, overwrite_if_some,
  ts::{
    package::{PackageConfig, RootPackage},
    package_json::{PackageJson, Person, PersonData},
    pnpm::PnpmWorkspace,
    ts_config::TsConfig,
  },
  versions::VersionRange,
};

impl TypescriptConfig {
  pub fn get_contributor(&self, name: &str) -> Option<Person> {
    self
      .people
      .get(name)
      .map(|person| Person::Data(person.clone()))
  }
}

impl Default for TypescriptConfig {
  fn default() -> Self {
    Self {
      no_default_deps: false,
      no_convert_latest_to_range: false,
      package_json_presets: Default::default(),
      package_manager: Default::default(),
      package_presets: Default::default(),
      catalog: false,
      version_range: Default::default(),
      tsconfig_presets: Default::default(),
      people: Default::default(),
      pnpm_config: Default::default(),
      root_package: Default::default(),
    }
  }
}

#[derive(Clone, Debug, Deserialize, Serialize, Merge, Parser, PartialEq, JsonSchema)]
#[serde(default)]
pub struct TypescriptConfig {
  /// The configuration for the root typescript package to generate in new monorepos.
  /// Can be empty to use defaults.
  #[merge(skip)]
  #[arg(skip)]
  pub root_package: Option<RootPackage>,

  /// The package manager being used. [default: pnpm].
  #[merge(strategy = overwrite_if_some)]
  #[arg(value_enum, long, value_name = "NAME")]
  pub package_manager: Option<PackageManager>,

  /// Does not add default dependencies to new `package.json` files (typescript and oxlint, plus vitest if enabled)
  #[merge(strategy = merge::bool::overwrite_false)]
  #[arg(long)]
  pub no_default_deps: bool,

  /// The kind of version ranges to use for dependencies that are fetched automatically. [default: minor]
  #[merge(strategy = overwrite_if_some)]
  #[arg(value_enum)]
  #[arg(long, value_name = "KIND")]
  pub version_range: Option<VersionRange>,

  /// Uses the pnpm catalog for default dependencies, and automatically adds dependencies marked with `catalog:` to `pnpm-workspace.yaml`.
  #[merge(strategy = merge::bool::overwrite_false)]
  #[arg(long)]
  pub catalog: bool,

  /// Does not convert dependencies marked as `latest` to a version range.
  #[merge(strategy = merge::bool::overwrite_false)]
  #[arg(long = "no-convert-latest")]
  pub no_convert_latest_to_range: bool,

  /// A map of individual [`PersonData`] that can be referenced in [`PackageJson::contributors`] or [`PackageJson::maintainers`].
  #[arg(skip)]
  #[merge(strategy = merge_index_maps)]
  pub people: IndexMap<String, PersonData>,

  /// A map containing [`PackageJson`] presets.
  #[merge(strategy = merge_index_maps)]
  #[arg(skip)]
  pub package_json_presets: IndexMap<String, PackageJson>,

  /// A map containing [`TsConfig`] presets.
  #[merge(strategy = merge_index_maps)]
  #[arg(skip)]
  pub tsconfig_presets: IndexMap<String, TsConfig>,

  /// A map of [`PackageConfig`] presets.
  #[merge(strategy = merge_index_maps)]
  #[arg(skip)]
  pub package_presets: IndexMap<String, PackageConfig>,

  /// The settings to use in the generated pnpm-workspace.yaml file, if pnpm is selected as a package manager.
  #[merge(skip)]
  #[arg(skip)]
  pub pnpm_config: Option<PnpmWorkspace>,
}

#[derive(
  Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default, ValueEnum, Copy, JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum PackageManager {
  #[default]
  Pnpm,
  Npm,
  Deno,
  Bun,
  Yarn,
}

impl Display for PackageManager {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      PackageManager::Pnpm => {
        write!(f, "pnpm")
      }
      PackageManager::Npm => {
        write!(f, "npm")
      }
      PackageManager::Deno => {
        write!(f, "deno")
      }
      PackageManager::Bun => {
        write!(f, "bun")
      }
      PackageManager::Yarn => {
        write!(f, "yarn")
      }
    }
  }
}

#[derive(Debug, Template, Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
#[template(path = "oxlint.json.j2")]
#[serde(untagged)]
pub enum OxlintConfig {
  Bool(bool),
  Text(String),
}

impl Default for OxlintConfig {
  fn default() -> Self {
    Self::Bool(true)
  }
}
