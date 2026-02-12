use super::*;

/// Handling of LTO in a build profile
#[derive(Debug, Clone, PartialEq, Serialize, Eq, PartialOrd, Ord, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(try_from = "RawLto", into = "RawLto")]
#[cfg_attr(feature = "schemars", schemars(with = "RawLto"))]
#[serde(rename_all = "kebab-case")]
pub enum LtoSetting {
	None,
	ThinLocal,
	Thin,
	Fat,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
enum RawLto {
	Bool(bool),
	String(String),
}

impl TryFrom<RawLto> for LtoSetting {
	type Error = String;

	fn try_from(value: RawLto) -> Result<Self, Self::Error> {
		match value {
			RawLto::Bool(false) => Ok(Self::ThinLocal),
			RawLto::Bool(true) => Ok(Self::Fat),

			RawLto::String(s) => match s.as_str() {
				"off" => Ok(Self::None),
				"thin-local" => Ok(Self::ThinLocal),
				"thin" => Ok(Self::Thin),
				"fat" => Ok(Self::Fat),
				_ => Err(format!("Unknown LTO setting: {s}")),
			},
		}
	}
}

impl From<LtoSetting> for RawLto {
	fn from(setting: LtoSetting) -> Self {
		match setting {
			LtoSetting::None => Self::String("off".to_string()),
			LtoSetting::ThinLocal => Self::Bool(false),
			LtoSetting::Thin => Self::String("thin".to_string()),
			LtoSetting::Fat => Self::Bool(true),
		}
	}
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "RawDebugSetting", into = "RawDebugSetting")]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "schemars", schemars(with = "RawDebugSetting"))]
pub enum DebugSetting {
	None,               // 0, false, "none"
	LineDirectivesOnly, // "line-directives-only"
	LineTablesOnly,     // "line-tables-only"
	Limited,            // 1, "limited"
	Full,               // 2, true, "full"
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
enum RawDebugSetting {
	Bool(bool),
	Integer(u8),
	String(String),
}

impl TryFrom<RawDebugSetting> for DebugSetting {
	type Error = String;

	fn try_from(value: RawDebugSetting) -> Result<Self, Self::Error> {
		match value {
			RawDebugSetting::Bool(false) | RawDebugSetting::Integer(0) => Ok(Self::None),
			RawDebugSetting::Integer(1) => Ok(Self::Limited),
			RawDebugSetting::Bool(true) | RawDebugSetting::Integer(2) => Ok(Self::Full),
			RawDebugSetting::Integer(n) => {
				Err(format!("Invalid debug level: {n}. Use 0, 1, or 2."))
			}

			RawDebugSetting::String(s) => match s.as_str() {
				"none" | "false" | "0" => Ok(Self::None),

				"line-directives-only" => Ok(Self::LineDirectivesOnly),
				"line-tables-only" => Ok(Self::LineTablesOnly),

				"limited" | "1" => Ok(Self::Limited),
				"full" | "true" | "2" => Ok(Self::Full),
				_ => Err(format!("Unknown debug setting: '{s}'")),
			},
		}
	}
}

impl From<DebugSetting> for RawDebugSetting {
	fn from(setting: DebugSetting) -> Self {
		match setting {
			DebugSetting::None => Self::Bool(false),
			DebugSetting::Full => Self::Bool(true),

			DebugSetting::Limited => Self::Integer(1),

			DebugSetting::LineDirectivesOnly => Self::String("line-directives-only".to_string()),
			DebugSetting::LineTablesOnly => Self::String("line-tables-only".to_string()),
		}
	}
}

impl AsTomlValue for DebugSetting {
	fn as_toml_value(&self) -> Item {
		let raw: RawDebugSetting = (*self).into();

		match raw {
			RawDebugSetting::Bool(bool) => bool.into(),
			RawDebugSetting::Integer(int) => (i64::from(int)).into(),
			RawDebugSetting::String(str) => str.into(),
		}
	}
}

/// Handling of debug symbols in a build profile
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "RawStripSetting", into = "RawStripSetting")]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "schemars", schemars(with = "RawStripSetting"))]
pub enum StripSetting {
	None,
	Debuginfo,
	Symbols,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
enum RawStripSetting {
	Bool(bool),
	String(String),
}

impl TryFrom<RawStripSetting> for StripSetting {
	type Error = String;

