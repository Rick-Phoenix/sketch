use std::fmt::{self, Display};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema, PartialOrd, Ord)]
#[serde(untagged)]
pub enum StringOrNum {
  Num(i64),
  String(String),
}

impl Display for StringOrNum {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Num(n) => write!(f, "{}", n),
      Self::String(s) => write!(f, "{}", s),
    }
  }
}
