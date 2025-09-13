use std::collections::{BTreeMap, BTreeSet};

use askama::Template;
use indexmap::{IndexMap, IndexSet};
use merge::Merge;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
  cli::parsers::parse_key_value_pairs, merge_index_sets, overwrite_option, templating::filters,
  GenError, OrderedMap, Preset,
};

pub(crate) mod tsconfig_defaults;
pub(crate) mod tsconfig_elements;

pub use tsconfig_elements::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum TsConfigKind {
  Id(String),
  Config(Box<TsConfig>),
}

impl Default for TsConfigKind {
  fn default() -> Self {
    Self::Config(TsConfig::default().into())
  }
}

/// A struct representing instructions for outputting a tsconfig file.
/// The file name will be joined to the root directory of the package that the generated config will belong to.
#[derive(Deserialize, Debug, Clone, Serialize, PartialEq, JsonSchema)]
pub struct TsConfigDirective {
  pub output: Option<String>,
  pub config: Option<TsConfigKind>,
}

impl Default for TsConfigDirective {
  fn default() -> Self {
    Self {
      output: Some("tsconfig.json".to_string()),
      config: Some(TsConfigKind::default()),
    }
  }
}

impl TsConfigDirective {
  pub(crate) fn from_cli(s: &str) -> Result<TsConfigDirective, String> {
    let mut directive: TsConfigDirective = Default::default();

    let pairs = parse_key_value_pairs("TsConfigDirective", s)?;

    for (key, val) in pairs {
      match key {
        "output" => {
          directive.output = if val.is_empty() {
            None
          } else {
            Some(val.to_string())
          }
        }
        "id" => {
          directive.config = if val.is_empty() {
            None
          } else {
            Some(TsConfigKind::Id(val.to_string()))
          }
        }
        _ => return Err(format!("Invalid key for TsConfigDirective: {}", key)),
      };
    }

    Ok(directive)
  }
}

impl TsConfig {
  fn merge_configs_recursive(
    &mut self,
    store: &IndexMap<String, TsConfig>,
    processed_ids: &mut IndexSet<String>,
  ) -> Result<(), GenError> {
    for id in self.extend_presets.clone() {
      let was_absent = processed_ids.insert(id.clone());

      if !was_absent {
        let chain: Vec<&str> = processed_ids.iter().map(|s| s.as_str()).collect();

        return Err(GenError::CircularDependency(format!(
          "Found circular dependency for tsconfig '{}'. The full processed chain is: {}",
          id,
          chain.join(" -> ")
        )));
      }

      let mut target = store
        .get(id.as_str())
        .ok_or(GenError::PresetNotFound {
          kind: Preset::TsConfig,
          name: id.to_string(),
        })?
        .clone();

      target.merge_configs_recursive(store, processed_ids)?;

      self.merge(target);
    }

    Ok(())
  }

  pub fn merge_configs(
    mut self,
    initial_id: &str,
    store: &IndexMap<String, TsConfig>,
  ) -> Result<TsConfig, GenError> {
    let mut processed_ids: IndexSet<String> = Default::default();

    processed_ids.insert(initial_id.to_string());

    self.merge_configs_recursive(store, &mut processed_ids)?;

    Ok(self)
  }
}

#[derive(Deserialize, Debug, Clone, Serialize, Template, PartialEq, Eq, JsonSchema)]
#[template(path = "watch_options.j2")]
#[serde(rename_all = "camelCase")]
pub struct WatchOptions {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub watch_file: Option<WatchFile>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub watch_directory: Option<WatchDirectory>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub fallback_polling: Option<FallbackPolling>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub synchronous_watch_directory: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub exclude_directories: Option<BTreeSet<String>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub exclude_files: Option<BTreeSet<String>>,
}

#[derive(
  Deserialize, Debug, Clone, Serialize, Template, Default, Merge, PartialEq, Eq, JsonSchema,
)]
#[template(path = "tsconfig.json.j2")]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct TsConfig {
  #[merge(strategy = merge_index_sets)]
  #[serde(rename = "extend_presets", skip_serializing)]
  pub extend_presets: IndexSet<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = overwrite_option)]
  pub extends: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = overwrite_option)]
  pub files: Option<BTreeSet<String>>,

  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = overwrite_option)]
  pub exclude: Option<BTreeSet<String>>,

  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = overwrite_option)]
  pub include: Option<BTreeSet<String>>,

  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = overwrite_option)]
  pub references: Option<BTreeSet<TsConfigReference>>,

  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = overwrite_option)]
  pub type_acquisition: Option<TypeAcquisition>,

  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = overwrite_option)]
  pub compiler_options: Option<CompilerOptions>,

  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = overwrite_option)]
  pub watch_options: Option<WatchOptions>,
}

