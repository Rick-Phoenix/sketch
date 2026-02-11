#![allow(
	clippy::result_large_err,
	clippy::large_enum_variant,
	clippy::field_reassign_with_default
)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]

use indexmap::{IndexMap, IndexSet};
use merge_it::{Merge, merge_option, overwrite_always, overwrite_if_none};
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tera::Tera;

use crate::custom_templating::*;
use crate::fs::*;
use clap::{Args, Parser, ValueEnum};
use cli::parsers::*;
use licenses::License;
use maplit::btreeset;
use serde_utils::*;
use std::{
	collections::{BTreeSet, HashMap},
	convert::Infallible,
	env,
	env::current_dir,
	ffi::OsStr,
	fmt::{Debug, Display},
	fs::{File, create_dir_all, exists, read_to_string, remove_dir_all},
	io::Write,
	mem,
	path::{Component, Path, PathBuf},
	process::{Command, Stdio},
	str::FromStr,
	sync::Arc,
	sync::LazyLock,
};

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
pub mod gh_workflow;
pub mod licenses;
pub mod rust;
pub mod ts;
pub mod versions;

#[doc(inline)]
pub use config::*;
#[doc(inline)]
pub use errors::*;
pub(crate) use merging_strategies::*;
pub(crate) use templating::*;

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

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
pub enum Or<A, B> {
	A(A),
	B(B),
}
