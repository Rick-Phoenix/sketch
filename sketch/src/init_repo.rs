use std::fs::{create_dir_all, File};

use askama::Template;

use crate::{
  commands::launch_command, paths::get_cwd, Config, GenError, PreCommitConfig, PreCommitSetting,
};

impl Config {
  pub fn init_repo(self, remote: Option<&str>) -> Result<(), GenError> {
    let root_dir = self.root_dir.unwrap_or_else(|| get_cwd());
    let shell = self.shell.as_deref();

    create_dir_all(&root_dir).map_err(|e| GenError::DirCreation {
      path: root_dir.to_owned(),
      source: e,
    })?;

    macro_rules! write_to_output {
      ($($tokens:tt)*) => {
        write_file!(root_dir, !self.no_overwrite, $($tokens)*)
      };
    }

    write_to_output!(self.gitignore, ".gitignore");

    let pre_commit_config = match &self.pre_commit {
      PreCommitSetting::Bool(val) => {
        if *val {
          Some(&PreCommitConfig::default())
        } else {
          None
        }
      }
      PreCommitSetting::Config(conf) => Some(conf),
    };

    if let Some(pre_commit) = pre_commit_config {
      write_to_output!(pre_commit, ".pre-commit-config.yaml");
      launch_command(
        shell,
        &["pre-commit", "install"],
        &root_dir,
        Some("Failed to install the pre-commit hooks"),
      )?;
    }

    launch_command(
      shell,
      &["git", "init"],
      &root_dir,
      Some("Failed to initialize a new git repo"),
    )?;

    if let Some(remote) = remote {
      launch_command(
        shell,
        &["git", "remote", "add", "origin", remote],
        &root_dir,
        Some("Failed to add the remote to the git repo"),
      )?;
    }

    Ok(())
  }
}
