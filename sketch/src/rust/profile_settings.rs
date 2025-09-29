use std::collections::BTreeMap;

use merge::Merge;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{merge_btree_maps, merge_optional_nested, overwrite_if_some};

/// Handling of LTO in a build profile
#[derive(Debug, Clone, PartialEq, Serialize, Eq, PartialOrd, Ord, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum LtoSetting {
  /// off
  None,
  /// false
  ThinLocal,
  Thin,
  /// True
  Fat,
}

/// Verbosity of debug info in a [`Profile`]
#[derive(
  Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum DebugSetting {
  /// 0 or false
  None = 0,
  /// 1 = line tables only
  Lines = 1,
  /// 2 or true
  Full = 2,
}

/// Handling of debug symbols in a build profile
#[derive(
  Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, JsonSchema, Serialize,
)]
#[serde(rename_all = "kebab-case")]
pub enum StripSetting {
  /// Same as `strip = false`
  None,
  /// Detailed debug is stripped, but coarse debug is preserved
  Debuginfo,
  /// Stronger than the `Debuginfo` setting, same as `strip = true`
  Symbols,
}

/// Compilation/optimization settings for a workspace
#[derive(Debug, Clone, PartialEq, Default, JsonSchema, Serialize, Deserialize, Merge)]
#[merge(strategy = overwrite_if_some)]
#[serde(rename_all = "kebab-case")]
pub struct Profile {
  /// num or z, s
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub opt_level: Option<Value>,

  /// 0,1,2 or bool
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub debug: Option<DebugSetting>,

  /// Move debug info to separate files
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub split_debuginfo: Option<String>,

  /// For dynamic libraries
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub rpath: Option<bool>,

  /// Link-time-optimization
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub lto: Option<LtoSetting>,

  /// Extra assertions
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub debug_assertions: Option<bool>,

  /// Parallel compilation
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub codegen_units: Option<u16>,

  /// Handling of panics/unwinding
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub panic: Option<String>,

  /// Support for incremental rebuilds
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub incremental: Option<bool>,

  /// Check integer arithmetic
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub overflow_checks: Option<bool>,

  /// Remove debug info
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub strip: Option<StripSetting>,

  /// Profile overrides for dependencies, `*` is special.
  #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
  #[merge(strategy = merge_btree_maps)]
  pub package: BTreeMap<String, Value>,

  /// Profile overrides for build dependencies, `*` is special.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub build_override: Option<Value>,

  /// Only relevant for non-standard profiles
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub inherits: Option<String>,
}

/// Build-in an custom build/optimization settings
#[derive(Debug, Clone, PartialEq, Default, JsonSchema, Serialize, Deserialize, Merge)]
#[merge(strategy = merge_optional_nested)]
pub struct Profiles {
  /// Used for `--release`
  #[serde(skip_serializing_if = "Option::is_none")]
  pub release: Option<Profile>,

  /// Used by default, weirdly called `debug` profile.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub dev: Option<Profile>,

  /// Used for `cargo test`
  #[serde(skip_serializing_if = "Option::is_none")]
  pub test: Option<Profile>,

  /// Used for `cargo bench` (nightly)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub bench: Option<Profile>,

  /// User-suppiled for `cargo --profile=name`
  #[serde(flatten)]
  #[merge(strategy = merge_btree_maps)]
  pub custom: BTreeMap<String, Profile>,
}
