#![allow(
	clippy::result_large_err,
	clippy::large_enum_variant,
	clippy::field_reassign_with_default
)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]

use schemars::JsonSchema_repr;

#[macro_use]
mod macros;

pub(crate) mod exec;
pub(crate) mod fs;
pub(crate) mod init_repo;
pub(crate) mod merging_strategies;
pub(crate) mod serde_utils;
pub(crate) mod templating;

use serde_utils::is_false;

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
use std::{fmt::Display, str::FromStr};

use clap::ValueEnum;

#[doc(inline)]
pub use config::*;
#[doc(inline)]
pub use errors::*;
pub(crate) use merging_strategies::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
pub(crate) use templating::*;

use merge::Merge;
use serde_repr::{Deserialize_repr, Serialize_repr};

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
	GithubWorkflowStep,
	RustCrate,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum Or<A, B> {
	A(A),
	B(B),
}

type StringOrNum = Or<String, i64>;
