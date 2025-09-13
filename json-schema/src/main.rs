use std::{
  fs::{create_dir_all, File},
  path::PathBuf,
};

use clap::Parser;
use schemars::schema_for;
use sketch_it::Config;

#[derive(Debug, Parser)]
pub(crate) struct SchemaCmd {
  pub(crate) version: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let schemas_dir = PathBuf::from("schemas");

  create_dir_all(&schemas_dir)?;

  let version = SchemaCmd::parse().version;

  let schema = schema_for!(Config);
  let versioned = File::create(schemas_dir.join(format!("{}.json", version)))?;
  serde_json::to_writer_pretty(&versioned, &schema)?;

  let latest = File::create(schemas_dir.join("latest.json"))?;
  serde_json::to_writer_pretty(&latest, &schema)?;

  Ok(())
}
