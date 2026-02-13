use std::mem::take;

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

	#[cfg(feature = "schemars")]
	pub fn generate_json_schema(output: &Path) -> AppResult {
		#[allow(clippy::use_self)]
		let schema = schemars::schema_for!(Config);
		serialize_json(&schema, output, true)?;

		Ok(())
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

	#[merge(skip)]
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
		mut self,
		processed_sources: &mut IndexSet<PathBuf>,
	) -> Result<Self, AppError> {
		// There is always a config file when this is called
		let config_file = self.config_file.clone().unwrap();

		let config_file_parent_dir = get_parent_dir(&config_file)?.to_path_buf();

		let paths_to_extend = take(&mut self.extends);

		for target_rel_path in &paths_to_extend {
			let target_abs_path = get_abs_path(&config_file_parent_dir.join(target_rel_path))?;

			let was_absent = processed_sources.insert(target_abs_path.clone());

			if !was_absent {
				let chain: Vec<_> = processed_sources
					.iter()
					.map(|source| source.to_string_lossy())
					.collect();

				return Err(AppError::CircularDependency(format!(
					"Found circular dependency to the config file {}. The full processed path is: {}",
					target_abs_path.display(),
					chain.join(" -> ")
				)));
			}

			let mut config_to_extend = extract_config_from_file(&target_abs_path)?
				.merge_configs_recursive(processed_sources)?;

			config_to_extend.merge(self);

			self = config_to_extend;
		}

		self.extends = paths_to_extend;
		self.config_file = Some(config_file);

		Ok(self)
	}

	/// Recursively merges a [`Config`] with its extended configs.
	pub fn merge_config_files(mut self) -> Result<Self, AppError> {
		let mut processed_sources: IndexSet<PathBuf> = Default::default();

		let config_file = self
			.config_file
			.clone()
			.context("Attempted to merge a config without a source file")?;

		processed_sources.insert(config_file.clone());

		self = self.merge_configs_recursive(&mut processed_sources)?;

		// Should not show up in the `extends` list we insert below
		processed_sources.swap_remove(&config_file);

		// Replace rel paths with abs paths for better debugging
		self.extends = processed_sources;

		Ok(self)
	}
}
