#![allow(clippy::result_large_err)]

use clap::{error::ErrorKind, CommandFactory, Parser};
mod cli;
use figment::providers::{Format, Toml, Yaml};
use scaffolder::{config::Config, GenError};

use crate::cli::Cli;

#[tokio::main]
async fn main() -> Result<(), GenError> {
  let cli = Cli::parse();

  let mut config_figment = Config::figment();

  if let Some(config_path) = cli.config.as_deref() {
    if config_path.ends_with(".yaml") {
      config_figment = config_figment.merge(Yaml::file(config_path));
    } else if config_path.ends_with(".toml") {
      config_figment = config_figment.merge(Toml::file(config_path));
    } else {
      let mut cmd = Cli::command();
      cmd
        .error(
          ErrorKind::InvalidValue,
          "Unrecognized configuration format. Allowed formats are: yaml, toml",
        )
        .exit();
    }
  }

  let config: Config = config_figment
    .extract()
    .map_err(|e| GenError::ConfigParsing { source: e })?;

  match cli.command {
    cli::Commands::Repo { .. } => {
      config.build_repo().await?;
    }
    cli::Commands::Package { name } => {
      config.build_package(&name).await?;
    }
    cli::Commands::Render { name } => todo!(),
    cli::Commands::Init => todo!(),
  };

  Ok(())
}
