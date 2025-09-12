use std::path::PathBuf;

use serde_json::Value;
use sketch_it::{config::Config, GenError};

#[tokio::test]
async fn root_dir_resolution() -> Result<(), GenError> {
  let config_file_path = PathBuf::from("tests/paths_resolution/root_config.toml");
  let config = Config::from_file(&config_file_path)?;

  let abs_path = config_file_path
    .parent()
    .unwrap()
    .join("../output")
    .canonicalize()
    .unwrap();

  assert_eq!(config.root_dir.expect("Missing root dir"), abs_path);

  Ok(())
}

#[tokio::test]
async fn config_files_resolution() -> Result<(), GenError> {
  let mut config = Config::from_file(PathBuf::from("tests/paths_resolution/root_config.toml"))?;

  assert_eq!(
    config
      .global_templates_vars
      .shift_remove("hello")
      .expect("Error in paths resolution test"),
    Value::String("there".to_string())
  );

  assert_eq!(
    config
      .global_templates_vars
      .shift_remove("general")
      .expect("Error in paths resolution test"),
    Value::String("kenobi".to_string())
  );

  Ok(())
}