#[derive(Deserialize, Debug, Clone, Serialize, Template, PartialEq, Eq, JsonSchema)]
#[template(path = "type_acquisition.j2")]
#[serde(untagged)]
pub enum TypeAcquisition {
  Bool(bool),
  Object {
    enable: bool,
    include: Option<BTreeSet<String>>,
    exclude: Option<BTreeSet<String>>,
    disable_filename_based_type_acquisition: Option<bool>,
  },
}

#[derive(Deserialize, Serialize, Debug, Clone, Default, Merge, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
#[merge(strategy = overwrite_option)]
pub struct CompilerOptions {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub plugins: Option<Vec<OrderedMap>>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub paths: Option<BTreeMap<String, BTreeSet<String>>>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub allow_js: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub check_js: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub composite: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub declaration: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub declaration_map: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub downlevel_iteration: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub import_helpers: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub incremental: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub isolated_modules: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub jsx: Option<Jsx>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub lib: Option<BTreeSet<Lib>>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub module: Option<Module>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub module_detection: Option<ModuleDetection>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_emit: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub out_dir: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub out_file: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub remove_comments: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub root_dir: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub source_map: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub target: Option<Target>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub ts_build_info_file: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub always_strict: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_implicit_any: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_implicit_this: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub strict: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub strict_bind_call_apply: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub strict_function_types: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub strict_null_checks: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub strict_property_initialization: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub allow_synthetic_default_imports: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub allow_umd_global_access: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub base_url: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub es_module_interop: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub module_resolution: Option<ModuleResolution>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub preserve_symlinks: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub root_dirs: Option<BTreeSet<String>>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub type_roots: Option<BTreeSet<String>>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub types: Option<BTreeSet<String>>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub inline_source_map: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub inline_sources: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub map_root: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub source_root: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_fallthrough_cases_in_switch: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_implicit_returns: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_property_access_from_index_signature: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_unchecked_indexed_access: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_unused_locals: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub emit_decorator_metadata: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub experimental_decorators: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub allow_unreachable_code: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub allow_unused_labels: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub assume_changes_only_affect_direct_dependencies: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub declaration_dir: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub disable_referenced_project_load: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub disable_size_limit: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub disable_solution_searching: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub disable_source_of_project_reference_redirect: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "emitBOM")]
  pub emit_bom: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub emit_declaration_only: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub explain_files: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub extended_diagnostics: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub force_consistent_casing_in_file_names: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub jsx_factory: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub jsx_fragment_factory: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub jsx_import_source: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub list_emitted_files: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub list_files: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub max_node_module_js_depth: Option<u32>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_emit_helpers: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_emit_on_error: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_error_truncation: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_implicit_use_strict: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_lib: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_resolve: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_strict_generic_checks: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub preserve_const_enums: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub react_namespace: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub resolve_json_module: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub skip_default_lib_check: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub skip_lib_check: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub strip_internal: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub suppress_excess_property_errors: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub suppress_implicit_any_index_errors: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub trace_resolution: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub use_define_for_class_fields: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub preserve_watch_output: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub pretty: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub verbatim_module_syntax: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub exact_optional_property_types: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_implicit_override: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_unused_parameters: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub strict_builtin_iterator_return: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub use_unknown_in_catch_variables: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub allow_arbitrary_extensions: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub allow_importing_ts_extensions: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_unchecked_side_effects_imports: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub resolve_package_json_exports: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub resolve_package_json_imports: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub rewrite_relative_imports_extensions: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub erasable_syntax_only: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub isolated_declarations: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub lib_replacement: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub generate_trace: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_check: Option<bool>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub custom_conditions: Option<BTreeSet<String>>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub module_suffixes: Option<BTreeSet<String>>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub new_line: Option<NewLine>,
}
