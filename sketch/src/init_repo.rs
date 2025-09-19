use askama::Template;

use crate::{
  exec::launch_command,
  fs::{create_parent_dirs, get_cwd},
  Config, GenError,
};

impl Config {
  pub fn init_repo(self, remote: Option<&str>) -> Result<(), GenError> {
    let out_dir = self.out_dir.unwrap_or_else(|| get_cwd());
    let shell = self.shell.as_deref();

    create_parent_dirs(&out_dir)?;

    macro_rules! write_to_output {
      ($($tokens:tt)*) => {
        write_file!(out_dir, self.no_overwrite, $($tokens)*)
      };
    }

    write_to_output!(self.gitignore, ".gitignore");

    if self.pre_commit.is_enabled() {
      write_to_output!(&self.pre_commit, ".pre-commit-config.yaml");
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
