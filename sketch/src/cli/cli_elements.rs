use std::{fmt::Display, str::FromStr};

use clap::ValueEnum;

use crate::{custom_templating::TemplateOutput, GenError};

#[derive(Debug, Clone)]
pub(crate) enum TemplateRef {
  PresetId(String),
  Template(TemplateOutput),
}

impl FromStr for TemplateRef {
  type Err = &'static str;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let s = s.trim();

    if let Ok(output) = TemplateOutput::from_cli(s) {
      Ok(Self::Template(output))
    } else {
      Ok(Self::PresetId(s.to_string()))
    }
  }
}

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
