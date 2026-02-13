use super::*;
use crate::cli::parsers::parse_key_value_pairs;
pub(crate) use ::ts_config::*;

/// A preset for a `tsconfig` file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, Merge)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default)]
pub struct TsConfigPreset {
	/// The list of extended presets.
	#[merge(skip)]
	pub extends_presets: IndexSet<String>,

	#[serde(flatten)]
	pub config: TsConfig,
}

impl ExtensiblePreset for TsConfigPreset {
	fn kind() -> PresetKind {
		PresetKind::TsConfig
	}

	fn extended_ids(&mut self) -> &mut IndexSet<String> {
		&mut self.extends_presets
	}
}

/// The kind of data for a [`TsConfig`]. It can be a string indicating a preset it, or a full configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
pub enum TsConfigPresetRef {
	Id(String),
	Config(TsConfigPreset),
}

impl Default for TsConfigPresetRef {
	fn default() -> Self {
		Self::Config(TsConfigPreset::default())
	}
}

/// A struct representing instructions for generating a tsconfig file.
/// If the output path is relative, it will be joined to the root path of its package.
#[derive(Deserialize, Debug, Clone, Serialize, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct TsConfigData {
	/// The output path of the config file [default: `tsconfig.json`]
	pub output: Option<String>,

	/// The configuration for the output file. It can be a preset id or a new definition.
	pub config: Option<TsConfigPresetRef>,
}

impl Default for TsConfigData {
	fn default() -> Self {
		Self {
			output: Some("tsconfig.json".to_string()),
			config: Some(TsConfigPresetRef::default()),
		}
	}
}

impl TsConfigData {
	pub(crate) fn from_cli(s: &str) -> Result<Self, String> {
		let mut directive: Self = Default::default();

		let pairs = parse_key_value_pairs("TsConfigData", s)?;

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
						Some(TsConfigPresetRef::Id(val.to_string()))
					}
				}
				_ => return Err(format!("Invalid key for TsConfigData: {key}")),
			};
		}

		Ok(directive)
	}
}

pub(crate) fn get_default_root_tsconfig() -> TsConfig {
	TsConfig {
		compiler_options: Some(CompilerOptions {
			lib: btreeset![Lib::EsNext, Lib::Dom],
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
		references: btreeset![],
		include: btreeset![
			"src".to_string(),
			"*.ts".to_string(),
			"tests".to_string(),
			"scripts".to_string(),
		],
		compiler_options: Some(CompilerOptions {
			out_dir: Some(".out".to_string()),
			ts_build_info_file: Some(".out/.tsBuildInfoSrc".to_string()),
			emit_declaration_only: Some(true),
			..Default::default()
		}),
		..Default::default()
	});

	base
}
