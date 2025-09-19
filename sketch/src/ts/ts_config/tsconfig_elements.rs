use std::{collections::BTreeMap, fmt::Display};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::JsonValueBTreeMap;

/// A Typescript plugin definition.
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

/// A reference to a Typescript project. Requires TypeScript version 3.0 or later.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, JsonSchema)]
pub struct TsConfigReference {
  /// Path to referenced tsconfig or to folder containing tsconfig.
  pub path: String,
}

/// Set the newline character for emitting files. See more: https://www.typescriptlang.org/tsconfig#newLine
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

/// Specify how the TypeScript watch mode works. See more: https://www.typescriptlang.org/tsconfig#watchFile
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
  #[serde(alias = "fixedchunksizepolling")]
  FixedChunkSizePolling,
}

impl Display for WatchFile {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      WatchFile::FixedPollingInterval => write!(f, "fixedPollingInterval"),
      WatchFile::PriorityPollingInterval => write!(f, "priorityPollingInterval"),
      WatchFile::DynamicPriorityPolling => write!(f, "dynamicPriorityPolling"),
      WatchFile::UseFsEvents => write!(f, "useFsEvents"),
      WatchFile::UseFsEventsOnParentDirectory => write!(f, "useFsEventsOnParentDirectory"),
      WatchFile::FixedChunkSizePolling => write!(f, "fixedChunkSizePolling"),
    }
  }
}

/// Specify how directories are watched on systems that lack recursive file-watching functionality. See more: https://www.typescriptlang.org/tsconfig#watchDirectory
#[derive(Deserialize, Debug, Clone, Serialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum WatchDirectory {
  #[serde(alias = "usefsevents")]
  UseFsEvents,
  #[serde(alias = "fixedpollinginterval")]
  FixedPollingInterval,
  #[serde(alias = "dynamicprioritypolling")]
  DynamicPriorityPolling,
  #[serde(alias = "fixedchunksizepolling")]
  FixedChunkSizePolling,
}

impl Display for WatchDirectory {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      WatchDirectory::FixedPollingInterval => write!(f, "fixedPollingInterval"),
      WatchDirectory::DynamicPriorityPolling => write!(f, "dynamicPriorityPolling"),
      WatchDirectory::UseFsEvents => write!(f, "useFsEvents"),
      WatchDirectory::FixedChunkSizePolling => write!(f, "fixedChunkSizePolling"),
    }
  }
}

/// Specify what approach the watcher should use if the system runs out of native file watchers. See more: https://www.typescriptlang.org/tsconfig#fallbackPolling
#[derive(Deserialize, Debug, Clone, Serialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum FallbackPolling {
  #[serde(alias = "fixedpollinginterval")]
  FixedPollingInterval,
  #[serde(alias = "prioritypollinginterval")]
  PriorityPollingInterval,
  #[serde(alias = "dynamicprioritypolling")]
  DynamicPriorityPolling,
  #[serde(alias = "fixedinterval")]
  FixedInterval,
  #[serde(alias = "priorityinterval")]
  PriorityInterval,
  #[serde(alias = "dynamicpriority")]
  DynamicPriority,
  #[serde(alias = "fixedchunksize")]
  FixedChunkSize,
}

impl Display for FallbackPolling {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::FixedPollingInterval => write!(f, "fixedPollingInterval"),
      Self::PriorityPollingInterval => write!(f, "priorityPollingInterval"),
      Self::DynamicPriorityPolling => write!(f, "dynamicPriorityPolling"),
      Self::DynamicPriority => write!(f, "dynamicPriority"),
      Self::PriorityInterval => write!(f, "priorityInterval"),
      Self::FixedInterval => write!(f, "fixedInterval"),
      Self::FixedChunkSize => write!(f, "fixedChunkSize"),
    }
  }
}

/// Specify what JSX code is generated. See more: https://www.typescriptlang.org/tsconfig/#jsx
#[derive(Deserialize, Serialize, Debug, PartialEq, Copy, Clone, Eq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Jsx {
  /// Emit .js files with JSX changed to the equivalent React.createElement calls
  React,
  /// Emit .js files with the JSX changed to _jsx calls optimized for production
  ReactJsx,
  /// Emit .js files with the JSX changed to _jsx calls for development only
  ReactJsxdev,
  /// Emit .js files with the JSX unchanged
  ReactNative,
  /// Emit .jsx files with the JSX unchanged
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

