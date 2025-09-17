use maplit::btreeset;

use super::{
  CompilerOptions, Lib, Module, ModuleDetection, ModuleResolution, Target, TsConfig,
  TsConfigReference,
};

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
      no_unchecked_side_effects_imports: Some(true),
      ..Default::default()
    }),
    ..Default::default()
  }
}

pub(crate) fn get_default_package_tsconfig(rel_path_to_root_dir: String, is_app: bool) -> TsConfig {
  TsConfig {
    extends: Some(rel_path_to_root_dir),
    files: Some(btreeset![]),
    references: {
      let mut references = btreeset![TsConfigReference {
        path: "tsconfig.src.json".to_string(),
      }];
      if !is_app {
        references.insert(TsConfigReference {
          path: "tsconfig.dev.json".to_string(),
        });
      }
      Some(references)
    },
    ..Default::default()
  }
}

pub(crate) fn get_default_dev_tsconfig(out_dir: &str) -> TsConfig {
  TsConfig {
    extends: Some("tsconfig.src.json".to_string()),
    include: Some(btreeset![
      "*.ts".to_string(),
      "tests".to_string(),
      "scripts".to_string(),
      "src".to_string(),
    ]),
    references: Some(btreeset![TsConfigReference {
      path: "tsconfig.src.json".to_string(),
    }]),
    compiler_options: Some(CompilerOptions {
      root_dir: Some(".".to_string()),
      no_emit: Some(true),
      ts_build_info_file: Some(format!("{}/.tsBuildInfoDev", out_dir)),
      ..Default::default()
    }),
    ..Default::default()
  }
}

pub(crate) fn get_default_src_tsconfig(is_app: bool, out_dir: &str) -> TsConfig {
  TsConfig {
    extends: Some("./tsconfig.json".to_string()),
    references: Some(btreeset![]),
    include: if is_app {
      Some(btreeset![
        "src".to_string(),
        "*.ts".to_string(),
        "tests".to_string(),
        "scripts".to_string(),
      ])
    } else {
      Some(btreeset!["src".to_string()])
    },
    compiler_options: Some(CompilerOptions {
      root_dir: Some("src".to_string()),
      out_dir: Some(out_dir.to_string()),
      ts_build_info_file: Some(format!("{}/.tsBuildInfoSrc", out_dir)),
      no_emit: is_app.then_some(true),
      emit_declaration_only: (!is_app).then_some(true),
      ..Default::default()
    }),
    ..Default::default()
  }
}
