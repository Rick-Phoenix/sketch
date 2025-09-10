#![allow(clippy::large_enum_variant)]
use std::{
  fmt::Display,
  fs::read_to_string,
  io::{self, Write},
  path::PathBuf,
  str::FromStr,
};

use Commands::*;

use crate::{
  commands::launch_command,
  tera::{TemplateData, TemplateOutput},
};

pub(crate) mod parsers;

use std::env::current_dir;

use clap::{error::ErrorKind, Args, CommandFactory, Parser, Subcommand, ValueEnum};
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
    TsMonorepo {
      root_package: root_package_overrides,
      boolean_flags,
      ..
    } => {
      let root_package = &mut config.root_package.unwrap_or_default();
      if let Some(root_package_overrides) = root_package_overrides {
        root_package.merge(root_package_overrides);
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
        root_package.oxlint = Some(OxlintConfig::Bool(false));
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
    TsPackage {
      config: package_config,
      kind,
      preset,
      name,
      moonrepo,
      no_vitest,
      oxlint,
      no_update_root_tsconfig,
      install,
      ..
    } => {
      let mut package = package_config.unwrap_or_default();
      let package_dir = package.dir.clone();

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

      if install {
        launch_command(
          None,
          &[
            config
              .package_manager
              .unwrap_or_default()
              .to_string()
              .as_str(),
            "install",
          ],
          package_dir.as_deref().unwrap_or("."),
          Some("Could not install dependencies"),
        )?;
      }
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
    Init { output } => {
      let output_path = output.unwrap_or_else(|| PathBuf::from("sketch.yaml"));

      if let Some(parent_dir) = output_path.parent() {
        create_dir_all(parent_dir).map_err(|e| GenError::DirCreation {
          path: parent_dir.to_path_buf(),
          source: e,
        })?;
      }

      let format = <ConfigFormat as FromStr>::from_str(
        &output_path
          .extension()
          .unwrap_or_else(|| panic!("File {} has no extension.", output_path.display()))
          .to_string_lossy(),
      )?;

      let mut output_file = if config.overwrite {
        File::create(&output_path).map_err(|e| GenError::FileCreation {
          path: output_path.clone(),
          source: e,
        })?
      } else {
        File::create_new(&output_path).map_err(|e| match e.kind() {
          io::ErrorKind::AlreadyExists => GenError::FileExists {
            path: output_path.clone(),
          },
          _ => GenError::WriteError {
            path: output_path.clone(),
            source: e,
          },
        })?
      };

      let base_config = Config::default();

      match format {
        ConfigFormat::Yaml => serde_yaml_ng::to_writer(output_file, &base_config).map_err(|e| {
          GenError::SerializationError {
            target: "the new config file".to_string(),
            error: e.to_string(),
          }
        })?,
        ConfigFormat::Toml => {
          let content =
            toml::to_string_pretty(&base_config).map_err(|e| GenError::SerializationError {
              target: "the new config file".to_string(),
              error: e.to_string(),
            })?;

          output_file
            .write_all(&content.into_bytes())
            .map_err(|e| GenError::WriteError {
              path: output_path.clone(),
              source: e,
            })?;
        }
        ConfigFormat::Json => {
          serde_json::to_writer_pretty(output_file, &base_config).map_err(|e| {
            GenError::SerializationError {
              target: "the new config file".to_string(),
              error: e.to_string(),
            }
          })?
        }
      };
    }
    Command { command, cwd } => {
      let command = if let Some(literal) = command.command {
        literal
      } else if let Some(file_path) = command.file {
        read_to_string(&file_path).map_err(|e| GenError::ReadError {
          path: file_path,
          source: e,
        })?
      } else {
        panic!("At least one between command and file must be set.")
      };

      exit_if_dry_run!();

      let shell = config.shell.clone();
      config.execute_command(shell.as_deref(), cwd, &command)?;
    }
  };

  Ok(())
}

#[derive(Parser)]
#[command(name = "sketch")]
#[command(version, about, long_about = None)]
struct Cli {
  /// Sets a custom config file
  #[arg(short, long, value_name = "FILE")]
  pub config: Option<PathBuf>,

  #[command(subcommand)]
  pub command: Commands,

  #[command(flatten)]
  pub overrides: Option<Config>,

  /// Aborts before writing any content to disk.
  #[arg(long)]
  pub dry_run: bool,

  /// Set variables to use in templates. It overrides previously set variables with the same names.
  #[arg(long = "set", short = 's', value_parser = parse_serializable_key_value_pair)]
  pub templates_vars: Option<Vec<(String, Value)>>,
}

#[derive(Args, Debug)]
#[group(multiple = false)]
struct PackageKindFlag {
  /// Marks the package as an application.
  #[arg(long)]
  app: bool,

  /// Marks the package as a library.
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
  /// Do not convert 'latest' to a version range.
  #[arg(long)]
  pub(crate) no_convert_latest: bool,

  /// Do not generate an oxlint config.
  #[arg(long)]
  pub(crate) no_oxlint: bool,

  /// Do not use the catalog for dependencies.
  #[arg(long)]
  pub(crate) no_catalog: bool,

  /// Do not overwrite files.
  #[arg(long)]
  pub(crate) no_overwrite: bool,

  /// Do not generate a pre-commit config.
  #[arg(long)]
  pub(crate) no_pre_commit: bool,

  /// Generate setup for moonrepo
  #[arg(long)]
  pub(crate) moonrepo: bool,

  /// The path to the shared out_dir for TS packages.
  #[arg(long, conflicts_with = "no_shared_out_dir")]
  pub(crate) shared_out_dir: Option<String>,

  /// Do not use a shared out_dir for TS packages.
  #[arg(long, default_value_t = false)]
  pub(crate) no_shared_out_dir: bool,

  /// Do not create a new git repo.
  #[arg(long)]
  pub(crate) no_git: bool,
}

#[derive(Clone, Debug, ValueEnum, Default)]
enum ConfigFormat {
  #[default]
  Yaml,
  Toml,
  Json,
}

impl FromStr for ConfigFormat {
  type Err = GenError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "yaml" => Ok(Self::Yaml),
      "toml" => Ok(Self::Toml),
      "json" => Ok(Self::Json),
      _ => Err(GenError::Custom(format!(
        "Invalid configuration format '{}'. Allowed formats are: yaml, toml, json",
        s
      ))),
    }
  }
}

impl Display for ConfigFormat {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ConfigFormat::Yaml => write!(f, "yaml"),
      ConfigFormat::Toml => write!(f, "toml"),
      ConfigFormat::Json => write!(f, "json"),
    }
  }
}

#[derive(Subcommand)]
enum Commands {
  /// Generates a new config file
  Init { output: Option<PathBuf> },
  /// Generates a new typescript monorepo
  TsMonorepo {
    #[command(flatten)]
    root_package: Option<RootPackage>,
    #[command(flatten)]
    boolean_flags: RepoBooleanFlags,
  },

  /// Generates a new typescript package
  TsPackage {
    name: String,
    #[arg(long, conflicts_with = "PackageConfig")]
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
    #[arg(short, long)]
    install: bool,
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

#[test]
fn verify_cli() {
  use clap::CommandFactory;
  Cli::command().debug_assert();
}
