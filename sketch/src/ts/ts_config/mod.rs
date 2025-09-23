#[cfg(test)]
mod tsconfig_tests;

pub(crate) mod tsconfig_defaults;
pub(crate) mod tsconfig_elements;

use std::collections::{BTreeMap, BTreeSet};

use indexmap::{IndexMap, IndexSet};
use merge::Merge;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
pub use tsconfig_elements::*;

use crate::{
  cli::parsers::parse_key_value_pairs, merge_index_sets, merge_nested, merge_optional_btree_maps,
  merge_optional_btree_sets, merge_optional_nested, merge_presets, overwrite_if_some, Extensible,
  GenError, Preset,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema, Default, Merge)]
#[serde(default)]
pub struct TsConfigPreset {
  #[merge(strategy = merge_index_sets)]
  pub extend_presets: IndexSet<String>,

  #[serde(flatten)]
  #[merge(strategy = merge_nested)]
  pub tsconfig: TsConfig,
}

impl Extensible for TsConfigPreset {
  fn get_extended(&self) -> &IndexSet<String> {
    &self.extend_presets
  }
}

impl TsConfigPreset {
  pub fn process_data(
    self,
    id: &str,
    store: &IndexMap<String, TsConfigPreset>,
  ) -> Result<TsConfig, GenError> {
    if self.extend_presets.is_empty() {
      return Ok(self.tsconfig);
    }

    let mut processed_ids: IndexSet<String> = IndexSet::new();

    let merged_preset = merge_presets(Preset::TsConfig, id, self, store, &mut processed_ids)?;

    Ok(merged_preset.tsconfig)
  }
}

/// The kind of data for a [`TsConfig`]. It can be a string indicating a preset it, or a full configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum TsConfigKind {
  Id(String),
  Config(TsConfigPreset),
}

impl Default for TsConfigKind {
  fn default() -> Self {
    Self::Config(TsConfigPreset::default())
  }
}

/// A struct representing instructions for generating a tsconfig file.
/// If the output path is relative, it will be joined to the root path of its package.
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

/// Settings for the watch mode in TypeScript.
#[derive(Deserialize, Debug, Clone, Serialize, PartialEq, Eq, JsonSchema, Merge)]
#[merge(strategy = overwrite_if_some)]
#[serde(rename_all = "camelCase")]
pub struct WatchOptions {
  /// Specify how the TypeScript watch mode works. See more: https://www.typescriptlang.org/tsconfig#watchFile
  #[serde(skip_serializing_if = "Option::is_none")]
  pub watch_file: Option<WatchFile>,

  /// Specify how directories are watched on systems that lack recursive file-watching functionality. See more: https://www.typescriptlang.org/tsconfig#watchDirectory
  #[serde(skip_serializing_if = "Option::is_none")]
  pub watch_directory: Option<WatchDirectory>,

  /// Specify what approach the watcher should use if the system runs out of native file watchers. See more: https://www.typescriptlang.org/tsconfig#fallbackPolling
  #[serde(skip_serializing_if = "Option::is_none")]
  pub fallback_polling: Option<FallbackPolling>,

  /// Synchronously call callbacks and update the state of directory watchers on platforms that don`t support recursive watching natively. See more: https://www.typescriptlang.org/tsconfig#synchronousWatchDirectory
  #[serde(skip_serializing_if = "Option::is_none")]
  pub synchronous_watch_directory: Option<bool>,

  /// Remove a list of directories from the watch process. See more: https://www.typescriptlang.org/tsconfig#excludeDirectories
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_btree_sets)]
  pub exclude_directories: Option<BTreeSet<String>>,

  /// Remove a list of files from the watch mode's processing. See more: https://www.typescriptlang.org/tsconfig#excludeFiles
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_btree_sets)]
  pub exclude_files: Option<BTreeSet<String>>,
}

/// A struct representing the contents of a `tsconfig.json` file.
#[derive(Deserialize, Debug, Clone, Serialize, Default, Merge, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
#[merge(strategy = overwrite_if_some)]
pub struct TsConfig {
  /// Path to base configuration file to inherit from (requires TypeScript version 2.1 or later), or array of base files, with the rightmost files having the greater priority (requires TypeScript version 5.0 or later).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub extends: Option<String>,

