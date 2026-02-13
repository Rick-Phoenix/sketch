pub(crate) use pre_commit_config::*;

use super::*;

/// A preset for a `.pre-commit-config.yaml` configuration file.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Merge)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default)]
pub struct PreCommitPreset {
	/// The ids of the extended configurations.
	pub extends_presets: IndexSet<String>,

	#[serde(flatten)]
	pub config: PreCommitConfig,
}

impl Default for PreCommitPreset {
	fn default() -> Self {
		Self {
			extends_presets: Default::default(),
			config: default_pre_commit(),
		}
	}
}

impl Config {
	pub fn get_pre_commit_preset(&self, id: &str) -> AppResult<PreCommitPreset> {
		self.pre_commit_presets
			.get(id)
			.ok_or(AppError::PresetNotFound {
				kind: PresetKind::PreCommit,
				name: id.to_string(),
			})?
			.clone()
			.merge_presets(id, &self.pre_commit_presets)
	}
}

impl ExtensiblePreset for PreCommitPreset {
	fn kind() -> PresetKind {
		PresetKind::PreCommit
	}

	fn get_extended_ids(&self) -> &IndexSet<String> {
		&self.extends_presets
	}
}

fn default_pre_commit() -> PreCommitConfig {
	PreCommitConfig {
		repos: btreeset! { GITLEAKS_REPO.clone() },
		ci: None,
		default_install_hook_types: Default::default(),
		default_language_version: Default::default(),
		default_stages: Default::default(),
		files: None,
		exclude: None,
		fail_fast: None,
		minimum_pre_commit_version: None,
	}
}

/// Settings for [`pre-commit`](https://pre-commit.com)  Can be a preset id, a newly defined configuration, or a boolean to use defaults or to disable pre-commit.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
pub enum PreCommitSetting {
	Bool(bool),
	Id(String),
	Config(PreCommitPreset),
}

impl Default for PreCommitSetting {
	fn default() -> Self {
		Self::Bool(true)
	}
}

impl PreCommitSetting {
	pub(crate) const fn is_enabled(&self) -> bool {
		!matches!(self, Self::Bool(false))
	}
}

pub(crate) static GITLEAKS_REPO: LazyLock<Repo> = LazyLock::new(|| Repo::Uri {
	repo: "https://github.com/gitleaks/gitleaks".to_string(),
	rev: Some("v8.28.0".to_string()),
	hooks: BTreeSet::from_iter([PreCommitHook {
		id: "gitleaks".to_string(),
		..Default::default()
	}]),
});
