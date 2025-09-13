use std::{fs::File, path::PathBuf};

use schemars::schema_for;
use sketch_it::Config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let schemas_dir = PathBuf::from("schemas");

  macro_rules! write_schema {
    ($name:ident) => {
      paste::paste! {
        {
          let schema = schema_for!($name);
          let output = File::create(schemas_dir.join(concat!(stringify!([< $name:snake >]), ".json")))?;
          serde_json::to_writer_pretty(&output, &schema)?;
        }
      }
    };
  }

  write_schema!(Config);

  Ok(())
}
