use std::fs::{File, create_dir_all};
use std::path::PathBuf;

use maplit::{btreemap, btreeset};
use pretty_assertions::assert_eq;
use serde_json::Value;

use typescript_config::*;

#[test]
fn tsconfig_generation() -> Result<(), Box<dyn std::error::Error>> {
	let ts_config = TsConfig {
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
			root_dirs: btreeset!["abc".to_string(), "abc".to_string()],
			types: btreeset!["abc".to_string(), "abc".to_string()],
			type_roots: btreeset!["abc".to_string(), "abc".to_string()],
			lib: btreeset![Lib::Dom, Lib::EsNext],
			paths: btreemap! { "@".to_string() => btreeset!["src/".to_string()], "@components".to_string() => btreeset!["src/components".to_string()] },
			verbatim_module_syntax: Some(true),
			new_line: Some(NewLine::Lf),
			rewrite_relative_imports_extensions: Some(true),
			resolve_package_json_imports: Some(true),
			resolve_package_json_exports: Some(true),
			no_unchecked_side_effect_imports: Some(true),
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
			module_suffixes: btreeset!["abc".to_string(), "abc".to_string()],
			custom_conditions: btreeset!["abc".to_string(), "abc".to_string()],
			plugins: btreeset! {
			  TsPlugin {
				name: "typescript-svelte-plugin".to_string(),
				extras: btreemap! {
				  "enabled".to_string() => Value::Bool(true), "assumeIsSvelteProject".to_string() => Value::Bool(true)
				}
			  }
			},
		}),
		extends: Some("tsconfig.options.json".to_string()),
		files: btreeset!["*.ts".to_string(), "*.js".to_string()],
		include: btreeset!["*.ts".to_string(), "*.js".to_string()],
		exclude: btreeset!["*.ts".to_string(), "*.js".to_string()],
		references: btreeset![
			TsConfigReference {
				path: "abc.json".to_string(),
			},
			TsConfigReference {
				path: "abc.json".to_string(),
			},
		],
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
			exclude_directories: btreeset!["abc".to_string(), "abc".to_string()],
			exclude_files: btreeset!["abc".to_string(), "abc".to_string()],
		}),
	};

	let output_path = PathBuf::from("tests/output/tsconfig.json");

	create_dir_all(output_path.parent().unwrap()).unwrap();

	serde_json::to_writer_pretty(File::create(&output_path)?, &ts_config)?;

	let result: TsConfig = serde_json::from_reader(File::open(&output_path)?)
		.expect("Error in TsConfig deserialization in test");

	assert_eq!(result, ts_config);

	Ok(())
}
