use std::{fmt::Display, str::FromStr};

use clap::{Args, ValueEnum};

use crate::{package::PackageKind, GenError};

#[derive(Clone, Debug, ValueEnum, Default)]
pub enum ConfigFormat {
  #[default]
  Yaml,
  Toml,
  Json,
}

impl FromStr for ConfigFormat {
  type Err = GenError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "yaml" => Ok(Self::Yaml),
      "toml" => Ok(Self::Toml),
      "json" => Ok(Self::Json),
      _ => Err(GenError::Custom(format!(
        "Invalid configuration format '{}'. Allowed formats are: yaml, toml, json",
        s
      ))),
    }
  }
}

impl Display for ConfigFormat {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ConfigFormat::Yaml => write!(f, "yaml"),
      ConfigFormat::Toml => write!(f, "toml"),
      ConfigFormat::Json => write!(f, "json"),
    }
  }
}

#[derive(Args, Debug)]
#[group(multiple = false)]
pub struct PackageKindFlag {
  /// Marks the package as an application.
  #[arg(long)]
  app: bool,

  /// Marks the package as a library.
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
