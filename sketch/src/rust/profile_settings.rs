use super::*;
use crate::*;

/// Handling of LTO in a build profile
#[derive(Debug, Clone, PartialEq, Serialize, Eq, PartialOrd, Ord, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum LtoSetting {
	/// off
	None,
	/// false
	ThinLocal,
	Thin,
	/// True
	Fat,
}

impl AsTomlValue for LtoSetting {
	fn as_toml_value(&self) -> Item {
		match self {
			Self::None => "off".into(),
			Self::ThinLocal => false.into(),
			Self::Thin => "thin".into(),
			Self::Fat => true.into(),
		}
	}
}

/// Verbosity of debug info in a [`Profile`]
#[derive(
	Debug,
	Copy,
	Clone,
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
	Serialize_repr,
	Deserialize_repr,
	JsonSchema_repr,
)]
#[repr(u8)]
pub enum DebugSetting {
	/// 0 or false
	None = 0,
	/// 1 = line tables only
	Lines = 1,
	/// 2 or true
	Full = 2,
}

impl AsTomlValue for DebugSetting {
	fn as_toml_value(&self) -> Item {
		(*self as u8 as i64).into()
	}
}

/// Handling of debug symbols in a build profile
#[derive(
	Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, JsonSchema, Serialize,
)]
#[serde(rename_all = "kebab-case")]
pub enum StripSetting {
	/// Same as `strip = false`
	None,
	/// Detailed debug is stripped, but coarse debug is preserved
	Debuginfo,
	/// Stronger than the `Debuginfo` setting, same as `strip = true`
	Symbols,
}

impl AsTomlValue for StripSetting {
	fn as_toml_value(&self) -> Item {
		let str = match self {
			StripSetting::None => "none",
			StripSetting::Debuginfo => "debuginfo",
			StripSetting::Symbols => "symbols",
		};

		str.into()
	}
}

/// Compilation/optimization settings for a workspace
#[derive(Debug, Clone, PartialEq, Default, JsonSchema, Serialize, Deserialize, Merge)]
#[merge(strategy = overwrite_if_some)]
#[serde(rename_all = "kebab-case")]
pub struct Profile {
	/// num or z, s
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub opt_level: Option<Value>,

	/// 0,1,2 or bool
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub debug: Option<DebugSetting>,

	/// Move debug info to separate files
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub split_debuginfo: Option<String>,

	/// For dynamic libraries
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub rpath: Option<bool>,

	/// Link-time-optimization
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub lto: Option<LtoSetting>,

	/// Extra assertions
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub debug_assertions: Option<bool>,

	/// Parallel compilation
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub codegen_units: Option<u16>,

	/// Handling of panics/unwinding
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub panic: Option<String>,

	/// Support for incremental rebuilds
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub incremental: Option<bool>,

	/// Check integer arithmetic
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub overflow_checks: Option<bool>,

	/// Remove debug info
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub strip: Option<StripSetting>,

	/// Profile overrides for dependencies, `*` is special.
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	#[merge(strategy = merge_btree_maps)]
	pub package: BTreeMap<String, Self>,

	/// Profile overrides for build dependencies, `*` is special.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub build_override: Option<Value>,

	/// Only relevant for non-standard profiles
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub inherits: Option<String>,
}

impl AsTomlValue for Profile {
	fn as_toml_value(&self) -> Item {
		let mut table = Table::new();

		add_value!(self, table => debug, lto, strip);
		add_string!(self, table => split_debuginfo, panic, inherits);
		add_map!(self, table => package);

		add_optional_bool!(self, table => rpath, debug_assertions, incremental, overflow_checks);

		if let Some(codegen_units) = self.codegen_units {
			table["codegen-units"] = i64::from(codegen_units).into();
		}

		table.into()
	}
}

/// Build-in an custom build/optimization settings
#[derive(Debug, Clone, PartialEq, Default, JsonSchema, Serialize, Deserialize, Merge)]
#[merge(strategy = merge_optional_nested)]
pub struct Profiles {
	/// Used for `--release`
	#[serde(skip_serializing_if = "Option::is_none")]
	pub release: Option<Profile>,

	/// Used by default, weirdly called `debug` profile.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub dev: Option<Profile>,

	/// Used for `cargo test`
	#[serde(skip_serializing_if = "Option::is_none")]
	pub test: Option<Profile>,

	/// Used for `cargo bench` (nightly)
	#[serde(skip_serializing_if = "Option::is_none")]
	pub bench: Option<Profile>,

	/// User-suppiled for `cargo --profile=name`
	#[serde(flatten)]
	#[merge(strategy = merge_btree_maps)]
	pub custom: BTreeMap<String, Profile>,
}

impl AsTomlValue for Profiles {
	fn as_toml_value(&self) -> Item {
		let mut table = Table::new();

		add_value!(self, table => release, dev, test, bench);
		add_map!(self, table => custom);

		table.into()
	}
}
