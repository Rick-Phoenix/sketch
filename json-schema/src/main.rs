use std::{
	fs::{File, create_dir_all},
	path::PathBuf,
};

use clap::Parser;
use regex::Regex;
use schemars::schema_for;
use sketch_it::Config;

#[derive(Debug, Parser)]
pub struct SchemaCmd {
	pub version: String,
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
	let schemas_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../schemas");

	create_dir_all(&schemas_dir)?;

	let schema = schema_for!(Config);

	let version = SchemaCmd::parse().version;

	if version == "development" {
		let file = File::create(schemas_dir.join("development.json"))?;
		serde_json::to_writer_pretty(&file, &schema)?;
		return Ok(());
	}

	let (major, minor, _) = get_version(&version);

	let new_schema_path = schemas_dir.join(format!("v{major}.{minor}.json"));

	let versioned = File::create(&new_schema_path)?;
	serde_json::to_writer_pretty(&versioned, &schema)?;

	eprintln!("Created new json schema in {}", new_schema_path.display());

	let latest_schema_file = schemas_dir.join("latest.json");

	let latest = File::create(latest_schema_file)?;
	serde_json::to_writer_pretty(&latest, &schema)?;

	Ok(())
}
