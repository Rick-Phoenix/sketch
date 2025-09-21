use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod gitignore;
pub mod pre_commit;

use crate::{
  custom_templating::TemplateOutput,
  exec::launch_command,
  fs::{create_all_dirs, get_cwd, serialize_yaml, write_file},
  init_repo::{
    gitignore::{GitIgnore, GitIgnoreSetting, DEFAULT_GITIGNORE},
    pre_commit::{PreCommitPreset, PreCommitSetting},
  },
  Config, GenError, Preset,
};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, JsonSchema, Default)]
pub struct RepoPreset {
  /// Settings for the gitignore file to generate in new repos. It can be a list of strings to append to the defaults or a single string, to replace the defaults entirely.
  pub gitignore: Option<GitIgnoreSetting>,
  /// Configuration settings for [`pre-commit`](https://pre-commit.com/), to use when creating a new repo.
  pub pre_commit: PreCommitSetting,
  /// A set of templates to generate when this preset is used.
  pub with_templates: Option<Vec<TemplateOutput>>,
}

impl Config {
  pub fn init_repo(self, preset: RepoPreset, remote: Option<&str>) -> Result<(), GenError> {
    let out_dir = self.out_dir.clone().unwrap_or_else(|| get_cwd());
    let shell = self.shell.as_deref();

    create_all_dirs(&out_dir)?;

    let gitignore = if let Some(data) = preset.gitignore {
      match data {
        GitIgnoreSetting::Id(id) => self
          .gitignore_presets
          .get(&id)
          .ok_or(GenError::PresetNotFound {
            kind: Preset::Gitignore,
            name: id.clone(),
          })?
          .clone()
          .process_data(&id, &self.gitignore_presets)?,
        GitIgnoreSetting::Config(git_ignore) => git_ignore,
      }
    } else {
      GitIgnore::String(DEFAULT_GITIGNORE.trim().to_string())
    };

    write_file(
      &out_dir.join(".gitignore"),
      &gitignore.to_string(),
      self.no_overwrite,
    )?;

    launch_command(
      shell,
      &["git", "init"],
      &out_dir,
      Some("Failed to initialize a new git repo"),
    )?;

    if preset.pre_commit.is_enabled() {
      let (pre_commit_id, pre_commit_preset) = match preset.pre_commit {
        PreCommitSetting::Id(id) => (
          id.clone(),
          self
            .pre_commit_presets
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

      serialize_yaml(&pre_commit_config, &out_dir.join(".pre-commit-config.yaml"))?;

      launch_command(
        shell,
        &["pre-commit", "install"],
        &out_dir,
        Some("Failed to install the pre-commit hooks"),
      )?;
    }

    if let Some(remote) = remote {
      launch_command(
        shell,
        &["git", "remote", "add", "origin", remote],
        &out_dir,
        Some("Failed to add the remote to the git repo"),
      )?;
    }

    if let Some(templates) = preset.with_templates {
      self.generate_templates(&out_dir, templates)?;
    }

    Ok(())
  }
}