	fn try_from(value: RawStripSetting) -> Result<Self, Self::Error> {
		match value {
			RawStripSetting::Bool(false) => Ok(Self::None),
			RawStripSetting::Bool(true) => Ok(Self::Symbols),

			RawStripSetting::String(s) => match s.as_str() {
				"debuginfo" => Ok(Self::Debuginfo),
				"symbols" | "true" => Ok(Self::Symbols),

				"none" | "false" => Ok(Self::None),
				_ => Err(format!(
					"Invalid strip setting: '{s}'. Use 'none', 'debuginfo', 'symbols', true, or false."
				)),
			},
		}
	}
}

impl From<StripSetting> for RawStripSetting {
	fn from(setting: StripSetting) -> Self {
		match setting {
			StripSetting::None => Self::String("none".to_string()),
			StripSetting::Debuginfo => Self::String("debuginfo".to_string()),
			StripSetting::Symbols => Self::String("symbols".to_string()),
		}
	}
}

impl AsTomlValue for StripSetting {
	fn as_toml_value(&self) -> Item {
		let raw: RawStripSetting = (*self).into();

		match raw {
			RawStripSetting::Bool(bool) => bool.into(),
			RawStripSetting::String(str) => str.into(),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(try_from = "RawOptLevel", into = "RawOptLevel")]
pub enum OptLevel {
	Zero,
	One,
	Two,
	Three,
	S,
	Z,
}

impl AsTomlValue for OptLevel {
	fn as_toml_value(&self) -> Item {
		match self {
			Self::Zero => Item::from(0),
			Self::One => Item::from(1),
			Self::Two => Item::from(2),
			Self::Three => Item::from(3),
			Self::S => Item::from("s"),
			Self::Z => Item::from("z"),
		}
	}
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
enum RawOptLevel {
	Integer(u8),
	String(String),
}

impl TryFrom<RawOptLevel> for OptLevel {
	type Error = String;

	fn try_from(value: RawOptLevel) -> Result<Self, Self::Error> {
		match value {
			RawOptLevel::Integer(0) => Ok(Self::Zero),
			RawOptLevel::Integer(1) => Ok(Self::One),
			RawOptLevel::Integer(2) => Ok(Self::Two),
			RawOptLevel::Integer(3) => Ok(Self::Three),

			RawOptLevel::String(s) => match s.as_str() {
				"0" => Ok(Self::Zero),
				"1" => Ok(Self::One),
				"2" => Ok(Self::Two),
				"3" => Ok(Self::Three),
				"s" => Ok(Self::S),
				"z" => Ok(Self::Z),
				_ => Err(format!("Invalid opt-level: {s}")),
			},

			_ => Err("opt-level must be 0-3, 's', or 'z'".to_string()),
		}
	}
}

impl From<OptLevel> for RawOptLevel {
	fn from(val: OptLevel) -> Self {
		match val {
			OptLevel::Zero => Self::Integer(0),
			OptLevel::One => Self::Integer(1),
			OptLevel::Two => Self::Integer(2),
			OptLevel::Three => Self::Integer(3),
			OptLevel::S => Self::String("s".to_string()),
			OptLevel::Z => Self::String("z".to_string()),
		}
	}
}

/// Compilation/optimization settings for a workspace
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Merge)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct Profile {
	/// The opt-level setting controls the `-C opt-level` flag which controls the level of optimization.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub opt_level: Option<OptLevel>,

	/// The debug setting controls the `-C debuginfo` flag which controls the amount of debug information included in the compiled binary.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub debug: Option<DebugSetting>,

	/// The split-debuginfo setting controls the `-C split-debuginfo` flag which controls whether debug information, if generated, is either placed in the executable itself or adjacent to it.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub split_debuginfo: Option<String>,

	/// The rpath setting controls the `-C rpath` flag which controls whether or not rpath is enabled.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub rpath: Option<bool>,

	/// The lto setting controls rustc’s `-C lto`, `-C linker-plugin-lto`, and `-C embed-bitcode` options, which control LLVM’s link time optimizations.
	///
	/// LTO can produce better optimized code, using whole-program analysis, at the cost of longer linking time.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub lto: Option<LtoSetting>,

	/// The debug-assertions setting controls the `-C debug-assertions` flag which turns `cfg(debug_assertions)` conditional compilation on or off.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub debug_assertions: Option<bool>,

	/// The codegen-units setting controls the `-C codegen-units` flag which controls how many “code generation units” a crate will be split into.
	///
	/// More code generation units allows more of a crate to be processed in parallel possibly reducing compile time, but may produce slower code.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub codegen_units: Option<u16>,

	/// The panic setting controls the `-C panic` flag which controls which panic strategy to use.
	///
	/// The valid options are:
	/// - `unwind`: Unwind the stack upon panic.
	/// - `abort`: Terminate the process upon panic.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub panic: Option<String>,

	/// The incremental setting controls the `-C incremental` flag which controls whether or not incremental compilation is enabled.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub incremental: Option<bool>,

	/// The overflow-checks setting controls the `-C overflow-checks` flag which controls the behavior of runtime integer overflow.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub overflow_checks: Option<bool>,

	/// The strip option controls the `-C strip` flag, which directs rustc to strip either symbols or debuginfo from a binary.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub strip: Option<StripSetting>,

	/// Profile settings can be overridden for specific packages and build-time crates.
	///
	/// To override the settings for a specific package, use the package table to change the settings for the named package:
	/// ```toml
	/// # The `foo` package will use the -Copt-level=3 flag.
	/// [profile.dev.package.foo]
	/// opt-level = 3
	/// ```
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	pub package: BTreeMap<String, Self>,

	/// To override the settings for build scripts, proc macros, and their dependencies, use the build-override table:
	/// ```toml
	/// # Set the settings for build scripts and proc-macros.
	/// [profile.dev.build-override]
	/// opt-level = 3
	/// ```
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub build_override: Option<Box<Self>>,

	/// Specifies which profile the custom profile inherits settings from when the setting is not specified.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub inherits: Option<String>,
}

impl AsTomlValue for Profile {
	fn as_toml_value(&self) -> Item {
		let mut table = Table::new();

		table.set_implicit(true);

		add_value!(self, table => debug, lto, strip, opt_level, build_override);
		add_string!(self, table => split_debuginfo, panic, inherits);

		add_optional_bool!(self, table => rpath, debug_assertions, incremental, overflow_checks);

		if let Some(codegen_units) = self.codegen_units {
			table["codegen-units"] = i64::from(codegen_units).into();
		}

		if !self.package.is_empty() {
			let mut pkg_table = Table::from_iter(
				self.package
					.iter()
					.map(|(k, v)| (toml_edit::Key::from(k), v.as_toml_value())),
			);

			pkg_table.set_implicit(true);

			table.insert("package", pkg_table.into());
		};

		table.into()
	}
}

/// Custom build/optimization settings
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Merge)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[merge(with = merge_options)]
pub struct Profiles {
	/// Used for `--release`
	#[serde(skip_serializing_if = "Option::is_none")]
	pub release: Option<Profile>,

	/// Used by default
	#[serde(skip_serializing_if = "Option::is_none")]
	pub dev: Option<Profile>,

	/// Used for `cargo test`
	#[serde(skip_serializing_if = "Option::is_none")]
	pub test: Option<Profile>,

	/// Used for `cargo bench`
	#[serde(skip_serializing_if = "Option::is_none")]
	pub bench: Option<Profile>,

	/// User-supplied for `cargo --profile=name`
	#[serde(flatten)]
	#[merge(with = BTreeMap::extend)]
	pub custom: BTreeMap<String, Profile>,
}

impl AsTomlValue for Profiles {
	fn as_toml_value(&self) -> Item {
		let mut table = Table::new();

		table.set_implicit(true);

		add_value!(self, table => dev, test, bench, release);
		add_table!(self, table => custom);

		table.into()
	}
}
