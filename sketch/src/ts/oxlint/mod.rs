pub mod plugins;
use std::collections::{BTreeMap, BTreeSet};

use indexmap::IndexSet;
use plugins::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::JsonValueBTreeMap;

/// Settings for generating an `oxlint` configuration file.
/// It can be set to true/false (to use defaults or to disable it entirely) or to a literal configuration.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum OxlintConfigSetting {
  Bool(bool),
  Config(OxlintConfig),
}

/// Settings for global variables.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GlobalValue {
  /// Disallows overwriting a global variable.
  Readonly,
  /// Allows the global variable to be overwritten.
  Writeable,
  /// Disables a global variable entirely.
  Off,
}

/// The enforcement setting for a linting rule.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RuleEnforcement {
  /// Disables the rule.
  #[serde(alias = "allow")]
  Off,
  /// Violating the rule triggers a warning.
  Warn,
  /// Violating the rule causes an error.
  #[serde(alias = "deny")]
  Error,
}

/// The settings for an individual rule. Can be a single value such as `warn` or `error`, or an array with the rule enforcement value as the first value, and the rule-specific settings in an object right after that. (example: ["allow", { "setting1": true }])
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(untagged)]
pub enum RuleSetting {
  Simple(RuleEnforcement),
  Custom([(RuleEnforcement, JsonValueBTreeMap); 1]),
}

/// Configure an entire category of rules all at once.Rules enabled or disabled this way will be overwritten by individual rules in the `rules` field.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct Categories {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub correctness: Option<RuleEnforcement>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub nursery: Option<RuleEnforcement>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub pedantic: Option<RuleEnforcement>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub perf: Option<RuleEnforcement>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub restriction: Option<RuleEnforcement>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub style: Option<RuleEnforcement>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub suspicious: Option<RuleEnforcement>,
}

/// Settings to override for a group of files.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct Override {
  /// A list of glob patterns to override.
  pub files: BTreeSet<String>,

  /// Optionally change what plugins are enabled for this override. When omitted, the base config's plugins are used.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub plugins: Option<BTreeSet<Plugin>>,

  /// Enables or disables specific global variables.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub globals: Option<BTreeMap<String, GlobalValue>>,

  /// Environments enable and disable collections of global variables.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub env: Option<BTreeMap<String, bool>>,

  /// Override settings for specific rules.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub rules: Option<BTreeMap<String, RuleSetting>>,
}

/// Configure the behavior of linter plugins.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PluginsSettings {
  /// Settings for the Jsdoc plugin.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub jsdoc: Option<JsDocPluginSettings>,

  /// Settings for the jsx-a11y plugin.
  #[serde(rename = "jsx-a11y")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub jsx_ally: Option<JsxA11yPluginSettings>,

  /// Settings for the nextjs plugin.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub next: Option<NextPluginSettings>,

  /// Settings for the react plugin.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub react: Option<ReactPluginSettings>,
}

/// The configuration directives for `oxlint`. See more: https://oxc.rs/docs/guide/usage/linter/config-file-reference.html
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OxlintConfig {
  /// Paths of configuration files that this configuration file extends (inherits from). The files are resolved relative to the location of the configuration file that contains the `extends` property. The configuration files are merged from the first to the last, with the last file overriding the previous ones.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub extends: Option<IndexSet<String>>,

  /// Environments enable and disable collections of global variables.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub env: Option<BTreeMap<String, bool>>,

  /// Enables or disables specific global variables.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub globals: Option<BTreeMap<String, GlobalValue>>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub categories: Option<Categories>,

  /// Globs to ignore during linting. These are resolved from the configuration file path.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ignore_patterns: Option<BTreeSet<String>>,

  /// Add, remove, or otherwise reconfigure rules for specific files or groups of files.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub overrides: Option<Vec<Override>>,

  /// A list of plugins to enable for this config.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub plugins: Option<BTreeSet<Plugin>>,

  /// Settings for individual rules. See [Oxlint Rules](https://oxc.rs/docs/guide/usage/linter/rules.html) for the list of rules.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub rules: Option<BTreeMap<String, RuleSetting>>,

  /// Contains the settings for various plugins.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub settings: Option<PluginsSettings>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub extras: Option<JsonValueBTreeMap>,
}

impl Default for OxlintConfigSetting {
  fn default() -> Self {
    Self::Bool(true)
  }
}
