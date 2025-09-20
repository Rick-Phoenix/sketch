use askama::Template;

pub mod gitignore;
pub mod pre_commit;

use crate::{
  exec::launch_command,
  fs::{create_all_dirs, get_cwd, serialize_yaml},
  init_repo::pre_commit::{PreCommitPreset, PreCommitSetting},
  Config, GenError, Preset,
};

impl Config {
  pub fn init_repo(self, remote: Option<&str>) -> Result<(), GenError> {
    let out_dir = self.out_dir.unwrap_or_else(|| get_cwd());
    let shell = self.shell.as_deref();

    create_all_dirs(&out_dir)?;

    macro_rules! write_to_output {
      ($($tokens:tt)*) => {
        write_file!(out_dir, self.no_overwrite, $($tokens)*)
      };
    }

    write_to_output!(self.gitignore, ".gitignore");

    if self.pre_commit.is_enabled() {
      let (pre_commit_id, pre_commit_preset) = match self.pre_commit {
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

    launch_command(
      shell,
      &["git", "init"],
      &out_dir,
      Some("Failed to initialize a new git repo"),
    )?;

    if let Some(remote) = remote {
      launch_command(
        shell,
        &["git", "remote", "add", "origin", remote],
        &out_dir,
        Some("Failed to add the remote to the git repo"),
      )?;
    }

    Ok(())
  }
}
