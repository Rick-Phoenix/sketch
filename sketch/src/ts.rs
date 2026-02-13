pub mod oxlint;
pub mod package;
pub mod package_json;
pub mod pnpm;
pub mod ts_config;
pub mod vitest;

use regex::Regex;

use crate::{
	ts::{
		oxlint::*,
		package::TsPackagePreset,
		package_json::*,
		pnpm::{PnpmPreset, PnpmWorkspace},
		ts_config::*,
		vitest::VitestPreset,
	},
	versions::*,
	*,
};

/// All settings related to typescript projects.
#[derive(Clone, Debug, Deserialize, Serialize, Merge, Parser, PartialEq, Eq, Default)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default)]
pub struct TypescriptConfig {
	/// The package manager being used. [default: pnpm].
	#[arg(value_enum, long, value_name = "NAME")]
	pub package_manager: Option<PackageManager>,

	/// Do not add default dependencies to new `package.json` files (typescript and oxlint, plus vitest if enabled)
	#[arg(long)]
	pub no_default_deps: Option<bool>,

	/// The kind of version range to use for dependencies that are fetched automatically. [default: major]
	#[arg(value_enum)]
	#[arg(long, value_name = "KIND")]
	pub version_range: Option<VersionRange>,

	/// Uses the default catalog (supported by pnpm and bun) for default dependencies, and automatically adds dependencies marked with `catalog:` to the catalog, if they are missing.
	#[arg(long)]
	pub catalog: Option<bool>,

	/// Do not convert dependencies marked as `latest` to a version range.
	#[arg(long = "no-convert-latest")]
	pub no_convert_latest_to_range: Option<bool>,

	/// A map containing [`PackageJsonPreset`]s.
	#[arg(skip)]
	pub package_json_presets: IndexMap<String, PackageJsonPreset>,

	/// A map containing [`TsConfigPreset`]s.
	#[arg(skip)]
	pub ts_config_presets: IndexMap<String, TsConfigPreset>,

	/// A map containing [`OxlintPreset`]s.
	#[arg(skip)]
	pub oxlint_presets: IndexMap<String, OxlintPreset>,

	/// A map of [`PackageConfig`] presets.
	#[arg(skip)]
	pub package_presets: IndexMap<String, TsPackagePreset>,

	/// A map of presets for `pnpm-workspace.yaml` configurations.
	#[arg(skip)]
	pub pnpm_presets: IndexMap<String, PnpmPreset>,

	/// A map of presets for vitest setups.
	#[arg(skip)]
	pub vitest_presets: IndexMap<String, VitestPreset>,
}

impl TypescriptConfig {
	pub fn get_vitest_preset(&self, id: &str) -> AppResult<VitestPreset> {
		Ok(self
			.vitest_presets
			.get(id)
			.ok_or_else(|| AppError::PresetNotFound {
				kind: PresetKind::Vitest,
				name: id.to_string(),
			})?
			.clone())
	}

	pub fn get_tsconfig_preset(&self, id: &str) -> AppResult<TsConfigPreset> {
		self.ts_config_presets
			.get(id)
			.ok_or_else(|| AppError::PresetNotFound {
				kind: PresetKind::TsConfig,
				name: id.to_string(),
			})?
			.clone()
			.merge_presets(id, &self.ts_config_presets)
	}

	pub fn get_package_preset(&self, id: &str) -> AppResult<TsPackagePreset> {
		Ok(self
			.package_presets
			.get(id)
			.ok_or_else(|| AppError::PresetNotFound {
				kind: PresetKind::TsPackage,
				name: id.to_string(),
			})?
			.clone())
	}

	pub fn get_pnpm_preset(&self, id: &str) -> AppResult<PnpmPreset> {
		self.pnpm_presets
			.get(id)
			.ok_or_else(|| AppError::PresetNotFound {
				kind: PresetKind::PnpmWorkspace,
				name: id.to_string(),
			})?
			.clone()
			.merge_presets(id, &self.pnpm_presets)
	}

	pub fn get_oxlint_preset(&self, id: &str) -> AppResult<OxlintPreset> {
		self.oxlint_presets
			.get(id)
			.ok_or_else(|| AppError::PresetNotFound {
				kind: PresetKind::Oxlint,
				name: id.to_string(),
			})?
			.clone()
			.merge_presets(id, &self.oxlint_presets)
	}

	pub fn get_package_json(&self, id: &str) -> AppResult<PackageJsonPreset> {
		self.package_json_presets
			.get(id)
			.ok_or_else(|| AppError::PresetNotFound {
				kind: PresetKind::PackageJson,
				name: id.to_string(),
			})?
			.clone()
			.merge_presets(id, &self.package_json_presets)
	}
}

impl PackageManager {
	pub fn find_root(&self, start_dir: &Path) -> Option<PathBuf> {
		let root_marker = self.root_marker();

		if let Some(file) = find_file_up(start_dir, root_marker) {
			Some(get_parent_dir(&file).ok()?.to_path_buf())
		} else {
			None
		}
	}

	pub const fn root_marker(&self) -> &str {
		match self {
			Self::Pnpm => "pnpm-workspace.yaml",
			Self::Npm => "package-lock.json",
			Self::Deno => "deno.lock",
			Self::Bun => "bun.lock",
			Self::Yarn => "yarn.lock",
		}
	}

	/// Returns `true` if the package manager is [`Bun`].
	///
	/// [`Bun`]: PackageManager::Bun
	#[must_use]
	pub const fn is_bun(&self) -> bool {
		matches!(self, Self::Bun)
	}

	/// Returns `true` if the package manager is [`Pnpm`].
	///
	/// [`Pnpm`]: PackageManager::Pnpm
	#[must_use]
	pub const fn is_pnpm(&self) -> bool {
		matches!(self, Self::Pnpm)
	}
}

/// A js/ts package manager.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default, ValueEnum, Copy)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum PackageManager {
	#[default]
	Pnpm,
	Npm,
	Deno,
	Bun,
	Yarn,
}

impl Display for PackageManager {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Pnpm => {
				write!(f, "pnpm")
			}
			Self::Npm => {
				write!(f, "npm")
			}
			Self::Deno => {
				write!(f, "deno")
			}
			Self::Bun => {
				write!(f, "bun")
			}
			Self::Yarn => {
				write!(f, "yarn")
			}
		}
	}
}

#[cfg(feature = "npm-version")]
pub(crate) static CATALOG_REGEX: LazyLock<Regex> = LazyLock::new(|| {
	Regex::new(r"^catalog:(?<name>\w+)?$").expect("Failed to initialize the catalog regex")
});
