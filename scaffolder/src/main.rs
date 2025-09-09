#![allow(clippy::result_large_err)]

use clap::{error::ErrorKind, CommandFactory, Parser};
use merge::Merge;
mod cli;
use scaffolder::{
  moon::{MoonConfigKind, MoonDotYmlKind},
  package::vitest::VitestConfig,
  *,
};

use crate::cli::{
  Cli,
  Commands::{Command, Init, Package, Render, Repo},
  RepoBooleanFlags,
};

#[tokio::main]
async fn main() -> Result<(), GenError> {
  let cli = Cli::parse();

  let mut config = if let Some(config_path) = cli.config.as_deref() {
    match Config::from_file(config_path) {
      Ok(conf) => conf,
      Err(e) => {
        let mut cmd = Cli::command();
        cmd.error(ErrorKind::InvalidValue, format!("{}", e)).exit();
      }
    }
  } else {
    Config::default()
  };

  if let Some(overrides) = cli.overrides {
    config.merge(overrides);
  }

  match cli.command {
    Repo {
      root_package,
      boolean_flags,
      ..
    } => {
      if let Some(root_package) = root_package {
        config.root_package.merge(root_package);
      }

      println!("{:?}", boolean_flags);

      let RepoBooleanFlags {
        no_pre_commit,
        no_convert_latest,
        no_oxlint,
        no_catalog,
        no_overwrite,
        moonrepo,
        shared_out_dir,
        no_shared_out_dir,
      } = boolean_flags;

      if no_convert_latest {
        config.convert_latest_to_range = false;
      }

      if no_pre_commit {
        config.pre_commit = PreCommitSetting::Bool(false);
      }

      if no_oxlint {
        config.root_package.oxlint = Some(OxlintConfig::Bool(false));
      }

      if no_catalog {
        config.catalog = false;
      }

      if no_overwrite {
        config.overwrite = false;
      }

      if moonrepo {
        config.moonrepo = Some(MoonConfigKind::Bool(true));
      }

      if no_shared_out_dir {
        config.shared_out_dir = SharedOutDir::Bool(false);
      }

      if let Some(shared_out_dir) = shared_out_dir {
        config.shared_out_dir = SharedOutDir::Name(shared_out_dir);
      }
    }
    Package {
      config: package_config,
      kind,
      preset,
      name,
      moonrepo,
      no_vitest,
      oxlint,
      no_update_root_tsconfig,
      ..
    } => {
      let mut package = package_config.unwrap_or_default();

      package.name = name.clone();

      if let Some(kind) = kind {
        package.kind = Some(kind.into());
      }

      if moonrepo {
        package.moonrepo = Some(MoonDotYmlKind::Bool(true));
      }

      if no_vitest {
        package.vitest = Some(VitestConfig::Boolean(false));
      }

      if oxlint {
        package.oxlint = Some(OxlintConfig::Bool(true));
      }

      if no_update_root_tsconfig {
        package.update_root_tsconfig = false;
      }

      let _id = if let Some(preset) = preset {
        preset
      } else {
        let new_id = format!("__{}", name);
        config.package_presets.insert(new_id.clone(), package);
        new_id
      };
    }
    Render {
      content,
      output,
      id,
      ..
    } => {
      println!("Content: {:?}, Output: {}, Id: {:?}", content, output, id);
    }
    Init => {}
    Command => {}
  };

  Ok(())
}
