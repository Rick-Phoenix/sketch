pub mod oxlint;
pub mod package;
pub mod package_json;
pub mod pnpm;
pub mod ts_config;
mod ts_monorepo;
pub mod vitest;

use std::{
  fmt::Display,
  path::{Path, PathBuf},
};

use clap::{Parser, ValueEnum};
use indexmap::IndexMap;
use merge::Merge;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
  fs::{find_file_up, get_parent_dir},
  merge_index_maps, overwrite_if_some,
  ts::{
    oxlint::OxlintPreset,
    package::PackageConfig,
    package_json::{PackageJsonPreset, Person, PersonData},
    pnpm::PnpmWorkspace,
    ts_config::TsConfigPreset,
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
      ts_config_presets: Default::default(),
      oxlint_presets: Default::default(),
      people: Default::default(),
      pnpm: Default::default(),
    }
  }
}

#[derive(Clone, Debug, Deserialize, Serialize, Merge, Parser, PartialEq, JsonSchema)]
#[merge(strategy = overwrite_if_some)]
#[serde(default)]
pub struct TypescriptConfig {
  /// The package manager being used. [default: pnpm].
  #[arg(value_enum, long, value_name = "NAME")]
  pub package_manager: Option<PackageManager>,

  /// The settings to use in generated pnpm-workspace.yaml files, for new monorepos, if pnpm is selected as a package manager.
  #[arg(skip)]
  pub pnpm: Option<PnpmWorkspace>,

  /// Does not add default dependencies to new `package.json` files (typescript and oxlint, plus vitest if enabled)
  #[merge(strategy = merge::bool::overwrite_false)]
  #[arg(long)]
  pub no_default_deps: bool,

  /// The kind of version range to use for dependencies that are fetched automatically. [default: minor]
  #[arg(value_enum)]
  #[arg(long, value_name = "KIND")]
  pub version_range: Option<VersionRange>,

  /// Uses the pnpm catalog for default dependencies, and automatically adds dependencies marked with `catalog:` to the catalog, if they are missing.
  #[merge(strategy = merge::bool::overwrite_false)]
  #[arg(long)]
  pub catalog: bool,

  /// Does not convert dependencies marked as `latest` to a version range.
  #[merge(strategy = merge::bool::overwrite_false)]
  #[arg(long = "no-convert-latest")]
  pub no_convert_latest_to_range: bool,

  /// A map of individual [`PersonData`] that can be referenced as authors, contributors or maintainers in a [`PackageJsonPreset`].
  #[arg(skip)]
  #[merge(strategy = merge_index_maps)]
  pub people: IndexMap<String, PersonData>,

  /// A map containing [`PackageJsonPreset`]s.
  #[merge(strategy = merge_index_maps)]
  #[arg(skip)]
  pub package_json_presets: IndexMap<String, PackageJsonPreset>,

  /// A map containing [`TsConfigPreset`]s.
  #[merge(strategy = merge_index_maps)]
  #[arg(skip)]
  pub ts_config_presets: IndexMap<String, TsConfigPreset>,

  /// A map containing [`OxlintPreset`]s.
  #[merge(strategy = merge_index_maps)]
  #[arg(skip)]
  pub oxlint_presets: IndexMap<String, OxlintPreset>,

  /// A map of [`PackageConfig`] presets.
  #[merge(strategy = merge_index_maps)]
  #[arg(skip)]
  pub package_presets: IndexMap<String, PackageConfig>,
}

pub(crate) fn find_monorepo_root(
  start_dir: &Path,
  package_manager: PackageManager,
) -> Option<PathBuf> {
  let root_marker = package_manager.root_marker();

  find_file_up(start_dir, root_marker).map(|file| get_parent_dir(&file).to_path_buf())
}

impl PackageManager {
  pub fn root_marker(&self) -> &str {
    match self {
      PackageManager::Pnpm => "pnpm-workspace.yaml",
      PackageManager::Npm => "package-lock.json",
      PackageManager::Deno => "deno.lock",
      PackageManager::Bun => "bun.lock",
      PackageManager::Yarn => "yarn.lock",
    }
  }
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
