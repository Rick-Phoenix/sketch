use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub(crate) struct Cli {
  /// Sets a custom config file
  #[arg(short, long, value_name = "FILE")]
  pub(crate) config: Option<PathBuf>,

  #[command(subcommand)]
  pub(crate) command: Commands,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
  /// Generates a new config file
  Init,
  /// Generates a new monorepo
  Repo {
    /// the name for the new repo
    #[arg(short, long)]
    name: String,
  },

  /// Generates a new package
  Package {
    /// the name for the new package
    #[arg(short, long)]
    name: String,
  },

  /// Generates a file from a template
  Render {
    /// the name of the template to render
    #[arg(short, long)]
    name: String,
  },
}
