use std::{collections::BTreeMap, fmt::Display};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::JsonValueBTreeMap;

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq, JsonSchema)]
pub struct TsPlugin {
  pub name: String,
  #[serde(flatten, skip_serializing_if = "BTreeMap::is_empty")]
  pub extras: JsonValueBTreeMap,
}

impl PartialOrd for TsPlugin {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for TsPlugin {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.name.cmp(&other.name)
  }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, JsonSchema)]
pub struct TsConfigReference {
  pub path: String,
}

#[derive(Deserialize, Debug, Clone, Serialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum NewLine {
  Lf,
  Crlf,
}

impl Display for NewLine {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Lf => write!(f, "lf"),
      Self::Crlf => write!(f, "crlf"),
    }
  }
}

#[derive(Deserialize, Debug, Clone, Serialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum WatchFile {
  #[serde(alias = "fixedpollinginterval")]
  FixedPollingInterval,
  #[serde(alias = "prioritypollinginterval")]
  PriorityPollingInterval,
  #[serde(alias = "dynamicprioritypolling")]
  DynamicPriorityPolling,
  #[serde(alias = "usefsevents")]
  UseFsEvents,
  #[serde(alias = "usefseventsonparentdirectory")]
  UseFsEventsOnParentDirectory,
}

impl Display for WatchFile {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      WatchFile::FixedPollingInterval => write!(f, "fixedPollingInterval"),
      WatchFile::PriorityPollingInterval => write!(f, "priorityPollingInterval"),
      WatchFile::DynamicPriorityPolling => write!(f, "dynamicPriorityPolling"),
      WatchFile::UseFsEvents => write!(f, "useFsEvents"),
      WatchFile::UseFsEventsOnParentDirectory => write!(f, "useFsEventsOnParentDirectory"),
    }
  }
}

#[derive(Deserialize, Debug, Clone, Serialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum WatchDirectory {
  #[serde(alias = "fixedpollinginterval")]
  FixedPollingInterval,
  #[serde(alias = "dynamicprioritypolling")]
  DynamicPriorityPolling,
  #[serde(alias = "usefsevents")]
  UseFsEvents,
}

impl Display for WatchDirectory {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      WatchDirectory::FixedPollingInterval => write!(f, "fixedPollingInterval"),
      WatchDirectory::DynamicPriorityPolling => write!(f, "dynamicPriorityPolling"),
      WatchDirectory::UseFsEvents => write!(f, "useFsEvents"),
    }
  }
}

#[derive(Deserialize, Debug, Clone, Serialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum FallbackPolling {
  #[serde(alias = "fixedpollinginterval")]
  FixedPollingInterval,
  #[serde(alias = "prioritypollinginterval")]
  PriorityPollingInterval,
  #[serde(alias = "dynamicprioritypolling")]
  DynamicPriorityPolling,
}

impl Display for FallbackPolling {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      FallbackPolling::FixedPollingInterval => write!(f, "fixedPollingInterval"),
      FallbackPolling::PriorityPollingInterval => write!(f, "priorityPollingInterval"),
      FallbackPolling::DynamicPriorityPolling => write!(f, "dynamicPriorityPolling"),
    }
  }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Copy, Clone, Eq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Jsx {
  React,
  ReactJsx,
  ReactJsxdev,
  ReactNative,
  Preserve,
}

impl Display for Jsx {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Jsx::React => write!(f, "react"),
      Jsx::ReactJsx => write!(f, "react-jsx"),
      Jsx::ReactJsxdev => write!(f, "react-jsxdev"),
      Jsx::ReactNative => write!(f, "react-native"),
      Jsx::Preserve => write!(f, "preserve"),
    }
  }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Eq, PartialOrd, Ord, JsonSchema)]
pub enum Lib {
  #[serde(alias = "ES5")]
  Es5,
  #[serde(alias = "ES2015")]
  Es2015,
  #[serde(alias = "ES6")]
  Es6,
  #[serde(alias = "ES2016")]
  Es2016,
  #[serde(alias = "ES7")]
  Es7,
  #[serde(alias = "ES2017")]
  Es2017,
  #[serde(alias = "ES2018")]
  Es2018,
  #[serde(alias = "ES2019")]
  Es2019,
  #[serde(alias = "ES2020")]
  Es2020,
  #[serde(alias = "ESNext")]
  EsNext,
  #[serde(alias = "DOM")]
  Dom,
  WebWorker,
  ScriptHost,
}

