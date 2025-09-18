#![allow(clippy::result_large_err)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]

#[macro_use]
mod macros;

pub(crate) mod commands;
pub(crate) mod init_repo;
pub(crate) mod merging_strategies;
pub(crate) mod paths;
pub(crate) mod serde_strategies;
pub(crate) mod templating;

pub mod cli;
pub mod config;
pub mod errors;
pub mod ts;
pub mod versions;

use std::{
  collections::BTreeMap,
  fs::{create_dir_all, File},
};

#[doc(inline)]
pub use config::*;
#[doc(inline)]
pub use errors::*;
use indexmap::IndexMap;
pub(crate) use merging_strategies::*;
use serde_json::Value;
pub(crate) use templating::*;

use crate::{paths::get_abs_path, ts::package_json::PackageJson};

pub(crate) type StringBTreeMap = BTreeMap<String, String>;
pub(crate) type JsonValueBTreeMap = BTreeMap<String, Value>;

pub(crate) type OrderedMap = IndexMap<String, Value>;

/// The kinds of presets that can be stored in the global config, along with a name key.
#[derive(Debug, Clone, Copy)]
pub enum Preset {
  PackageJson,
  TsPackage,
  TsConfig,
  Templates,
}
