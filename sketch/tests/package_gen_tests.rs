use std::path::PathBuf;

use sketch_it::{
  config::Config,
  package::{PackageData, PackageDataKind},
  GenError,
};

#[tokio::test]
async fn package_test() -> Result<(), GenError> {
  let config = Config::from_file(PathBuf::from("sketch.toml"))?;

  config
    .build_package(PackageData {
      name: None,
      kind: PackageDataKind::Preset("alt2".to_string()),
    })
    .await
}

#[tokio::test]
async fn circular_package_json() -> Result<(), GenError> {
  let config = Config::from_file(PathBuf::from("tests/circular_package_json/sketch.toml"))?;

  let result = config
    .build_package(PackageData {
      name: None,
      kind: PackageDataKind::Preset("circular_package_json".to_string()),
    })
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
    .build_package(PackageData {
      name: None,
      kind: PackageDataKind::Preset("circular_tsconfigs".to_string()),
    })
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
