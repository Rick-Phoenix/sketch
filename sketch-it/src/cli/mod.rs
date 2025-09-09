#![allow(clippy::large_enum_variant)]
use std::{fs::read_to_string, path::PathBuf};

use Commands::*;

use crate::{
  commands::launch_command,
  tera::{TemplateData, TemplateOutput},
};

pub(crate) mod parsers;

use std::env::current_dir;

use clap::{error::ErrorKind, Args, CommandFactory, Parser, Subcommand};
use merge::Merge;
use parsers::parse_serializable_key_value_pair;
use serde_json::Value;

use crate::{
  moon::{MoonConfigKind, MoonDotYmlKind},
  package::{vitest::VitestConfig, PackageConfig, PackageKind},
  Config, RootPackage, *,
};

pub async fn start_cli() -> Result<(), GenError> {
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

  if let Some(vars) = cli.templates_vars {
    config.global_templates_vars.extend(vars);
  }

  macro_rules! exit_if_dry_run {
    () => {
      if cli.dry_run {
        println!("Aborting due to dry run...");
        return Ok(());
      }
    };
  }

  match cli.command {
    Repo {
      root_package,
      boolean_flags,
      ..
    } => {
      println!("{:#?}", root_package);
      println!("{:?}", boolean_flags);

      if let Some(root_package) = root_package {
        config.root_package.merge(root_package);
      }

      let RepoBooleanFlags {
        no_git,
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

      exit_if_dry_run!();

      if !no_git {
        let cwd = config.root_dir.as_deref().unwrap_or(".");
        let shell = config.shell.as_deref();

        launch_command(
          shell,
          &["git", "init"],
          cwd,
          Some("Failed to initialize a new git repo"),
        )?;

        if let Some(remote) = config.remote {
          launch_command(
            shell,
            &["git", "remote", "add", "origin", remote.as_str()],
            cwd,
            Some("Failed to add the remote to the git repo"),
          )?;
        }

        if !no_pre_commit {
          launch_command(
            shell,
            &["pre-commit", "install"],
            cwd,
            Some("Failed to install the pre-commit hooks"),
          )?;
        }
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

      if config.debug {
        println!("DEBUG: {:#?}", config);
      }

      exit_if_dry_run!();
    }
    Render {
      content,
      output,
      id,
      ..
    } => {
      let template_data = if let Some(id) = id {
        TemplateData::Id(id)
      } else if let Some(content) = content {
        TemplateData::Content {
          name: "template_from_cli".to_string(),
          content,
        }
      } else {
        panic!("Missing id or content for template generation");
      };

      let template = TemplateOutput {
        output,
        context: Default::default(),
        template: template_data,
      };

      if config.debug {
        println!("DEBUG: {:#?}", template);
      }

      exit_if_dry_run!();

      config.generate_templates(
        &current_dir()
          .expect("Could not get the cwd")
          .to_string_lossy(),
        vec![template],
      )?;
    }
    Init => {}
    Command { command, cwd } => {
      let command = if let Some(literal) = command.command {
        literal
      } else if let Some(file_path) = command.file {
        read_to_string(&file_path).map_err(|e| GenError::ReadError {
          path: file_path,
          source: e,
        })?
      } else {
        panic!("At least one between --command and --file must be set.")
      };

      exit_if_dry_run!();

      let shell = config.shell.clone();
      config.execute_command(shell.as_deref(), cwd, &command)?;
    }
  };

  Ok(())
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
  /// Sets a custom config file
  #[arg(short, long, value_name = "FILE")]
  pub config: Option<PathBuf>,

  #[command(subcommand)]
  pub command: Commands,

  #[command(flatten)]
  pub overrides: Option<Config>,

  #[arg(long)]
  pub dry_run: bool,

  #[arg(long = "set", short = 's', value_parser = parse_serializable_key_value_pair)]
  pub templates_vars: Option<Vec<(String, Value)>>,
}

#[derive(Args, Debug)]
#[group(multiple = false)]
struct PackageKindFlag {
  #[arg(long)]
  app: bool,

  #[arg(long)]
  library: bool,
}

impl From<PackageKindFlag> for PackageKind {
  fn from(value: PackageKindFlag) -> Self {
    if value.app {
      Self::App
    } else {
      Self::Library
    }
  }
}

#[derive(Args, Debug)]
struct RepoBooleanFlags {
  #[arg(long)]
  pub(crate) no_convert_latest: bool,
  #[arg(long)]
  pub(crate) no_oxlint: bool,
  #[arg(long)]
  pub(crate) no_catalog: bool,
  #[arg(long)]
  pub(crate) no_overwrite: bool,
  #[arg(long)]
  pub(crate) no_pre_commit: bool,
  #[arg(long)]
  pub(crate) moonrepo: bool,
  #[arg(long, conflicts_with = "no_shared_out_dir")]
  pub(crate) shared_out_dir: Option<String>,
  #[arg(long, default_value_t = false)]
  pub(crate) no_shared_out_dir: bool,
  #[arg(long)]
  pub(crate) no_git: bool,
}

#[derive(Subcommand)]
enum Commands {
  /// Generates a new config file
  Init,
  /// Generates a new monorepo
  Repo {
    #[command(flatten)]
    root_package: Option<RootPackage>,
    #[command(flatten)]
    boolean_flags: RepoBooleanFlags,
  },

  /// Generates a new package
  Package {
    name: String,
    #[arg(short, long, conflicts_with = "PackageConfig")]
    preset: Option<String>,
    #[command(flatten)]
    kind: Option<PackageKindFlag>,
    #[command(flatten)]
    config: Option<PackageConfig>,
    #[arg(long)]
    moonrepo: bool,
    #[arg(long)]
    no_vitest: bool,
    #[arg(long)]
    oxlint: bool,
    #[arg(long)]
    no_update_root_tsconfig: bool,
  },

  /// Generates a file from a template
  Render {
    #[arg(requires = "input")]
    output: String,
    #[arg(short, long, group = "input")]
    id: Option<String>,
    #[arg(short, long, group = "input")]
    content: Option<String>,
  },

  Command {
    #[command(flatten)]
    command: CommandContent,
    #[arg(long)]
    cwd: Option<PathBuf>,
  },
}

#[derive(Args)]
#[group(required = true, multiple = false)]
struct CommandContent {
  command: Option<String>,
  #[arg(short, long)]
  file: Option<PathBuf>,
}
