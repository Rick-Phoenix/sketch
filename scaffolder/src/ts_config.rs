use std::{
  collections::{BTreeMap, HashMap},
  fmt::Display,
};

use askama::Template;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::package::PackageKind;

#[derive(Deserialize, Debug, Clone, Serialize, Template, Default)]
#[template(path = "tsconfig.json.j2")]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct TsConfig {
  pub extends: Option<String>,
  pub files: Option<Vec<String>>,
  pub exclude: Option<Vec<String>>,
  pub include: Option<Vec<String>>,
  pub references: Option<Vec<TsConfigReference>>,
  pub type_acquisition: Option<TypeAcquisition>,
  pub compiler_options: Option<CompilerOptions>,
}

#[derive(Deserialize, Debug, Clone, Serialize, Template)]
#[template(path = "type_acquisition.j2")]
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
  Es5,
  Es2015,
  Es6,
  Es2016,
  Es7,
  Es2017,
  Es2018,
  Es2019,
  Es2020,
  EsNext,
  Dom,
  WebWorker,
  ScriptHost,
  Other(String),
}

impl Display for Lib {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Lib::Es5 => write!(f, "Es5"),
      Lib::Es2015 => write!(f, "Es2015"),
      Lib::Es6 => write!(f, "Es6"),
      Lib::Es2016 => write!(f, "Es2016"),
      Lib::Es7 => write!(f, "Es7"),
      Lib::Es2017 => write!(f, "Es2017"),
      Lib::Es2018 => write!(f, "Es2018"),
      Lib::Es2019 => write!(f, "Es2019"),
      Lib::Es2020 => write!(f, "Es2020"),
      Lib::EsNext => write!(f, "EsNext"),
      Lib::Dom => write!(f, "Dom"),
      Lib::WebWorker => write!(f, "WebWorker"),
      Lib::ScriptHost => write!(f, "ScriptHost"),
      Lib::Other(v) => write!(f, "{}", v),
    }
  }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Copy, Clone, Default)]
#[serde(rename_all = "lowercase")]
pub enum ModuleDetectionMode {
  #[default]
  Auto,
  Legacy,
  Force,
}

impl Display for ModuleDetectionMode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ModuleDetectionMode::Auto => write!(f, "auto"),
      ModuleDetectionMode::Legacy => write!(f, "legacy"),
      ModuleDetectionMode::Force => write!(f, "force"),
    }
  }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Copy, Clone)]
pub enum ModuleResolutionMode {
  #[serde(rename = "classic", alias = "Classic")]
  Classic,
  #[serde(rename = "node", alias = "Node", alias = "node10", alias = "Node10")]
  Node,
  #[serde(rename = "node16", alias = "Node16")]
  Node16,
  #[serde(rename = "nodenext", alias = "NodeNext")]
  NodeNext,
  #[serde(rename = "bundler", alias = "Bundler")]
  Bundler,
}

impl Display for ModuleResolutionMode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ModuleResolutionMode::Classic => write!(f, "Classic"),
      ModuleResolutionMode::Node => write!(f, "Node"),
      ModuleResolutionMode::Node16 => write!(f, "Node16"),
      ModuleResolutionMode::NodeNext => write!(f, "NodeNext"),
      ModuleResolutionMode::Bundler => write!(f, "Bundler"),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Module {
  CommonJs,
  Es6,
  Es2015,
  Es2020,
  None,
  Umd,
  Amd,
  System,
  EsNext,
  Other(String),
}

impl Display for Module {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Module::CommonJs => write!(f, "CommonJs"),
      Module::Es6 => write!(f, "Es6"),
      Module::Es2015 => write!(f, "Es2015"),
      Module::Es2020 => write!(f, "Es2020"),
      Module::None => write!(f, "none"),
      Module::Umd => write!(f, "Umd"),
      Module::Amd => write!(f, "Amd"),
      Module::System => write!(f, "System"),
      Module::EsNext => write!(f, "EsNext"),
      Module::Other(v) => write!(f, "{}", v),
    }
  }
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub enum Target {
  Es3,
  Es5,
  Es2015,
  Es6,
  Es2016,
  Es7,
  Es2017,
  Es2018,
  Es2019,
  Es2020,
  EsNext,
  Other(String),
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
      Target::Other(v) => write!(f, "{}", v),
    }
  }
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
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
  pub module_detection: Option<ModuleDetectionMode>,
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
  pub module_resolution: Option<ModuleResolutionMode>,
  pub paths: Option<HashMap<String, Vec<String>>>,
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
  pub imports_not_used_as_values: Option<String>,
  pub jsx_factory: Option<String>,
  pub jsx_fragment_factory: Option<String>,
  pub jsx_import_source: Option<String>,
  pub keyof_strings_only: Option<bool>,
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
  pub fallback_polling: Option<String>,
  pub watch_directory: Option<String>,
  pub watch_file: Option<String>,
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
