use std::{
  collections::{BTreeMap, BTreeSet},
  fmt::{self, Display},
};

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

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema, PartialOrd, Ord)]
#[serde(untagged)]
pub enum StringOrList {
  String(String),
  List(Vec<String>),
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema, PartialOrd, Ord)]
#[serde(untagged)]
pub enum ListOrMap {
  List(BTreeSet<String>),
  Map(BTreeMap<String, String>),
}

impl ListOrMap {
  pub fn contains(&self, key: &str) -> bool {
    match self {
      ListOrMap::List(list) => list.contains(key),
      ListOrMap::Map(map) => map.contains_key(key),
    }
  }

  pub fn get(&self, key: &str) -> Option<&String> {
    match self {
      ListOrMap::List(list) => list.get(key),
      ListOrMap::Map(map) => map.get(key),
    }
  }
}

pub(crate) fn merge_list_or_map(left: &mut Option<ListOrMap>, right: Option<ListOrMap>) {
  if let Some(right) = right {
    if let Some(left_data) = left {
      if let ListOrMap::List(left_list) = left_data && let ListOrMap::List(right_list) = right {
        left_list.extend(right_list);
      } else if let ListOrMap::Map(left_list) = left_data && let ListOrMap::Map(right_list) = right {
        left_list.extend(right_list);
      } else {
        *left = Some(right);
      }
    } else {
      *left = Some(right);
    }
  }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema, PartialOrd, Ord)]
#[serde(untagged)]
pub enum StringOrSortedList {
  String(String),
  List(BTreeSet<String>),
}

pub(crate) fn merge_string_or_sorted_list(
  left: &mut Option<StringOrSortedList>,
  right: Option<StringOrSortedList>,
) {
  if let Some(right) = right {
    if let Some(left_data) = left {
      if let StringOrSortedList::List(left_list) = left_data && let StringOrSortedList::List(right_list) = right  {
        left_list.extend(right_list);
      } else {
        *left = Some(right);
      }
    } else {
      *left = Some(right);
    }
  }
}

impl Default for StringOrSortedList {
  fn default() -> Self {
    Self::String(String::new())
  }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd, JsonSchema)]
#[serde(untagged)]
pub enum SingleValue {
  String(String),
  Bool(bool),
  Int(i64),
  Float(f64),
}

impl fmt::Display for SingleValue {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Self::String(s) => f.write_str(s),
      Self::Bool(b) => write!(f, "{b}"),
      Self::Int(i) => write!(f, "{i}"),
      Self::Float(fl) => write!(f, "{fl}"),
    }
  }
}
