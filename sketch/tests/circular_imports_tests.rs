use std::path::PathBuf;

use sketch_it::{config::Config, ts::package::PackageData, GenError};

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

#[tokio::test]
async fn circular_package_json() -> Result<(), GenError> {
  let config = Config::from_file(PathBuf::from("tests/circular_package_json/sketch.toml"))?;

  let result = config
    .build_package(
      PackageData::Preset("circular_package_json".to_string()),
      "tests/output/circular_configs".into(),
      None,
      None,
    )
    .await;

  match result {
    Ok(_) => panic!("Circular package json test did not fail as expected"),
    Err(e) => {
      if matches!(e, GenError::CircularDependency(_)) {
        Ok(())
      } else {
        panic!("Circular package json test returned wrong kind of error")
      }
    }
  }
}

#[tokio::test]
async fn circular_tsconfig() -> Result<(), GenError> {
  let config = Config::from_file(PathBuf::from("tests/circular_tsconfigs/sketch.toml"))?;

  let result = config
    .build_package(
      PackageData::Preset("circular_tsconfigs".to_string()),
      "tests/output/circular_configs".into(),
      None,
      None,
    )
    .await;

  match result {
    Ok(_) => panic!("Circular tsconfig test did not fail as expected"),
    Err(e) => {
      if matches!(e, GenError::CircularDependency(_)) {
        Ok(())
      } else {
        panic!("Circular tsconfig test returned wrong kind of error: {}", e)
      }
    }
  }
}
