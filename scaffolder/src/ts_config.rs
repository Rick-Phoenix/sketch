use std::{collections::BTreeMap, fmt::Display};

use askama::Template;
use merge::Merge;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{overwrite_option, package::PackageKind, GenError, Preset};

impl TsConfig {
  fn merge_configs_recursive(
    &mut self,
    store: &BTreeMap<String, TsConfig>,
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
    store: &BTreeMap<String, TsConfig>,
  ) -> Result<TsConfig, GenError> {
    let mut processed_ids: Vec<String> = Default::default();

    processed_ids.push(initial_id.to_string());

    self.merge_configs_recursive(store, &mut processed_ids)?;

    Ok(self)
  }
}

#[derive(Deserialize, Debug, Clone, Serialize, Template)]
#[template(path = "watch_options.j2")]
#[serde(rename_all = "camelCase")]
pub struct WatchOptions {
  pub watch_file: Option<WatchFile>,
  pub watch_directory: Option<WatchDirectory>,
  pub fallback_polling: Option<FallbackPolling>,
  pub synchronous_watch_directory: Option<bool>,
  pub exclude_directories: Option<Vec<String>>,
  pub exclude_files: Option<Vec<String>>,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
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

#[derive(Deserialize, Debug, Clone, Serialize)]
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

#[derive(Deserialize, Debug, Clone, Serialize)]
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

#[derive(Deserialize, Debug, Clone, Serialize)]
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

mod filters {
  pub fn strip_trailing_comma<T: std::fmt::Display>(
    s: T,
    _: &dyn askama::Values,
  ) -> askama::Result<String> {
    let mut s = s.to_string();
    let last_non_whitespace_idx_byte = s
      .char_indices()
      .rev()
      .find(|(_, c)| !c.is_whitespace())
      .map(|(idx, _)| idx);

    if let Some(idx) = last_non_whitespace_idx_byte {
      let char_at_idx = s[idx..].chars().next();

      if char_at_idx == Some(',') {
        s.replace_range(idx..idx + ','.len_utf8(), "");
      }
    };

    Ok(s)
  }
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct TsConfigDirective {
  pub file_name: String,
  pub id: String,
}

#[derive(Deserialize, Debug, Clone, Serialize, Template, Default, Merge)]
#[template(path = "tsconfig.json.j2")]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct TsConfig {
  #[merge(strategy = merge::vec::append)]
  #[serde(rename = "extend_presets")]
  pub extend_presets: Vec<String>,
  #[merge(strategy = overwrite_option)]
  pub extends: Option<String>,
  #[merge(strategy = overwrite_option)]
  pub files: Option<Vec<String>>,
  #[merge(strategy = overwrite_option)]
  pub exclude: Option<Vec<String>>,
  #[merge(strategy = overwrite_option)]
  pub include: Option<Vec<String>>,
  #[merge(strategy = overwrite_option)]
  pub references: Option<Vec<TsConfigReference>>,
  #[merge(strategy = overwrite_option)]
  pub type_acquisition: Option<TypeAcquisition>,
  #[merge(strategy = overwrite_option)]
  pub compiler_options: Option<CompilerOptions>,
  #[merge(strategy = overwrite_option)]
  pub watch_options: Option<WatchOptions>,
}

#[derive(Deserialize, Debug, Clone, Serialize, Template)]
#[template(path = "type_acquisition.j2")]
#[serde(untagged)]
pub enum TypeAcquisition {
  Bool(bool),
  Object {
    enable: bool,
    include: Option<Vec<String>>,
    exclude: Option<Vec<String>>,
    disable_filename_based_type_acquisition: Option<bool>,
  },
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Copy, Clone)]
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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
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

#[derive(Deserialize, Serialize, Debug, PartialEq, Copy, Clone, Default)]
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

#[derive(Deserialize, Serialize, Debug, PartialEq, Copy, Clone)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
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

#[derive(Deserialize, Serialize, Debug, Clone, Default, Merge)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
#[merge(strategy = overwrite_option)]
pub struct CompilerOptions {
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
  pub lib: Option<Vec<Lib>>,
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
  pub paths: Option<BTreeMap<String, Vec<String>>>,
  pub preserve_symlinks: Option<bool>,
  pub root_dirs: Option<Vec<String>>,
  pub type_roots: Option<Vec<String>>,
  pub types: Option<Vec<String>>,
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
  pub plugins: Option<Vec<BTreeMap<String, Value>>>,
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
  pub custom_conditions: Option<Vec<String>>,
  pub module_suffixes: Option<Vec<String>>,
  pub new_line: Option<NewLine>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TsConfigReference {
  pub path: String,
}

impl From<PackageKind> for TsConfigKind {
  fn from(value: PackageKind) -> Self {
    match value {
      PackageKind::Library => Self::Library,
      PackageKind::App => Self::App,
    }
  }
}

#[derive(Debug, Default, Copy, Clone)]
pub enum TsConfigKind {
  Root,
  App,
  #[default]
  Library,
}

#[cfg(test)]
mod test {
  use std::{fs::File, path::PathBuf};

  use maplit::btreemap;

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
        root_dirs: Some(vec!["abc".to_string(), "abc".to_string()]),
        types: Some(vec!["abc".to_string(), "abc".to_string()]),
        type_roots: Some(vec!["abc".to_string(), "abc".to_string()]),
        lib: Some(vec![Lib::Dom, Lib::EsNext]),
        paths: Some(
          btreemap! { "@".to_string() => vec!["src/".to_string()], "@components".to_string() => vec!["src/components".to_string()] },
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
        module_suffixes: Some(vec!["abc".to_string(), "abc".to_string()]),
        custom_conditions: Some(vec!["abc".to_string(), "abc".to_string()]),
        plugins: Some(vec![
          btreemap! { "name".to_string() => Value::String("typescript-svelte-plugin".to_string()), "enabled".to_string() => Value::Bool(true), "assumeIsSvelteProject".to_string() => Value::Bool(true) },
        ]),
      }),
      extends: Some("tsconfig.options.json".to_string()),
      files: Some(vec!["*.ts".to_string(), "*.js".to_string()]),
      include: Some(vec!["*.ts".to_string(), "*.js".to_string()]),
      exclude: Some(vec!["*.ts".to_string(), "*.js".to_string()]),
      references: Some(vec![
        TsConfigReference {
          path: "abc.json".to_string(),
        },
        TsConfigReference {
          path: "abc.json".to_string(),
        },
      ]),
      type_acquisition: Some(TypeAcquisition::Object {
        enable: true,
        include: Some(vec!["*.ts".to_string(), "*.js".to_string()]),
        exclude: Some(vec!["*.ts".to_string(), "*.js".to_string()]),
        disable_filename_based_type_acquisition: Some(true),
      }),
      watch_options: Some(WatchOptions {
        watch_file: Some(WatchFile::UseFsEventsOnParentDirectory),
        watch_directory: Some(WatchDirectory::UseFsEvents),
        fallback_polling: Some(FallbackPolling::DynamicPriorityPolling),
        synchronous_watch_directory: Some(true),
        exclude_directories: Some(vec!["abc".to_string(), "abc".to_string()]),
        exclude_files: Some(vec!["abc".to_string(), "abc".to_string()]),
      }),
    };

    let output_path = PathBuf::from("output/test/tsconfig.json");
    let mut output = File::create(&output_path)?;

    ts_config.write_into(&mut output)?;

    // Check that it deserializes correctly
    let _: TsConfig = serde_json::from_reader(File::open(&output_path)?)
      .expect("Error in TsConfig deserialization in test");

    Ok(())
  }
}
