use clap::ValueEnum;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Generates a new license file
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema, ValueEnum)]
pub enum License {
  /// Apache 2.0 license.
  ///
  /// See more: https://choosealicense.com/licenses/apache-2.0/
  #[serde(rename = "Apache-2.0")]
  Apache2,
  /// GNU GPL 3.0 license.
  ///
  /// See more: https://choosealicense.com/licenses/gpl-3.0/
  #[serde(rename = "GPL-3.0")]
  Gpl3,
  /// GNU AGPL 3.0 license.
  ///
  /// See more: https://choosealicense.com/licenses/agpl-3.0/
  #[serde(rename = "AGPL-3.0")]
  Agpl3,
  /// MIT license.
  ///
  /// See more: https://choosealicense.com/licenses/mit/
  MIT,
}

impl License {
  pub fn get_content(&self) -> &str {
    match self {
      License::Apache2 => APACHE_2_LICENSE,
      License::Gpl3 => GPL_3_LICENSE,
      License::Agpl3 => AGPL_3_LICENSE,
      License::MIT => MIT_LICENSE,
    }
  }
}

const APACHE_2_LICENSE: &str = include_str!("./templates/apache-2.0");
const MIT_LICENSE: &str = include_str!("./templates/mit");
const GPL_3_LICENSE: &str = include_str!("./templates/gpl-3.0");
const AGPL_3_LICENSE: &str = include_str!("./templates/agpl-3.0");
