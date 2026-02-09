use std::{fmt::Display, str::FromStr};

use clap::ValueEnum;

use crate::GenError;

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
				"Invalid configuration format '{s}'. Allowed formats are: yaml, toml, json"
			))),
		}
	}
}

impl Display for ConfigFormat {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Yaml => write!(f, "yaml"),
			Self::Toml => write!(f, "toml"),
			Self::Json => write!(f, "json"),
		}
	}
}