  /// If no 'files' or 'include' property is present in a tsconfig.json, the compiler defaults to including all files in the containing directory and subdirectories except those specified by 'exclude'. When a 'files' property is specified, only those files and those specified by 'include' are included.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_btree_sets)]
  pub files: Option<BTreeSet<String>>,

  /// Specifies a list of files to be excluded from compilation. The 'exclude' property only affects the files included via the 'include' property and not the 'files' property. Glob patterns require TypeScript version 2.0 or later.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_btree_sets)]
  pub exclude: Option<BTreeSet<String>>,

  /// Specifies a list of glob patterns that match files to be included in compilation. If no 'files' or 'include' property is present in a tsconfig.json, the compiler defaults to including all files in the containing directory and subdirectories except those specified by 'exclude'. Requires TypeScript version 2.0 or later.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_btree_sets)]
  pub include: Option<BTreeSet<String>>,

  /// Referenced projects. Requires TypeScript version 3.0 or later.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_btree_sets)]
  pub references: Option<BTreeSet<TsConfigReference>>,

  /// Auto type (.d.ts) acquisition options for this project. Requires TypeScript version 2.1 or later.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub type_acquisition: Option<TypeAcquisition>,

  /// Settings for the watch mode in TypeScript.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub watch_options: Option<WatchOptions>,

  /// Instructs the TypeScript compiler how to compile .ts files.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_nested)]
  pub compiler_options: Option<CompilerOptions>,
}

/// Auto type (.d.ts) acquisition options for this project. Requires TypeScript version 2.1 or later.
#[derive(Deserialize, Debug, Clone, Serialize, PartialEq, Eq, JsonSchema)]
#[serde(untagged)]
pub enum TypeAcquisition {
  Bool(bool),
  Object {
    /// Enable auto type acquisition
    enable: bool,
    /// Specifies a list of type declarations to be included in auto type acquisition. Ex. ["jquery", "lodash"]
    include: Option<BTreeSet<String>>,
    /// Specifies a list of type declarations to be excluded from auto type acquisition. Ex. ["jquery", "lodash"]
    exclude: Option<BTreeSet<String>>,

    /// TypeScript’s type acquisition can infer what types should be added based on filenames in a project. This means that having a file like jquery.js in your project would automatically download the types for JQuery from DefinitelyTyped.
    /// You can disable this via disableFilenameBasedTypeAcquisition. See more: https://www.typescriptlang.org/it/tsconfig/#type-disableFilenameBasedTypeAcquisition
    #[serde(rename = "disableFilenameBasedTypeAcquisition")]
    disable_filename_based_type_acquisition: Option<bool>,
  },
}

/// Instructs the TypeScript compiler how to compile .ts files.
#[derive(Deserialize, Serialize, Debug, Clone, Default, Merge, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
#[merge(strategy = overwrite_if_some)]
pub struct CompilerOptions {
  /// Enable importing files with any extension, provided a declaration file is present. See more: https://www.typescriptlang.org/tsconfig#allowArbitraryExtensions
  #[serde(skip_serializing_if = "Option::is_none")]
  pub allow_arbitrary_extensions: Option<bool>,

  /// Allow imports to include TypeScript file extensions. Requires either '--noEmit' or '--emitDeclarationOnly' to be set. See more: https://www.typescriptlang.org/tsconfig#allowImportingTsExtensions
  #[serde(skip_serializing_if = "Option::is_none")]
  pub allow_importing_ts_extensions: Option<bool>,

  /// Allow JavaScript files to be imported inside your project, instead of just .ts and .tsx files. See more: https://www.typescriptlang.org/tsconfig/#allowJs
  #[serde(skip_serializing_if = "Option::is_none")]
  pub allow_js: Option<bool>,

  /// Emit additional JavaScript to ease support for importing CommonJS modules. This enables `allowSyntheticDefaultImports` for type compatibility.See more: https://www.typescriptlang.org/tsconfig#esModuleInterop
  #[serde(skip_serializing_if = "Option::is_none")]
  pub allow_synthetic_default_imports: Option<bool>,

