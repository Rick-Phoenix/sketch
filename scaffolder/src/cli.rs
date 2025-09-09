#![allow(clippy::large_enum_variant)]

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::{
  package::{PackageConfig, PackageKind},
  Config, RootPackage,
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub(crate) struct Cli {
  /// Sets a custom config file
  #[arg(short, long, value_name = "FILE")]
  pub config: Option<PathBuf>,

  #[command(subcommand)]
  pub command: Commands,

  #[command(flatten)]
  pub overrides: Option<Config>,
}

#[derive(Args, Debug)]
#[group(multiple = false)]
pub(crate) struct PackageKindFlag {
  #[arg(long)]
  app: bool,

  #[arg(long)]
  library: bool,
}

impl From<PackageKindFlag> for PackageKind {
  fn from(value: PackageKindFlag) -> Self {
    if value.app {
      Self::App
    } else {
      Self::Library
    }
  }
}

#[derive(Args, Debug)]
pub(crate) struct RepoBooleanFlags {
  #[arg(long)]
  pub(crate) no_convert_latest: bool,
  #[arg(long)]
  pub(crate) no_oxlint: bool,
  #[arg(long)]
  pub(crate) no_catalog: bool,
  #[arg(long)]
  pub(crate) no_overwrite: bool,
  #[arg(long)]
  pub(crate) no_pre_commit: bool,
  #[arg(long)]
  pub(crate) moonrepo: bool,
  #[arg(long, conflicts_with = "no_shared_out_dir")]
  pub(crate) shared_out_dir: Option<String>,
  #[arg(long, default_value_t = false)]
  pub(crate) no_shared_out_dir: bool,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
  /// Generates a new config file
  Init,
  /// Generates a new monorepo
  Repo {
    #[command(flatten)]
    root_package: Option<RootPackage>,
    #[command(flatten)]
    boolean_flags: RepoBooleanFlags,
  },

  /// Generates a new package
  Package {
    name: String,
    #[arg(short, long, conflicts_with = "PackageConfig")]
    preset: Option<String>,
    #[command(flatten)]
    kind: Option<PackageKindFlag>,
    #[command(flatten)]
    config: Option<PackageConfig>,
    #[arg(long)]
    moonrepo: bool,
    #[arg(long)]
    no_vitest: bool,
    #[arg(long)]
    oxlint: bool,
    #[arg(long)]
    no_update_root_tsconfig: bool,
  },

  /// Generates a file from a template
  Render {
    #[arg(requires = "input")]
    output: String,
    #[arg(group = "input")]
    id: Option<String>,
    #[arg(long, group = "input")]
    content: Option<String>,
  },

  Command {
    command: String,
    #[arg(short, long, conflicts_with = "command")]
    file: Option<String>,
    #[arg(short, long, default_value_t = format!("sh"))]
    shell: String,
  },
}
