#![allow(clippy::result_large_err)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]

#[macro_use]
mod macros;

pub(crate) mod exec;
pub(crate) mod fs;
pub(crate) mod init_repo;
pub(crate) mod merging_strategies;
pub(crate) mod serde_utils;
pub(crate) mod templating;

pub mod cli;
pub mod config;
pub mod docker;
pub mod errors;
pub mod git_workflow;
pub mod licenses;
pub mod rust;
pub mod ts;
pub mod versions;

use std::{collections::BTreeMap, fmt::Debug};

#[doc(inline)]
pub use config::*;
#[doc(inline)]
pub use errors::*;
pub(crate) use merging_strategies::*;
use serde_json::Value;
pub(crate) use templating::*;

use crate::{fs::get_abs_path, ts::package_json::PackageJson};

pub(crate) type StringBTreeMap = BTreeMap<String, String>;
pub(crate) type JsonValueBTreeMap = BTreeMap<String, Value>;

/// The kinds of presets supported by `sketch`.
#[derive(Debug, Clone, Copy)]
pub enum Preset {
  PackageJson,
  TsPackage,
  TsConfig,
  Templates,
  Oxlint,
  PreCommit,
  Repo,
  Gitignore,
  PnpmWorkspace,
  Vitest,
  DockerCompose,
  DockerService,
  CargoToml,
  GithubWorkflow,
  GithubWorkflowJob,
}