  /// Allow accessing UMD globals from modules. See more: https://www.typescriptlang.org/tsconfig#allowUmdGlobalAccess
  #[serde(skip_serializing_if = "Option::is_none")]
  pub allow_umd_global_access: Option<bool>,

  /// Disable error reporting for unreachable code. See more: https://www.typescriptlang.org/tsconfig#allowUnreachableCode
  #[serde(skip_serializing_if = "Option::is_none")]
  pub allow_unreachable_code: Option<bool>,

  /// Disable error reporting for unused labels. See more: https://www.typescriptlang.org/tsconfig#allowUnusedLabels
  #[serde(skip_serializing_if = "Option::is_none")]
  pub allow_unused_labels: Option<bool>,

  /// Ensures that your files are parsed in the ECMAScript strict mode, and emit “use strict” for each source file. See more: https://www.typescriptlang.org/tsconfig/#alwaysStrict
  #[serde(skip_serializing_if = "Option::is_none")]
  pub always_strict: Option<bool>,

  /// Have recompiles in '--incremental' and '--watch' assume that changes within a file will only affect files directly depending on it. Requires TypeScript version 3.8 or later. See more: https://www.typescriptlang.org/tsconfig/#assumeChangesOnlyAffectDirectDependencies
  #[serde(skip_serializing_if = "Option::is_none")]
  pub assume_changes_only_affect_direct_dependencies: Option<bool>,

  /// Specify the base directory to resolve non-relative module names. See more: https://www.typescriptlang.org/tsconfig#baseUrl
  #[serde(skip_serializing_if = "Option::is_none")]
  pub base_url: Option<String>,

  /// Enable error reporting in type-checked JavaScript files. See more: https://www.typescriptlang.org/tsconfig#checkJs
  #[serde(skip_serializing_if = "Option::is_none")]
  pub check_js: Option<bool>,

  /// Enable constraints that allow a TypeScript project to be used with project references. See more: https://www.typescriptlang.org/tsconfig#composite
  #[serde(skip_serializing_if = "Option::is_none")]
  pub composite: Option<bool>,

  /// Conditions to set in addition to the resolver-specific defaults when resolving imports. See more: https://www.typescriptlang.org/tsconfig#customConditions
  #[merge(strategy = merge_optional_btree_sets)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub custom_conditions: Option<BTreeSet<String>>,

  /// Generate .d.ts files from TypeScript and JavaScript files in your project. See more: https://www.typescriptlang.org/tsconfig#declaration
  #[serde(skip_serializing_if = "Option::is_none")]
  pub declaration: Option<bool>,

  /// Specify the output directory for generated declaration files. See more: https://www.typescriptlang.org/tsconfig#declarationDir
  #[serde(skip_serializing_if = "Option::is_none")]
  pub declaration_dir: Option<String>,

  /// Create sourcemaps for d.ts files. See more: https://www.typescriptlang.org/tsconfig#declarationMap
  #[serde(skip_serializing_if = "Option::is_none")]
  pub declaration_map: Option<bool>,

  /// Reduce the number of projects loaded automatically by TypeScript. See more: https://www.typescriptlang.org/tsconfig#disableReferencedProjectLoad
  #[serde(skip_serializing_if = "Option::is_none")]
  pub disable_referenced_project_load: Option<bool>,

  /// Remove the 20mb cap on total source code size for JavaScript files in the TypeScript language server.See more: https://www.typescriptlang.org/tsconfig#disableSizeLimit
  #[serde(skip_serializing_if = "Option::is_none")]
  pub disable_size_limit: Option<bool>,

  /// Opt a project out of multi-project reference checking when editing. See more: https://www.typescriptlang.org/tsconfig#disableSolutionSearching
  #[serde(skip_serializing_if = "Option::is_none")]
  pub disable_solution_searching: Option<bool>,

  /// Disable preferring source files instead of declaration files when referencing composite projects. See more: https://www.typescriptlang.org/tsconfig#disableSourceOfProjectReferenceRedirect
  #[serde(skip_serializing_if = "Option::is_none")]
  pub disable_source_of_project_reference_redirect: Option<bool>,

