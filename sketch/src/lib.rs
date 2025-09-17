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

pub use config::*;
pub use errors::*;
use indexmap::IndexMap;
use serde_json::Value;

use crate::{paths::get_abs_path, ts::package_json::PackageJson};
pub mod commands;
pub(crate) mod init_repo;
pub(crate) mod serde_strategies;
pub mod ts;

pub(crate) mod templating;

pub(crate) use merging_strategies::*;
pub(crate) use templating::*;

pub mod cli;
pub mod config;
pub mod errors;
pub(crate) mod merging_strategies;

pub(crate) mod paths;
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