impl Display for Lib {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Lib::Es5 => write!(f, "ES5"),
      Lib::Es2015 => write!(f, "ES2015"),
      Lib::Es6 => write!(f, "ES6"),
      Lib::Es2016 => write!(f, "ES2016"),
      Lib::Es7 => write!(f, "ES7"),
      Lib::Es2017 => write!(f, "ES2017"),
      Lib::Es2018 => write!(f, "ES2018"),
      Lib::Es2019 => write!(f, "ES2019"),
      Lib::Es2020 => write!(f, "ES2020"),
      Lib::EsNext => write!(f, "ESNext"),
      Lib::Dom => write!(f, "DOM"),
      Lib::WebWorker => write!(f, "WebWorker"),
      Lib::ScriptHost => write!(f, "ScriptHost"),
    }
  }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Copy, Clone, Default, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ModuleDetection {
  #[default]
  Auto,
  Legacy,
  Force,
}

impl Display for ModuleDetection {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ModuleDetection::Auto => write!(f, "auto"),
      ModuleDetection::Legacy => write!(f, "legacy"),
      ModuleDetection::Force => write!(f, "force"),
    }
  }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Copy, Clone, Eq, JsonSchema)]
pub enum ModuleResolution {
  #[serde(rename = "node", alias = "Node", alias = "node10", alias = "Node10")]
  Node,
  #[serde(rename = "node16", alias = "Node16")]
  Node16,
  #[serde(rename = "nodenext", alias = "NodeNext", alias = "nodeNext")]
  NodeNext,
  #[serde(rename = "bundler", alias = "Bundler")]
  Bundler,
}

impl Display for ModuleResolution {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ModuleResolution::Node => write!(f, "Node"),
      ModuleResolution::Node16 => write!(f, "Node16"),
      ModuleResolution::NodeNext => write!(f, "NodeNext"),
      ModuleResolution::Bundler => write!(f, "Bundler"),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub enum Module {
  #[serde(alias = "none")]
  None,
  #[serde(alias = "commonJs", alias = "commonjs")]
  CommonJs,
  #[serde(alias = "umd")]
  Umd,
  #[serde(alias = "amd")]
  Amd,
  #[serde(alias = "system")]
  System,
  #[serde(alias = "es6")]
  Es6,
  #[serde(alias = "es2015")]
  Es2015,
  #[serde(alias = "es2020")]
  Es2020,
  #[serde(alias = "es2022")]
  Es2022,
  #[serde(alias = "ESNext", alias = "esnext", alias = "esNext")]
  EsNext,
  #[serde(alias = "node16")]
  Node16,
  #[serde(alias = "node18")]
  Node18,
  #[serde(alias = "node20")]
  Node20,
  #[serde(alias = "nodenext", alias = "nodeNext")]
  NodeNext,
  #[serde(alias = "preserve")]
  Preserve,
}

impl Display for Module {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Module::None => write!(f, "none"),
      Module::Umd => write!(f, "Umd"),
      Module::Amd => write!(f, "Amd"),
      Module::CommonJs => write!(f, "CommonJs"),
      Module::Es6 => write!(f, "Es6"),
      Module::Es2015 => write!(f, "Es2015"),
      Module::Es2020 => write!(f, "Es2020"),
      Module::Es2022 => write!(f, "Es2022"),
      Module::EsNext => write!(f, "EsNext"),
      Module::System => write!(f, "System"),
      Module::Node16 => write!(f, "Node16"),
      Module::Node18 => write!(f, "Node18"),
      Module::Node20 => write!(f, "Node20"),
      Module::NodeNext => write!(f, "NodeNext"),
      Module::Preserve => write!(f, "Preserve"),
    }
  }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
pub enum Target {
  #[serde(alias = "es3")]
  Es3,
  #[serde(alias = "es5")]
  Es5,
  #[serde(alias = "es6")]
  Es6,
  #[serde(alias = "es7")]
  Es7,
  #[serde(alias = "es2015")]
  Es2015,
  #[serde(alias = "es2016")]
  Es2016,
  #[serde(alias = "es2017")]
  Es2017,
  #[serde(alias = "es2018")]
  Es2018,
  #[serde(alias = "es2019")]
  Es2019,
  #[serde(alias = "es2020")]
  Es2020,
  #[serde(alias = "esnext", alias = "ESNext")]
  EsNext,
}

impl Display for Target {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Target::Es3 => write!(f, "Es3"),
      Target::Es5 => write!(f, "Es5"),
      Target::Es2015 => write!(f, "Es2015"),
      Target::Es6 => write!(f, "Es6"),
      Target::Es2016 => write!(f, "Es2016"),
      Target::Es7 => write!(f, "Es7"),
      Target::Es2017 => write!(f, "Es2017"),
      Target::Es2018 => write!(f, "Es2018"),
      Target::Es2019 => write!(f, "Es2019"),
      Target::Es2020 => write!(f, "Es2020"),
      Target::EsNext => write!(f, "EsNext"),
    }
  }
}