  /// Emit more compliant, but verbose and less performant JavaScript for iteration. See more: https://www.typescriptlang.org/tsconfig#downlevelIteration
  #[serde(skip_serializing_if = "Option::is_none")]
  pub downlevel_iteration: Option<bool>,

  /// Emit a UTF-8 Byte Order Mark (BOM) in the beginning of output files. See more: https://www.typescriptlang.org/tsconfig#emitBOM
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "emitBOM")]
  pub emit_bom: Option<bool>,

  /// Only output d.ts files and not JavaScript files.See more: https://www.typescriptlang.org/tsconfig#emitDeclarationOnly
  #[serde(skip_serializing_if = "Option::is_none")]
  pub emit_declaration_only: Option<bool>,

  /// Emit design-type metadata for decorated declarations in source files. See more: https://www.typescriptlang.org/tsconfig#emitDecoratorMetadata
  #[serde(skip_serializing_if = "Option::is_none")]
  pub emit_decorator_metadata: Option<bool>,

  /// Do not allow runtime constructs that are not part of ECMAScript. See more: https://www.typescriptlang.org/tsconfig#erasableSyntaxOnly
  #[serde(skip_serializing_if = "Option::is_none")]
  pub erasable_syntax_only: Option<bool>,

  /// Emit additional JavaScript to ease support for importing CommonJS modules. This enables `allowSyntheticDefaultImports` for type compatibility.See more: https://www.typescriptlang.org/tsconfig#esModuleInterop
  #[serde(skip_serializing_if = "Option::is_none")]
  pub es_module_interop: Option<bool>,

  /// Differentiate between undefined and not present when type checking. See more: https://www.typescriptlang.org/tsconfig#exactOptionalPropertyTypes
  #[serde(skip_serializing_if = "Option::is_none")]
  pub exact_optional_property_types: Option<bool>,

  /// Output more detailed compiler performance information after building. See more: https://www.typescriptlang.org/tsconfig#extendedDiagnostics
  #[serde(skip_serializing_if = "Option::is_none")]
  pub extended_diagnostics: Option<bool>,

  /// Enable experimental support for TC39 stage 2 draft decorators. See more: https://www.typescriptlang.org/tsconfig#experimentalDecorators
  #[serde(skip_serializing_if = "Option::is_none")]
  pub experimental_decorators: Option<bool>,

  /// Print names of files which TypeScript sees as a part of your project and the reason they are part of the compilation. See more: https://www.typescriptlang.org/tsconfig/#explainFiles
  #[serde(skip_serializing_if = "Option::is_none")]
  pub explain_files: Option<bool>,

  /// Ensure that casing is correct in imports. See more: https://www.typescriptlang.org/tsconfig#forceConsistentCasingInFileNames
  #[serde(skip_serializing_if = "Option::is_none")]
  pub force_consistent_casing_in_file_names: Option<bool>,

  /// Generates an event trace and a list of types. See more: https://www.typescriptlang.org/tsconfig/#generateTrace
  #[serde(skip_serializing_if = "Option::is_none")]
  pub generate_trace: Option<bool>,

  /// Specify what JSX code is generated. See more: https://www.typescriptlang.org/tsconfig/#jsx
  #[serde(skip_serializing_if = "Option::is_none")]
  pub jsx: Option<Jsx>,

  /// Specify the JSX factory function used when targeting React JSX emit, e.g. 'React.createElement' or 'h'. See more: https://www.typescriptlang.org/tsconfig#jsxFactory
  #[serde(skip_serializing_if = "Option::is_none")]
  pub jsx_factory: Option<String>,

  /// Specify the JSX Fragment reference used for fragments when targeting React JSX emit e.g. 'React.Fragment' or 'Fragment'. See more: https://www.typescriptlang.org/tsconfig#jsxFragmentFactory
  #[serde(skip_serializing_if = "Option::is_none")]
  pub jsx_fragment_factory: Option<String>,

  /// Specify module specifier used to import the JSX factory functions when using `jsx: react-jsx`. See more: https://www.typescriptlang.org/tsconfig#jsxImportSource
  #[serde(skip_serializing_if = "Option::is_none")]
  pub jsx_import_source: Option<String>,

  /// Specify a set of bundled library declaration files that describe the target runtime environment. See more: https://www.typescriptlang.org/tsconfig#lib
  #[merge(strategy = merge_optional_btree_sets)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub lib: Option<BTreeSet<Lib>>,

  /// Enable lib replacement. See more: https://www.typescriptlang.org/tsconfig#libReplacement
  #[serde(skip_serializing_if = "Option::is_none")]
  pub lib_replacement: Option<bool>,

  /// Print names of generated files part of the compilation to the terminal. See more: https://www.typescriptlang.org/tsconfig/#listEmittedFiles
  #[serde(skip_serializing_if = "Option::is_none")]
  pub list_emitted_files: Option<bool>,

  /// Print all of the files read during the compilation.See more: https://www.typescriptlang.org/tsconfig#listFiles
  #[serde(skip_serializing_if = "Option::is_none")]
  pub list_files: Option<bool>,

  /// Specify the location where debugger should locate map files instead of generated locations. See more: https://www.typescriptlang.org/tsconfig#mapRoot
  #[serde(skip_serializing_if = "Option::is_none")]
  pub map_root: Option<String>,

  /// Specify the maximum folder depth used for checking JavaScript files from `node_modules`. Only applicable with `allowJs`. See more: https://www.typescriptlang.org/tsconfig#maxNodeModuleJsDepth
  #[serde(skip_serializing_if = "Option::is_none")]
  pub max_node_module_js_depth: Option<u32>,

  /// Specify what module code is generated. See more: https://www.typescriptlang.org/tsconfig#module
  #[serde(skip_serializing_if = "Option::is_none")]
  pub module: Option<Module>,

  /// Specify how TypeScript determine a file as module. See more: https://www.typescriptlang.org/tsconfig/#moduleDetection
  #[serde(skip_serializing_if = "Option::is_none")]
  pub module_detection: Option<ModuleDetection>,

  /// Provides a way to override the default list of file name suffixes to search when resolving a module. See more: https://www.typescriptlang.org/tsconfig/#moduleSuffixes
  #[merge(strategy = merge_optional_btree_sets)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub module_suffixes: Option<BTreeSet<String>>,

  /// Log paths used during the `moduleResolution` process. See more: https://www.typescriptlang.org/tsconfig#traceResolution
  #[serde(skip_serializing_if = "Option::is_none")]
  pub module_resolution: Option<ModuleResolution>,

  /// Set the newline character for emitting files. See more: https://www.typescriptlang.org/tsconfig#newLine
  #[serde(skip_serializing_if = "Option::is_none")]
  pub new_line: Option<NewLine>,

  /// Disable full type checking (only critical parse and emit errors will be reported). See more: https://www.typescriptlang.org/tsconfig#noCheck
  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_check: Option<bool>,

  /// Disable emitting file from a compilation. See more: https://www.typescriptlang.org/tsconfig#noEmit
  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_emit: Option<bool>,

  /// Enable error reporting for fallthrough cases in switch statements. See more: https://www.typescriptlang.org/tsconfig#noFallthroughCasesInSwitch
  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_fallthrough_cases_in_switch: Option<bool>,

  /// Disable generating custom helper functions like `__extends` in compiled output. See more: https://www.typescriptlang.org/tsconfig#noEmitHelpers
  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_emit_helpers: Option<bool>,

  /// Disable emitting files if any type checking errors are reported. See more: https://www.typescriptlang.org/tsconfig#noEmitOnError
  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_emit_on_error: Option<bool>,

  /// Disable truncating types in error messages. See more: https://www.typescriptlang.org/tsconfig#noErrorTruncation
  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_error_truncation: Option<bool>,

  /// Disable including any library files, including the default lib.d.ts. See more: https://www.typescriptlang.org/tsconfig#noLib
  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_lib: Option<bool>,

  /// Enable error reporting for expressions and declarations with an implied `any` type. See more: https://www.typescriptlang.org/tsconfig#noImplicitAny
  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_implicit_any: Option<bool>,

  /// Ensure overriding members in derived classes are marked with an override modifier. See more: https://www.typescriptlang.org/tsconfig#noImplicitOverride
  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_implicit_override: Option<bool>,

  /// Enable error reporting for codepaths that do not explicitly return in a function. See more: https://www.typescriptlang.org/tsconfig#noImplicitReturns
  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_implicit_returns: Option<bool>,

  /// Enable error reporting when `this` is given the type `any`. See more: https://www.typescriptlang.org/tsconfig#noImplicitThis
  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_implicit_this: Option<bool>,

  /// Disable adding 'use strict' directives in emitted JavaScript files. See more: https://www.typescriptlang.org/tsconfig#noImplicitUseStrict
  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_implicit_use_strict: Option<bool>,

  /// Enforces using indexed accessors for keys declared using an indexed type. See more: https://www.typescriptlang.org/tsconfig#noPropertyAccessFromIndexSignature
  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_property_access_from_index_signature: Option<bool>,

  /// Disallow `import`s, `require`s or `<reference>`s from expanding the number of files TypeScript should add to a project. See more: https://www.typescriptlang.org/tsconfig#noResolve
  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_resolve: Option<bool>,

  /// Disable strict checking of generic signatures in function types. See more: https://www.typescriptlang.org/tsconfig#noStrictGenericChecks
  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_strict_generic_checks: Option<bool>,

  /// Add `undefined` to a type when accessed using an index. See more: https://www.typescriptlang.org/tsconfig#noUncheckedIndexedAccess
  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_unchecked_indexed_access: Option<bool>,

  /// Check side effect imports. See more: https://www.typescriptlang.org/tsconfig#noUncheckedSideEffectImports
  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_unchecked_side_effects_imports: Option<bool>,

  /// Enable error reporting when a local variable isn't read. See more: https://www.typescriptlang.org/tsconfig#noUnusedLocals
  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_unused_locals: Option<bool>,

  /// Raise an error when a function parameter isn't read. See more: https://www.typescriptlang.org/tsconfig#noUnusedParameters
  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_unused_parameters: Option<bool>,

  /// Save .tsbuildinfo files to allow for incremental compilation of projects. See more: https://www.typescriptlang.org/tsconfig#incremental
  #[serde(skip_serializing_if = "Option::is_none")]
  pub incremental: Option<bool>,

  /// Include source code in the sourcemaps inside the emitted JavaScript. See more: https://www.typescriptlang.org/tsconfig#inlineSources
  #[serde(skip_serializing_if = "Option::is_none")]
  pub inline_sources: Option<bool>,

  /// Include sourcemap files inside the emitted JavaScript. See more: https://www.typescriptlang.org/tsconfig#inlineSourceMap
  #[serde(skip_serializing_if = "Option::is_none")]
  pub inline_source_map: Option<bool>,

  /// Allow importing helper functions from tslib once per project, instead of including them per-file. See more: https://www.typescriptlang.org/tsconfig#importHelpers
  #[serde(skip_serializing_if = "Option::is_none")]
  pub import_helpers: Option<bool>,

  /// Require sufficient annotation on exports so other tools can trivially generate declaration files. See more: https://www.typescriptlang.org/tsconfig#isolatedDeclarations
  #[serde(skip_serializing_if = "Option::is_none")]
  pub isolated_declarations: Option<bool>,

  /// Ensure that each file can be safely transpiled without relying on other imports. See more: https://www.typescriptlang.org/tsconfig#isolatedModules
  #[serde(skip_serializing_if = "Option::is_none")]
  pub isolated_modules: Option<bool>,

  /// Specify an output folder for all emitted files. See more: https://www.typescriptlang.org/tsconfig#outDir
  #[serde(skip_serializing_if = "Option::is_none")]
  pub out_dir: Option<String>,

  /// Specify a file that bundles all outputs into one JavaScript file. If `declaration` is true, also designates a file that bundles all .d.ts output. See more: https://www.typescriptlang.org/tsconfig#outFile
  #[serde(skip_serializing_if = "Option::is_none")]
  pub out_file: Option<String>,

  /// Specify a set of entries that re-map imports to additional lookup locations. See more: https://www.typescriptlang.org/tsconfig/#paths
  #[merge(strategy = merge_optional_btree_maps)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub paths: Option<BTreeMap<String, BTreeSet<String>>>,

  /// Specify a list of language service plugins to include. See more: https://www.typescriptlang.org/tsconfig#plugins
  #[merge(strategy = merge_optional_btree_sets)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub plugins: Option<BTreeSet<TsPlugin>>,

  /// Disable erasing `const enum` declarations in generated code. See more: https://www.typescriptlang.org/tsconfig#preserveConstEnums
  #[serde(skip_serializing_if = "Option::is_none")]
  pub preserve_const_enums: Option<bool>,

  /// Disable resolving symlinks to their realpath. This correlates to the same flag in node. See more: https://www.typescriptlang.org/tsconfig#preserveSymlinks
  #[serde(skip_serializing_if = "Option::is_none")]
  pub preserve_symlinks: Option<bool>,

  /// Disable wiping the console in watch mode. See more: https://www.typescriptlang.org/tsconfig#preserveWatchOutput
  #[serde(skip_serializing_if = "Option::is_none")]
  pub preserve_watch_output: Option<bool>,

  /// Enable color and formatting in output to make compiler errors easier to read. See more: https://www.typescriptlang.org/tsconfig#pretty
  #[serde(skip_serializing_if = "Option::is_none")]
  pub pretty: Option<bool>,

  /// Specify the object invoked for `createElement`. This only applies when targeting `react` JSX emit.See more: https://www.typescriptlang.org/tsconfig#reactNamespace
  #[serde(skip_serializing_if = "Option::is_none")]
  pub react_namespace: Option<String>,

  /// Disable emitting comments. See more: https://www.typescriptlang.org/tsconfig#removeComments
  #[serde(skip_serializing_if = "Option::is_none")]
  pub remove_comments: Option<bool>,

  /// Enable importing .json files. See more: https://www.typescriptlang.org/tsconfig#resolveJsonModule
  #[serde(skip_serializing_if = "Option::is_none")]
  pub resolve_json_module: Option<bool>,

  /// Specify the root folder within your source files. See more: https://www.typescriptlang.org/tsconfig#rootDir
  #[serde(skip_serializing_if = "Option::is_none")]
  pub root_dir: Option<String>,

  /// Specify the root folder within your source files.See more: https://www.typescriptlang.org/tsconfig#rootDir
  #[merge(strategy = merge_optional_btree_sets)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub root_dirs: Option<BTreeSet<String>>,

  /// Skip type checking .d.ts files that are included with TypeScript. See more: https://www.typescriptlang.org/tsconfig#skipDefaultLibCheck
  #[serde(skip_serializing_if = "Option::is_none")]
  pub skip_default_lib_check: Option<bool>,

  /// Skip type checking all .d.ts files. See more: https://www.typescriptlang.org/tsconfig#skipLibCheck
  #[serde(skip_serializing_if = "Option::is_none")]
  pub skip_lib_check: Option<bool>,

  /// Create source map files for emitted JavaScript files. See more: https://www.typescriptlang.org/tsconfig#sourceMap
  #[serde(skip_serializing_if = "Option::is_none")]
  pub source_map: Option<bool>,

  /// Specify the root path for debuggers to find the reference source code. See more: https://www.typescriptlang.org/tsconfig#sourceRoot
  #[serde(skip_serializing_if = "Option::is_none")]
  pub source_root: Option<String>,

  /// Enable all strict type checking options. See more: https://www.typescriptlang.org/tsconfig#strict
  #[serde(skip_serializing_if = "Option::is_none")]
  pub strict: Option<bool>,

  /// Check that the arguments for `bind`, `call`, and `apply` methods match the original function. See more: https://www.typescriptlang.org/tsconfig#strictBindCallApply
  #[serde(skip_serializing_if = "Option::is_none")]
  pub strict_bind_call_apply: Option<bool>,

  /// Built-in iterators are instantiated with a 'TReturn' type of 'undefined' instead of 'any'. See more: https://www.typescriptlang.org/tsconfig#strictBuiltinIteratorReturn
  #[serde(skip_serializing_if = "Option::is_none")]
  pub strict_builtin_iterator_return: Option<bool>,

  /// When assigning functions, check to ensure parameters and the return values are subtype-compatible. See more: https://www.typescriptlang.org/tsconfig#strictFunctionTypes
  #[serde(skip_serializing_if = "Option::is_none")]
  pub strict_function_types: Option<bool>,

  /// When type checking, take into account `null` and `undefined`. See more: https://www.typescriptlang.org/tsconfig#strictNullChecks
  #[serde(skip_serializing_if = "Option::is_none")]
  pub strict_null_checks: Option<bool>,

  /// Check for class properties that are declared but not set in the constructor.\n\nSee more: https://www.typescriptlang.org/tsconfig#strictPropertyInitialization
  #[serde(skip_serializing_if = "Option::is_none")]
  pub strict_property_initialization: Option<bool>,

  /// Disable emitting declarations that have `@internal` in their JSDoc comments. See more: https://www.typescriptlang.org/tsconfig#stripInternal
  #[serde(skip_serializing_if = "Option::is_none")]
  pub strip_internal: Option<bool>,

  /// Disable reporting of excess property errors during the creation of object literals. See more: https://www.typescriptlang.org/tsconfig#suppressExcessPropertyErrors
  #[serde(skip_serializing_if = "Option::is_none")]
  pub suppress_excess_property_errors: Option<bool>,

  /// Suppress `noImplicitAny` errors when indexing objects that lack index signatures. See more: https://www.typescriptlang.org/tsconfig#suppressImplicitAnyIndexErrors
  #[serde(skip_serializing_if = "Option::is_none")]
  pub suppress_implicit_any_index_errors: Option<bool>,

  /// Set the JavaScript language version for emitted JavaScript and include compatible library declarations. See more: https://www.typescriptlang.org/tsconfig#target
  #[serde(skip_serializing_if = "Option::is_none")]
  pub target: Option<Target>,

  /// Log paths used during the `moduleResolution` process. See more: https://www.typescriptlang.org/tsconfig#traceResolution
  #[serde(skip_serializing_if = "Option::is_none")]
  pub trace_resolution: Option<bool>,

  /// Specify the folder for .tsbuildinfo incremental compilation files. See more: https://www.typescriptlang.org/tsconfig#tsBuildInfoFile
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ts_build_info_file: Option<String>,

  /// Specify type package names to be included without being referenced in a source file. See more: https://www.typescriptlang.org/tsconfig#types
  #[merge(strategy = merge_optional_btree_sets)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub types: Option<BTreeSet<String>>,

  /// Specify multiple folders that act like `./node_modules/@types`. See more: https://www.typescriptlang.org/tsconfig#typeRoots
  #[merge(strategy = merge_optional_btree_sets)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub type_roots: Option<BTreeSet<String>>,

  /// Use the package.json 'exports' field when resolving package imports. See more: https://www.typescriptlang.org/tsconfig#resolvePackageJsonExports
  #[serde(skip_serializing_if = "Option::is_none")]
  pub resolve_package_json_exports: Option<bool>,

  /// Use the package.json 'imports' field when resolving imports. See more: https://www.typescriptlang.org/tsconfig#resolvePackageJsonImports
  #[serde(skip_serializing_if = "Option::is_none")]
  pub resolve_package_json_imports: Option<bool>,

  /// Rewrite '.ts', '.tsx', '.mts', and '.cts' file extensions in relative import paths to their JavaScript equivalent in output files. See more: https://www.typescriptlang.org/tsconfig#rewriteRelativeImportExtensions
  #[serde(skip_serializing_if = "Option::is_none")]
  pub rewrite_relative_imports_extensions: Option<bool>,

  /// Emit ECMAScript-standard-compliant class fields.See more: https://www.typescriptlang.org/tsconfig#useDefineForClassFields
  #[serde(skip_serializing_if = "Option::is_none")]
  pub use_define_for_class_fields: Option<bool>,

  /// Default catch clause variables as `unknown` instead of `any`. See more: https://www.typescriptlang.org/tsconfig#useUnknownInCatchVariables
  #[serde(skip_serializing_if = "Option::is_none")]
  pub use_unknown_in_catch_variables: Option<bool>,

  /// Do not transform or elide any imports or exports not marked as type-only, ensuring they are written in the output file's format based on the 'module' setting. See more: https://www.typescriptlang.org/tsconfig#verbatimModuleSyntax
  #[serde(skip_serializing_if = "Option::is_none")]
  pub verbatim_module_syntax: Option<bool>,
}
