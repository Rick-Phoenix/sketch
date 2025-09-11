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

  println!("{}", abs_path.display());

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

#[tokio::test]
async fn ts_repo_gen() -> Result<(), GenError> {
  let config = Config::from_file(PathBuf::from("sketch.toml"))?;

  config.create_ts_monorepo().await
}

#[tokio::test]
async fn circular_configs() -> Result<(), GenError> {
  let config = Config::from_file(PathBuf::from("tests/circular_configs/sketch.toml"));

  match config {
    Ok(_) => panic!("Circular configs test did not fail as expected"),
    Err(e) => {
      if matches!(e, GenError::CircularDependency(_)) {
        Ok(())
      } else {
        panic!("Circular configs test returned wrong kind of error")
      }
    }
  }
}
