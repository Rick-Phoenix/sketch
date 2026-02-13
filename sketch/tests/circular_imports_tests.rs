use std::path::PathBuf;

use sketch_it::{
	AppError,
	config::Config,
	ts::package::{PackageType, TsPackagePresetRef, TsPackageSetup},
};

#[tokio::test]
async fn circular_configs() -> Result<(), AppError> {
	let config = Config::from_file(PathBuf::from("tests/circular_configs/sketch.toml"));

	match config {
		Ok(_) => panic!("Circular configs test did not fail as expected"),
		Err(e) => {
			if matches!(e, AppError::CircularDependency(_)) {
				Ok(())
			} else {
				panic!("Circular configs test returned wrong kind of error")
			}
		}
	}
}

#[tokio::test]
async fn circular_package_json() -> Result<(), AppError> {
	let config = Config::from_file(PathBuf::from("tests/circular_package_json/sketch.toml"))?;

	let result = config
		.create_ts_package(TsPackageSetup {
			data: TsPackagePresetRef::Preset("circular_package_json".to_string()),
			pkg_root: &PathBuf::from("tests/output/circular_configs"),
			tsconfig_files_to_update: vec![],
			cli_vars: &Default::default(),
			package_type: PackageType::Normal,
			install: false,
		})
		.await;

	match result {
		Ok(()) => panic!("Circular package json test did not fail as expected"),
		Err(e) => {
			if matches!(e, AppError::CircularDependency(_)) {
				Ok(())
			} else {
				panic!("Circular package json test returned wrong kind of error")
			}
		}
	}
}

#[tokio::test]
async fn circular_tsconfig() -> Result<(), AppError> {
	let config = Config::from_file(PathBuf::from("tests/circular_tsconfigs/sketch.toml"))?;

	let result = config
		.create_ts_package(TsPackageSetup {
			data: TsPackagePresetRef::Preset("circular_tsconfigs".to_string()),
			pkg_root: &PathBuf::from("tests/output/circular_configs"),
			tsconfig_files_to_update: vec![],
			cli_vars: &Default::default(),
			package_type: PackageType::Normal,
			install: false,
		})
		.await;

	match result {
		Ok(()) => panic!("Circular tsconfig test did not fail as expected"),
		Err(e) => {
			if matches!(e, AppError::CircularDependency(_)) {
				Ok(())
			} else {
				panic!("Circular tsconfig test returned wrong kind of error: {e}")
			}
		}
	}
}
