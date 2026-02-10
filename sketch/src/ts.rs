pub mod oxlint;
pub mod package;
pub mod package_json;
pub mod pnpm;
pub mod ts_config;
pub mod ts_monorepo;
pub mod vitest;

use askama::Template;
use regex::Regex;

use crate::{
	ts::{
		oxlint::*,
		package::PackageConfig,
		package_json::*,
		pnpm::{PnpmPreset, PnpmWorkspace},
		ts_config::*,
		tsconfig_defaults::*,
		vitest::VitestConfig,
	},
	versions::*,
	*,
};

impl TypescriptConfig {
	pub fn get_contributor(&self, name: &str) -> Option<Person> {
		self.people
			.get(name)
			.map(|person| Person::Data(person.clone()))
	}
}

/// All settings related to typescript projects.
#[derive(
	Clone, Debug, Deserialize, Serialize, Merge, Parser, PartialEq, Eq, JsonSchema, Default,
)]
#[merge(strategy = overwrite_if_some)]
#[serde(default)]
pub struct TypescriptConfig {
	/// The package manager being used. [default: pnpm].
	#[arg(value_enum, long, value_name = "NAME")]
	pub package_manager: Option<PackageManager>,

	/// Do not add default dependencies to new `package.json` files (typescript and oxlint, plus vitest if enabled)
	#[arg(long)]
	pub no_default_deps: Option<bool>,

	/// The kind of version range to use for dependencies that are fetched automatically. [default: minor]
	#[arg(value_enum)]
	#[arg(long, value_name = "KIND")]
	pub version_range: Option<VersionRange>,

	/// Uses the default catalog (supported by pnpm and bun) for default dependencies, and automatically adds dependencies marked with `catalog:` to the catalog, if they are missing.
	#[arg(long)]
	pub catalog: Option<bool>,

	/// Do not convert dependencies marked as `latest` to a version range.
	#[arg(long = "no-convert-latest")]
	pub no_convert_latest_to_range: Option<bool>,

	/// A map of individual [`PersonData`] that can be referenced as authors, contributors or maintainers in a [`PackageJsonPreset`].
	#[arg(skip)]
	#[merge(strategy = IndexMap::extend)]
	pub people: IndexMap<String, PersonData>,

	/// A map containing [`PackageJsonPreset`]s.
	#[merge(strategy = IndexMap::extend)]
	#[arg(skip)]
	pub package_json_presets: IndexMap<String, PackageJsonPreset>,

	/// A map containing [`TsConfigPreset`]s.
	#[merge(strategy = IndexMap::extend)]
	#[arg(skip)]
	pub ts_config_presets: IndexMap<String, TsConfigPreset>,

	/// A map containing [`OxlintPreset`]s.
	#[merge(strategy = IndexMap::extend)]
	#[arg(skip)]
	pub oxlint_presets: IndexMap<String, OxlintPreset>,

	/// A map of [`PackageConfig`] presets.
	#[merge(strategy = IndexMap::extend)]
	#[arg(skip)]
	pub package_presets: IndexMap<String, PackageConfig>,

	/// A map of presets for `pnpm-workspace.yaml` configurations.
	#[merge(strategy = IndexMap::extend)]
	#[arg(skip)]
	pub pnpm_presets: IndexMap<String, PnpmPreset>,

	/// A map of presets for vitest setups.
	#[merge(strategy = IndexMap::extend)]
	#[arg(skip)]
	pub vitest_presets: IndexMap<String, VitestConfig>,
}

impl PackageManager {
	pub fn find_root(&self, start_dir: &Path) -> Option<PathBuf> {
		let root_marker = self.root_marker();

		find_file_up(start_dir, root_marker).map(|file| get_parent_dir(&file).to_path_buf())
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
}

/// A js/ts package manager.
#[derive(
	Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default, ValueEnum, Copy, JsonSchema,
)]
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

pub(crate) static CATALOG_REGEX: LazyLock<Regex> = LazyLock::new(|| {
	Regex::new(r"^catalog:(?<name>\w+)?$").expect("Failed to initialize the catalog regex")
});
