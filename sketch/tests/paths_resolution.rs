use std::path::PathBuf;

use serde_json::Value;
use sketch_it::{config::Config, GenError};

#[tokio::test]
async fn paths_resolution() -> Result<(), GenError> {
  let mut config = Config::from_file(PathBuf::from("tests/paths_resolution/root_config.toml"))?;

  println!("{:#?}", config);

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
async fn repo_test() -> Result<(), GenError> {
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