/// Specify a set of bundled library declaration files that describe the target runtime environment. See more: https://www.typescriptlang.org/tsconfig#lib
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Eq, PartialOrd, Ord, JsonSchema)]
pub enum Lib {
  /// Core definitions for all ES5 functionality
  #[serde(alias = "ES5")]
  Es5,
  /// Additional APIs available in ES2015 (also known as ES6) - array.find, Promise, Proxy, Symbol, Map, Set, Reflect, etc.
  #[serde(alias = "ES2015", alias = "Es6", alias = "ES6")]
  Es2015,
  /// Additional APIs available in ES2016 - array.include, etc.
  #[serde(alias = "ES2016", alias = "Es7", alias = "ES7")]
  Es2016,
  /// Additional APIs available in ES2017 - Object.entries, Object.values, Atomics, SharedArrayBuffer, date.formatToParts, typed arrays, etc.
  #[serde(alias = "ES2017")]
  Es2017,
  /// Additional APIs available in ES2018 - async iterables, promise.finally, Intl.PluralRules, regexp.groups, etc.
  #[serde(alias = "ES2018")]
  Es2018,
  /// Additional APIs available in ES2019 - array.flat, array.flatMap, Object.fromEntries, string.trimStart, string.trimEnd, etc.
  #[serde(alias = "ES2019")]
  Es2019,
  /// Additional APIs available in ES2020 - string.matchAll, etc.
  #[serde(alias = "ES2020")]
  Es2020,
  /// Additional APIs available in ES2021 - promise.any, string.replaceAll etc.
  #[serde(alias = "ES2021")]
  Es2021,
  /// Additional APIs available in ES2022 - array.at, RegExp.hasIndices, etc.
  #[serde(alias = "ES2022")]
  Es2022,
  /// Additional APIs available in ES2023 - array.with, array.findLast, array.findLastIndex, array.toSorted, array.toReversed, etc.
  #[serde(alias = "ES2023")]
  Es2023,
  /// Additional APIs available in ESNext - This changes as the JavaScript specification evolves
  #[serde(alias = "ESNext")]
  EsNext,
  /// DOM definitions - window, document, etc.
  #[serde(alias = "DOM")]
  Dom,
  /// APIs available in WebWorker contexts
  WebWorker,
  /// APIs for the Windows Script Hosting System
  ScriptHost,
}

impl Display for Lib {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Lib::Es5 => write!(f, "ES5"),
      Lib::Es2015 => write!(f, "ES2015"),
      Lib::Es2016 => write!(f, "ES2016"),
      Lib::Es2017 => write!(f, "ES2017"),
      Lib::Es2018 => write!(f, "ES2018"),
      Lib::Es2019 => write!(f, "ES2019"),
      Lib::Es2020 => write!(f, "ES2020"),
      Lib::Es2021 => write!(f, "ES2021"),
      Lib::Es2022 => write!(f, "ES2022"),
      Lib::Es2023 => write!(f, "ES2023"),
      Lib::EsNext => write!(f, "ESNext"),
      Lib::Dom => write!(f, "DOM"),
      Lib::WebWorker => write!(f, "WebWorker"),
      Lib::ScriptHost => write!(f, "ScriptHost"),
    }
  }
}

/// Specify how TypeScript determine a file as module. See more: https://www.typescriptlang.org/tsconfig/#moduleDetection
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Copy, Clone, Default, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ModuleDetection {
  /// TypeScript will not only look for import and export statements, but it will also check whether the "type" field in a package.json is set to "module" when running with module: nodenext or node16, and check whether the current file is a JSX file when running under jsx: react-jsx
  #[default]
  Auto,
  /// The same behavior as 4.6 and prior, usings import and export statements to determine whether a file is a module.
  Legacy,
  /// Ensures that every non-declaration file is treated as a module.
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
  /// When combined with the corresponding module values, picks the right algorithm for each resolution based on whether Node.js will see an import or require in the output JavaScript code
  #[serde(rename = "node16", alias = "Node16")]
  Node16,
  /// When combined with the corresponding module values, picks the right algorithm for each resolution based on whether Node.js will see an import or require in the output JavaScript code
  #[serde(rename = "nodenext", alias = "NodeNext", alias = "nodeNext")]
  NodeNext,
  /// For use with bundlers. Like node16 and nodenext, this mode supports package.json "imports" and "exports", but unlike the Node.js resolution modes, bundler never requires file extensions on relative paths in imports.
  #[serde(rename = "bundler", alias = "Bundler")]
  Bundler,
}

impl Display for ModuleResolution {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ModuleResolution::Node16 => write!(f, "Node16"),
      ModuleResolution::NodeNext => write!(f, "NodeNext"),
      ModuleResolution::Bundler => write!(f, "Bundler"),
    }
  }
}

/// Specify what module code is generated. See more: https://www.typescriptlang.org/tsconfig#module
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

/// Set the JavaScript language version for emitted JavaScript and include compatible library declarations. See more: https://www.typescriptlang.org/tsconfig#target
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
