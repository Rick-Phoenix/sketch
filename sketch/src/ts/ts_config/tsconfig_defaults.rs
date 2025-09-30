use maplit::btreeset;
use merge::Merge;

use super::{CompilerOptions, Lib, Module, ModuleDetection, ModuleResolution, Target, TsConfig};

pub(crate) fn get_default_root_tsconfig() -> TsConfig {
  TsConfig {
    compiler_options: Some(CompilerOptions {
      lib: Some(btreeset![Lib::EsNext, Lib::Dom]),
      module_resolution: Some(ModuleResolution::NodeNext),
      module: Some(Module::NodeNext),
      target: Some(Target::EsNext),
      module_detection: Some(ModuleDetection::Force),
      isolated_modules: Some(true),
      es_module_interop: Some(true),
      resolve_json_module: Some(true),
      declaration: Some(true),
      declaration_map: Some(true),
      composite: Some(true),
      no_emit_on_error: Some(true),
      incremental: Some(true),
      source_map: Some(true),
      strict: Some(true),
      strict_null_checks: Some(true),
      skip_lib_check: Some(true),
      force_consistent_casing_in_file_names: Some(true),
      no_unchecked_indexed_access: Some(true),
      allow_synthetic_default_imports: Some(true),
      verbatim_module_syntax: Some(true),
      no_unchecked_side_effect_imports: Some(true),
      ..Default::default()
    }),
    ..Default::default()
  }
}

pub(crate) fn get_default_package_tsconfig() -> TsConfig {
  let mut base = get_default_root_tsconfig();

  base.merge(TsConfig {
    extends: None,
    references: Some(btreeset![]),
    include: Some(btreeset![
      "src".to_string(),
      "*.ts".to_string(),
      "tests".to_string(),
      "scripts".to_string(),
    ]),
    compiler_options: Some(CompilerOptions {
      root_dir: Some("src".to_string()),
      out_dir: Some(".out".to_string()),
      ts_build_info_file: Some(format!(".out/.tsBuildInfoSrc")),
      emit_declaration_only: Some(true),
      ..Default::default()
    }),
    ..Default::default()
  });

  base
}
