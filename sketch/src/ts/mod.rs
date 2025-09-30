pub mod oxlint;
pub mod package;
pub mod package_json;
pub mod pnpm;
pub mod ts_config;
pub mod ts_monorepo;
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
    pnpm::PnpmPreset,
    ts_config::TsConfigPreset,
    vitest::VitestConfig,
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

/// All settings related to typescript projects.
#[derive(Clone, Debug, Deserialize, Serialize, Merge, Parser, PartialEq, JsonSchema, Default)]
#[merge(strategy = overwrite_if_some)]
#[serde(default)]
pub struct TypescriptConfig {
  /// The package manager being used. [default: pnpm].
  #[arg(value_enum, long, value_name = "NAME")]
  pub package_manager: Option<PackageManager>,

  /// Do not add default dependencies to new `package.json` files (typescript and oxlint, plus vitest if enabled)
  #[arg(long)]
  pub no_default_deps: Option<bool>,

  /// The kind of version range to use for dependencies that are fetched automatically. [default: minor]
  #[arg(value_enum)]
  #[arg(long, value_name = "KIND")]
  pub version_range: Option<VersionRange>,

  /// Uses the pnpm catalog for default dependencies, and automatically adds dependencies marked with `catalog:` to the catalog, if they are missing.
  #[arg(long)]
  pub catalog: Option<bool>,

  /// Do not convert dependencies marked as `latest` to a version range.
  #[arg(long = "no-convert-latest")]
  pub no_convert_latest_to_range: Option<bool>,

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

  /// A map of presets for `pnpm-workspace.yaml` configurations.
  #[merge(strategy = merge_index_maps)]
  #[arg(skip)]
  pub pnpm_presets: IndexMap<String, PnpmPreset>,

  /// A map of presets for vitest setups.
  #[merge(strategy = merge_index_maps)]
  #[arg(skip)]
  pub vitest_presets: IndexMap<String, VitestConfig>,
}

impl PackageManager {
  pub fn find_root(&self, start_dir: &Path) -> Option<PathBuf> {
    let root_marker = self.root_marker();

    find_file_up(start_dir, root_marker).map(|file| get_parent_dir(&file).to_path_buf())
  }

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

/// A js/ts package manager.
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
