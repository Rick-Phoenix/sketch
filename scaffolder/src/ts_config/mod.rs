use std::collections::{BTreeMap, BTreeSet};

use askama::Template;
use indexmap::IndexMap;
use merge::Merge;
use serde::{Deserialize, Serialize};

use crate::{
  merge_sets, overwrite_option, parsers::parse_key_value_pairs, rendering::filters, GenError,
  OrderedMap, Preset,
};

pub(crate) mod tsconfig_defaults;
pub(crate) mod tsconfig_elements;

pub use tsconfig_elements::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Deserialize, Debug, Clone, Serialize)]
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
  pub(crate) fn multiple_from_cli(s: &str) -> Result<Vec<TsConfigDirective>, String> {
    let mut directives: Vec<TsConfigDirective> = Default::default();

    let groups: Vec<&str> = s.split(',').collect();

    for group in groups {
      directives.push(Self::from_cli(group)?);
    }

    Ok(directives)
  }

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
        "config" => {
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
    processed_ids: &mut Vec<String>,
  ) -> Result<(), GenError> {
    for id in self.extend_presets.clone() {
      let was_absent = !processed_ids.contains(&id);
      processed_ids.push(id.clone());

      if !was_absent {
        return Err(GenError::CircularDependency(format!(
          "Found circular dependency for tsconfig '{}'. The full processed chain is: {}",
          id,
          processed_ids.join(" -> ")
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
    let mut processed_ids: Vec<String> = Default::default();

    processed_ids.push(initial_id.to_string());

    self.merge_configs_recursive(store, &mut processed_ids)?;

    Ok(self)
  }
}

#[derive(Deserialize, Debug, Clone, Serialize, Template, PartialEq, Eq)]
#[template(path = "watch_options.j2")]
#[serde(rename_all = "camelCase")]
pub struct WatchOptions {
  pub watch_file: Option<WatchFile>,
  pub watch_directory: Option<WatchDirectory>,
  pub fallback_polling: Option<FallbackPolling>,
  pub synchronous_watch_directory: Option<bool>,
  pub exclude_directories: Option<BTreeSet<String>>,
  pub exclude_files: Option<BTreeSet<String>>,
}

#[derive(Deserialize, Debug, Clone, Serialize, Template, Default, Merge, PartialEq, Eq)]
#[template(path = "tsconfig.json.j2")]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct TsConfig {
  #[merge(strategy = merge_sets)]
  #[serde(rename = "extend_presets")]
  pub extend_presets: BTreeSet<String>,
  #[merge(strategy = overwrite_option)]
  pub extends: Option<String>,
  #[merge(strategy = overwrite_option)]
  pub files: Option<BTreeSet<String>>,
  #[merge(strategy = overwrite_option)]
  pub exclude: Option<BTreeSet<String>>,
  #[merge(strategy = overwrite_option)]
  pub include: Option<BTreeSet<String>>,
  #[merge(strategy = overwrite_option)]
  pub references: Option<BTreeSet<TsConfigReference>>,
  #[merge(strategy = overwrite_option)]
  pub type_acquisition: Option<TypeAcquisition>,
  #[merge(strategy = overwrite_option)]
  pub compiler_options: Option<CompilerOptions>,
  #[merge(strategy = overwrite_option)]
  pub watch_options: Option<WatchOptions>,
}

#[derive(Deserialize, Debug, Clone, Serialize, Template, PartialEq, Eq)]
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

#[derive(Deserialize, Serialize, Debug, Clone, Default, Merge, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
#[merge(strategy = overwrite_option)]
pub struct CompilerOptions {
  pub plugins: Option<Vec<OrderedMap>>,
  pub paths: Option<BTreeMap<String, BTreeSet<String>>>,
  pub allow_js: Option<bool>,
  pub check_js: Option<bool>,
  pub composite: Option<bool>,
  pub declaration: Option<bool>,
  pub declaration_map: Option<bool>,
  pub downlevel_iteration: Option<bool>,
  pub import_helpers: Option<bool>,
  pub incremental: Option<bool>,
  pub isolated_modules: Option<bool>,
  pub jsx: Option<Jsx>,
  pub lib: Option<BTreeSet<Lib>>,
  pub module: Option<Module>,
  pub module_detection: Option<ModuleDetection>,
  pub no_emit: Option<bool>,
  pub out_dir: Option<String>,
  pub out_file: Option<String>,
  pub remove_comments: Option<bool>,
  pub root_dir: Option<String>,
  pub source_map: Option<bool>,
  pub target: Option<Target>,
  pub ts_build_info_file: Option<String>,
  pub always_strict: Option<bool>,
  pub no_implicit_any: Option<bool>,
  pub no_implicit_this: Option<bool>,
  pub strict: Option<bool>,
  pub strict_bind_call_apply: Option<bool>,
  pub strict_function_types: Option<bool>,
  pub strict_null_checks: Option<bool>,
  pub strict_property_initialization: Option<bool>,
  pub allow_synthetic_default_imports: Option<bool>,
  pub allow_umd_global_access: Option<bool>,
  pub base_url: Option<String>,
  pub es_module_interop: Option<bool>,
  pub module_resolution: Option<ModuleResolution>,
  pub preserve_symlinks: Option<bool>,
  pub root_dirs: Option<BTreeSet<String>>,
  pub type_roots: Option<BTreeSet<String>>,
  pub types: Option<BTreeSet<String>>,
  pub inline_source_map: Option<bool>,
  pub inline_sources: Option<bool>,
  pub map_root: Option<String>,
  pub source_root: Option<String>,
  pub no_fallthrough_cases_in_switch: Option<bool>,
  pub no_implicit_returns: Option<bool>,
  pub no_property_access_from_index_signature: Option<bool>,
  pub no_unchecked_indexed_access: Option<bool>,
  pub no_unused_locals: Option<bool>,
  pub emit_decorator_metadata: Option<bool>,
  pub experimental_decorators: Option<bool>,
  pub allow_unreachable_code: Option<bool>,
  pub allow_unused_labels: Option<bool>,
  pub assume_changes_only_affect_direct_dependencies: Option<bool>,
  pub declaration_dir: Option<String>,
  pub disable_referenced_project_load: Option<bool>,
  pub disable_size_limit: Option<bool>,
  pub disable_solution_searching: Option<bool>,
  pub disable_source_of_project_reference_redirect: Option<bool>,
  #[serde(rename = "emitBOM")]
  pub emit_bom: Option<bool>,
  pub emit_declaration_only: Option<bool>,
  pub explain_files: Option<bool>,
  pub extended_diagnostics: Option<bool>,
  pub force_consistent_casing_in_file_names: Option<bool>,
  pub jsx_factory: Option<String>,
  pub jsx_fragment_factory: Option<String>,
  pub jsx_import_source: Option<String>,
  pub list_emitted_files: Option<bool>,
  pub list_files: Option<bool>,
  pub max_node_module_js_depth: Option<u32>,
  pub no_emit_helpers: Option<bool>,
  pub no_emit_on_error: Option<bool>,
  pub no_error_truncation: Option<bool>,
  pub no_implicit_use_strict: Option<bool>,
  pub no_lib: Option<bool>,
  pub no_resolve: Option<bool>,
  pub no_strict_generic_checks: Option<bool>,
  pub preserve_const_enums: Option<bool>,
  pub react_namespace: Option<String>,
  pub resolve_json_module: Option<bool>,
  pub skip_default_lib_check: Option<bool>,
  pub skip_lib_check: Option<bool>,
  pub strip_internal: Option<bool>,
  pub suppress_excess_property_errors: Option<bool>,
  pub suppress_implicit_any_index_errors: Option<bool>,
  pub trace_resolution: Option<bool>,
  pub use_define_for_class_fields: Option<bool>,
  pub preserve_watch_output: Option<bool>,
  pub pretty: Option<bool>,
  pub verbatim_module_syntax: Option<bool>,
  pub exact_optional_property_types: Option<bool>,
  pub no_implicit_override: Option<bool>,
  pub no_unused_parameters: Option<bool>,
  pub strict_builtin_iterator_return: Option<bool>,
  pub use_unknown_in_catch_variables: Option<bool>,
  pub allow_arbitrary_extensions: Option<bool>,
  pub allow_importing_ts_extensions: Option<bool>,
  pub no_unchecked_side_effects_imports: Option<bool>,
  pub resolve_package_json_exports: Option<bool>,
  pub resolve_package_json_imports: Option<bool>,
  pub rewrite_relative_imports_extensions: Option<bool>,
  pub erasable_syntax_only: Option<bool>,
  pub isolated_declarations: Option<bool>,
  pub lib_replacement: Option<bool>,
  pub generate_trace: Option<bool>,
  pub no_check: Option<bool>,
  pub custom_conditions: Option<BTreeSet<String>>,
  pub module_suffixes: Option<BTreeSet<String>>,
  pub new_line: Option<NewLine>,
}

#[cfg(test)]
mod test {
  use std::{fs::File, path::PathBuf};

  use indexmap::indexmap;
  use maplit::{btreemap, btreeset};
  use serde_json::Value;

  use super::*;

  #[test]
  fn tsconfig_generation() -> Result<(), Box<dyn std::error::Error>> {
    let ts_config = TsConfig {
      extend_presets: Default::default(),
      compiler_options: Some(CompilerOptions {
        out_dir: Some("out".to_string()),
        allow_js: Some(true),
        check_js: Some(true),
        composite: Some(true),
        declaration: Some(true),
        declaration_map: Some(true),
        downlevel_iteration: Some(true),
        import_helpers: Some(true),
        incremental: Some(true),
        isolated_modules: Some(true),
        no_emit: Some(true),
        remove_comments: Some(true),
        source_map: Some(true),
        always_strict: Some(true),
        no_implicit_any: Some(true),
        no_implicit_this: Some(true),
        strict_bind_call_apply: Some(true),
        strict_function_types: Some(true),
        strict: Some(true),
        strict_null_checks: Some(true),
        strict_property_initialization: Some(true),
        allow_synthetic_default_imports: Some(true),
        allow_umd_global_access: Some(true),
        es_module_interop: Some(true),
        preserve_symlinks: Some(true),
        inline_source_map: Some(true),
        inline_sources: Some(true),
        no_fallthrough_cases_in_switch: Some(true),
        no_implicit_returns: Some(true),
        no_unchecked_indexed_access: Some(true),
        no_unused_locals: Some(true),
        emit_decorator_metadata: Some(true),
        experimental_decorators: Some(true),
        allow_unreachable_code: Some(true),
        allow_unused_labels: Some(true),
        assume_changes_only_affect_direct_dependencies: Some(true),
        disable_referenced_project_load: Some(true),
        disable_size_limit: Some(true),
        max_node_module_js_depth: Some(15),
        disable_solution_searching: Some(true),
        disable_source_of_project_reference_redirect: Some(true),
        emit_declaration_only: Some(true),
        emit_bom: Some(true),
        explain_files: Some(true),
        extended_diagnostics: Some(true),
        force_consistent_casing_in_file_names: Some(true),
        list_emitted_files: Some(true),
        pretty: Some(true),
        list_files: Some(true),
        no_emit_helpers: Some(true),
        no_emit_on_error: Some(true),
        no_error_truncation: Some(true),
        no_implicit_use_strict: Some(true),
        no_lib: Some(true),
        no_resolve: Some(true),
        no_strict_generic_checks: Some(true),
        preserve_const_enums: Some(true),
        resolve_json_module: Some(true),
        skip_default_lib_check: Some(true),
        skip_lib_check: Some(true),
        strip_internal: Some(true),
        suppress_excess_property_errors: Some(true),
        suppress_implicit_any_index_errors: Some(true),
        trace_resolution: Some(true),
        use_define_for_class_fields: Some(true),
        preserve_watch_output: Some(true),
        no_property_access_from_index_signature: Some(true),
        map_root: Some("abc".to_string()),
        source_root: Some("abc".to_string()),
        declaration_dir: Some("abc".to_string()),
        jsx_factory: Some("abc".to_string()),
        jsx_fragment_factory: Some("abc".to_string()),
        jsx_import_source: Some("abc".to_string()),
        react_namespace: Some("abc".to_string()),
        out_file: Some("abc".to_string()),
        root_dir: Some("abc".to_string()),
        ts_build_info_file: Some("abc".to_string()),
        base_url: Some("abc".to_string()),
        target: Some(Target::EsNext),
        module: Some(Module::EsNext),
        jsx: Some(Jsx::React),
        module_detection: Some(ModuleDetection::Force),
        module_resolution: Some(ModuleResolution::NodeNext),
        root_dirs: Some(btreeset!["abc".to_string(), "abc".to_string()]),
        types: Some(btreeset!["abc".to_string(), "abc".to_string()]),
        type_roots: Some(btreeset!["abc".to_string(), "abc".to_string()]),
        lib: Some(btreeset![Lib::Dom, Lib::EsNext]),
        paths: Some(
          btreemap! { "@".to_string() => btreeset!["src/".to_string()], "@components".to_string() => btreeset!["src/components".to_string()] },
        ),
        verbatim_module_syntax: Some(true),
        new_line: Some(NewLine::Lf),
        rewrite_relative_imports_extensions: Some(true),
        resolve_package_json_imports: Some(true),
        resolve_package_json_exports: Some(true),
        no_unchecked_side_effects_imports: Some(true),
        erasable_syntax_only: Some(true),
        no_check: Some(true),
        generate_trace: Some(true),
        isolated_declarations: Some(true),
        allow_arbitrary_extensions: Some(true),
        use_unknown_in_catch_variables: Some(true),
        no_unused_parameters: Some(true),
        no_implicit_override: Some(true),
        allow_importing_ts_extensions: Some(true),
        exact_optional_property_types: Some(true),
        lib_replacement: Some(true),
        strict_builtin_iterator_return: Some(true),
        module_suffixes: Some(btreeset!["abc".to_string(), "abc".to_string()]),
        custom_conditions: Some(btreeset!["abc".to_string(), "abc".to_string()]),
        plugins: Some(vec![
          indexmap! { "name".to_string() => Value::String("typescript-svelte-plugin".to_string()), "enabled".to_string() => Value::Bool(true), "assumeIsSvelteProject".to_string() => Value::Bool(true) },
        ]),
      }),
      extends: Some("tsconfig.options.json".to_string()),
      files: Some(btreeset!["*.ts".to_string(), "*.js".to_string()]),
      include: Some(btreeset!["*.ts".to_string(), "*.js".to_string()]),
      exclude: Some(btreeset!["*.ts".to_string(), "*.js".to_string()]),
      references: Some(btreeset![
        TsConfigReference {
          path: "abc.json".to_string(),
        },
        TsConfigReference {
          path: "abc.json".to_string(),
        },
      ]),
      type_acquisition: Some(TypeAcquisition::Object {
        enable: true,
        include: Some(btreeset!["*.ts".to_string(), "*.js".to_string()]),
        exclude: Some(btreeset!["*.ts".to_string(), "*.js".to_string()]),
        disable_filename_based_type_acquisition: Some(true),
      }),
      watch_options: Some(WatchOptions {
        watch_file: Some(WatchFile::UseFsEventsOnParentDirectory),
        watch_directory: Some(WatchDirectory::UseFsEvents),
        fallback_polling: Some(FallbackPolling::DynamicPriorityPolling),
        synchronous_watch_directory: Some(true),
        exclude_directories: Some(btreeset!["abc".to_string(), "abc".to_string()]),
        exclude_files: Some(btreeset!["abc".to_string(), "abc".to_string()]),
      }),
    };

    let output_path = PathBuf::from("output/test/tsconfig/tsconfig.json");
    let mut output = File::create(&output_path)?;

    ts_config.write_into(&mut output)?;

    // Check that it deserializes correctly
    let result: TsConfig = serde_json::from_reader(File::open(&output_path)?)
      .expect("Error in TsConfig deserialization in test");

    assert_eq!(result, ts_config);

    Ok(())
  }
}
