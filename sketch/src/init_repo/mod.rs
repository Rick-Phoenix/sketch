use std::path::Path;

use clap::Args;
use indexmap::IndexMap;
use merge::Merge;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod gitignore;
pub mod pre_commit;

use crate::{
	Config, GenError, Preset,
	custom_templating::TemplatingPresetReference,
	exec::{Hook, launch_command},
	fs::{create_all_dirs, serialize_yaml, write_file},
	git_workflow::WorkflowReference,
	init_repo::{
		gitignore::{DEFAULT_GITIGNORE, GitIgnore, GitIgnoreRef, GitignorePreset},
		pre_commit::{PreCommitPreset, PreCommitSetting},
	},
	licenses::License,
	merge_vecs, overwrite_always, overwrite_if_some,
};

/// A preset for a git repository.
#[derive(Args, Clone, Debug, Deserialize, Serialize, PartialEq, JsonSchema, Default, Merge)]
#[serde(default)]
pub struct RepoPreset {
	#[arg(short, long)]
	#[merge(strategy = overwrite_if_some)]
	/// Settings for the gitignore file.
	pub gitignore: Option<GitIgnoreRef>,

	#[arg(short, long)]
	#[merge(strategy = overwrite_always)]
	/// Configuration settings for [`pre-commit`](https://pre-commit.com/).
	pub pre_commit: PreCommitSetting,

	#[arg(short = 't', long = "template")]
	#[merge(strategy = merge_vecs)]
	/// A set of templates to generate when this preset is used.
	pub with_templates: Vec<TemplatingPresetReference>,

	#[arg(short, long)]
	#[merge(strategy = overwrite_if_some)]
	/// A license file to generate for the new repo.
	pub license: Option<License>,

	#[arg(skip)]
	#[merge(strategy = merge_vecs)]
	/// One or many rendered commands to execute before the repo's creation
	pub hooks_pre: Vec<Hook>,

	#[arg(skip)]
	#[merge(strategy = merge_vecs)]
	/// One or many rendered commands to execute after the repo's creation
	pub hooks_post: Vec<Hook>,

	#[arg(
    long = "workflow",
    value_name = "id=PRESET_ID,file=PATH",
    value_parser = WorkflowReference::from_cli
  )]
	#[merge(strategy = merge_vecs)]
	/// One or many workflows to generate in the new repo.
	pub workflows: Vec<WorkflowReference>,
}

impl std::str::FromStr for PreCommitSetting {
	type Err = std::convert::Infallible;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(Self::Id(s.to_string()))
	}
}

impl Config {
	pub fn init_repo(
		self,
		mut preset: RepoPreset,
		remote: Option<&str>,
		out_dir: &Path,
		cli_vars: &IndexMap<String, Value>,
	) -> Result<(), GenError> {
		let overwrite = self.can_overwrite();

		if !preset.hooks_pre.is_empty() {
			self.execute_command(
				self.shell.as_deref(),
				out_dir,
				preset.hooks_pre,
				cli_vars,
				false,
			)?;
		}

		create_all_dirs(out_dir)?;

		let mut gitignore_id: Option<String> = None;

		if let Some(GitIgnoreRef::Id(id)) = preset.gitignore {
			gitignore_id = Some(id.clone());

			let data = self
				.gitignore_presets
				.get(&id)
				.ok_or_else(|| GenError::PresetNotFound {
					kind: Preset::Gitignore,
					name: id,
				})?
				.clone();

			preset.gitignore = Some(GitIgnoreRef::Config(data));
		}

		if let Some(GitIgnoreRef::Config(data)) = preset.gitignore {
			let resolved = data.process_data(
				gitignore_id.as_deref().unwrap_or("__inlined"),
				&self.gitignore_presets,
			)?;

			preset.gitignore = Some(GitIgnoreRef::Config(resolved));
		}

		if preset.gitignore.is_none() {
			preset.gitignore = Some(GitIgnoreRef::Config(GitignorePreset {
				extends_presets: Default::default(),
				content: GitIgnore::String(DEFAULT_GITIGNORE.trim().to_string()),
			}));
		}

		let Some(GitIgnoreRef::Config(gitignore)) = preset.gitignore else {
			panic!("Unresolved gitignore");
		};

		write_file(
			&out_dir.join(".gitignore"),
			&gitignore.content.to_string(),
			overwrite,
		)?;

		launch_command(
			"git",
			&["init"],
			out_dir,
			Some("Failed to initialize a new git repo"),
		)?;

		if preset.pre_commit.is_enabled() {
			let (pre_commit_id, pre_commit_preset) = match preset.pre_commit {
				PreCommitSetting::Id(id) => (
					id.clone(),
					self.pre_commit_presets
						.get(id.as_str())
						.ok_or(GenError::PresetNotFound {
							kind: Preset::PreCommit,
							name: id.clone(),
						})?
						.clone(),
				),
				PreCommitSetting::Bool(_) => ("__default".to_string(), PreCommitPreset::default()),
				PreCommitSetting::Config(pre_commit_config) => {
					("__inlined_definition".to_string(), pre_commit_config)
				}
			};

			let pre_commit_config =
				pre_commit_preset.process_data(&pre_commit_id, &self.pre_commit_presets)?;

			serialize_yaml(
				&pre_commit_config,
				&out_dir.join(".pre-commit-config.yaml"),
				overwrite,
			)?;

			launch_command(
				"pre-commit",
				&["install"],
				out_dir,
				Some("Failed to install the pre-commit hooks"),
			)?;
		}

		if let Some(remote) = remote {
			launch_command(
				"git",
				&["remote", "add", "origin", remote],
				out_dir,
				Some("Failed to add the remote to the git repo"),
			)?;
		}

		if let Some(license) = preset.license {
			write_file(&out_dir.join("LICENSE"), license.get_content(), overwrite)?;
		}

		if !preset.workflows.is_empty() {
			let workflows_dir = out_dir.join(".github/workflows");
			create_all_dirs(&workflows_dir)?;

			for workflow in preset.workflows {
				match workflow {
					WorkflowReference::Preset { file_name, id } => {
						let data = self
							.github
							.workflow_presets
							.get(&id)
							.ok_or(GenError::PresetNotFound {
								kind: Preset::GithubWorkflow,
								name: id.clone(),
							})?
							.clone()
							.process_data(&id, &self.github)?;

						serialize_yaml(&data, &workflows_dir.join(file_name), overwrite)?;
					}
					WorkflowReference::Data {
						file_name,
						workflow: config,
					} => {
						let data = config.process_data("__inlined", &self.github)?;

						serialize_yaml(&data, &workflows_dir.join(file_name), overwrite)?;
					}
				}
			}
		}

		if !preset.with_templates.is_empty() {
			self.generate_templates(out_dir, preset.with_templates, cli_vars)?;
		}

		if !preset.hooks_post.is_empty() {
			self.execute_command(
				self.shell.as_deref(),
				out_dir,
				preset.hooks_post,
				cli_vars,
				false,
			)?;
		}

		Ok(())
	}
}
