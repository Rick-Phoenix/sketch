use crate::{
	docker::DockerConfig,
	gh_workflow::GithubConfig,
	init_repo::{RepoPreset, gitignore::GitignorePreset, pre_commit::PreCommitPreset},
	rust::RustConfig,
	ts::TypescriptConfig,
	*,
};

mod config_setup;
use config_setup::extract_config_from_file;

impl Config {
	pub fn new() -> Self {
		Self {
			..Default::default()
		}
	}

	pub(crate) const fn can_overwrite(&self) -> bool {
		!self.no_overwrite
	}
}

/// The global configuration struct.
#[allow(clippy::struct_field_names)]
#[derive(Clone, Debug, Deserialize, Serialize, Merge, PartialEq, Default)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct Config {
	#[serde(skip)]
	#[merge(with = overwrite_if_none)]
	pub(crate) config_file: Option<PathBuf>,

	/// The configuration for typescript projects.
	#[merge(with = merge_options)]
	pub typescript: Option<TypescriptConfig>,

	/// Configuration and presets for Docker.
	#[merge(with = merge_options)]
	pub docker: Option<DockerConfig>,

	/// The shell to use for commands [default: `cmd.exe` on windows and `sh` elsewhere].
	pub shell: Option<String>,

	/// The path to the templates directory.
	pub templates_dir: Option<PathBuf>,

	/// Do not overwrite existing files.
	#[merge(with = overwrite_if_true)]
	pub no_overwrite: bool,

	/// The paths (absolute, or relative to the originating config file) to the config files to extend.
	pub extends: IndexSet<PathBuf>,

	/// A map that contains template definitions.
	pub templates: IndexMap<String, String>,

	/// A map that contains templating presets.
	pub templating_presets: IndexMap<String, TemplatingPreset>,

	/// A map that contains pre-commit presets.
	pub pre_commit_presets: IndexMap<String, PreCommitPreset>,

	/// A map that contains gitignore presets.
	pub gitignore_presets: IndexMap<String, GitignorePreset>,

	/// A map that contains presets for git repos.
	pub repo_presets: IndexMap<String, RepoPreset>,

	/// Configurations and presets for Rust based projects
	pub rust: RustConfig,

	/// Configurations and presets relating to Github
	pub github: GithubConfig,

	/// The global variables that will be available for every template being generated.
	/// They are overridden by vars set in a template's local context or via the cli.
	pub vars: IndexMap<String, Value>,
}

impl Config {
	fn merge_configs_recursive(
		&self,
		is_initial: bool,
		base: &mut Self,

		processed_sources: &mut IndexSet<PathBuf>,
	) -> Result<(), AppError> {
		// Safe unwrapping due to the check below
		let current_config_file = self.config_file.as_ref();
		let current_dir = get_parent_dir(current_config_file.unwrap())?;

		for rel_path in &self.extends {
			let abs_path = current_dir
				.join(rel_path)
				.canonicalize()
				.map_err(|e| AppError::PathCanonicalization {
					path: rel_path.clone(),
					source: e,
				})?;

			let extended_config = extract_config_from_file(&abs_path)?;

			let was_absent = processed_sources.insert(abs_path.clone());

			if !was_absent {
				let chain: Vec<_> = processed_sources
					.iter()
					.map(|source| source.to_string_lossy())
					.collect();

				return Err(AppError::CircularDependency(format!(
					"Found circular dependency to the config file {}. The full processed path is: {}",
					abs_path.display(),
					chain.join(" -> ")
				)));
			}

			extended_config.merge_configs_recursive(false, base, processed_sources)?;

			base.merge(extended_config);
		}

		if !is_initial {
			base.merge(self.clone());
		}

		Ok(())
	}

	/// Recursively merges a [`Config`] with its extended configs.
	pub fn merge_config_files(self) -> Result<Self, AppError> {
		let mut processed_sources: IndexSet<PathBuf> = Default::default();

		let config_file = self
			.config_file
			.clone()
			.context("Attempted to merge a config without a source file")?;

		processed_sources.insert(config_file.clone());

		let mut extended = Self::default();

		self.merge_configs_recursive(true, &mut extended, &mut processed_sources)?;

		extended.merge(self);

		// Should not show up in the `extends` list
		processed_sources.swap_remove(&config_file);

		// Replace rel paths with abs paths for better debugging
		extended.extends = processed_sources;

		Ok(extended)
	}
}
