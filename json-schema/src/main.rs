use std::{
  fs::{create_dir_all, read_to_string, File},
  path::PathBuf,
};

use clap::Parser;
use regex::Regex;
use schemars::schema_for;
use sketch_it::Config;

#[derive(Debug, Parser)]
pub(crate) struct SchemaCmd {
  pub(crate) version: String,
}

fn get_version(v: &str) -> (usize, usize, usize) {
  let version_regex = Regex::new(r"[v]?(\d)\.(\d)\.(\d)\w*").unwrap();

  let captures = version_regex.captures(v).unwrap();

  let major = captures[1].parse::<usize>().unwrap();
  let minor = captures[2].parse::<usize>().unwrap();
  let patch = captures[3].parse::<usize>().unwrap();

  (major, minor, patch)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let schemas_dir = PathBuf::from("schemas");

  create_dir_all(&schemas_dir)?;

  let schema = schema_for!(Config);

  let version = SchemaCmd::parse().version;

  let latest_schema_file = schemas_dir.join("latest.json");

  if latest_schema_file.is_file() {
    let schema_str: String = serde_json::to_string_pretty(&schema)?;
    let latest_schema_content = read_to_string(&latest_schema_file)?;

    if latest_schema_content == schema_str {
      return Ok(());
    }
  }

  let latest = File::create(latest_schema_file)?;
  serde_json::to_writer_pretty(&latest, &schema)?;

  if !version.starts_with('v') {
    let (major, minor, _) = get_version(&version);

    let minor_dir = schemas_dir.join(format!("v{}.{}", major, minor));

    create_dir_all(&minor_dir)?;

    let versioned = File::create(minor_dir.join(format!("v{}.json", version)))?;
    serde_json::to_writer_pretty(&versioned, &schema)?;
  }

  Ok(())
}
