#![allow(clippy::result_large_err)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../../docs/src/cli.md")]
//! # 🚩 Feature flags
#![doc = document_features::document_features!()]

#[macro_use]
mod macros;

use std::{
  collections::BTreeMap,
  fs::{create_dir_all, File},
};

use askama::Template;
use indexmap::IndexMap;
use merge::Merge;
use serde_json::Value;

use crate::{
  package_json::{PackageJson, Person},
  paths::get_abs_path,
};

pub mod config_elements;
pub use config::*;
pub use config_elements::*;
pub use errors::*;
pub mod commands;
pub mod config_setup;
pub(crate) mod init_repo;
pub(crate) mod serde_strategies;
mod ts_monorepo;

pub(crate) mod templating;

pub(crate) use merging_strategies::*;
pub(crate) use templating::*;

pub mod cli;
pub mod config;
pub mod errors;
pub(crate) mod merging_strategies;
pub mod moon;
pub mod package;
pub mod package_json;
pub(crate) mod paths;
pub mod pnpm;
pub mod ts_config;
pub mod versions;

pub(crate) type StringBTreeMap = BTreeMap<String, String>;
pub(crate) type JsonValueBTreeMap = BTreeMap<String, Value>;

pub(crate) type OrderedMap = IndexMap<String, Value>;

/// The kinds of presets that can be stored in the global config, along with a name key.
#[derive(Debug, Clone, Copy)]
pub enum Preset {
  Vitest,
  PackageJson,
  Package,
  TsConfig,
  Templating,
}
