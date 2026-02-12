use std::path::PathBuf;

use serde_json::Value;
use sketch_it::{AppError, config::Config};

#[tokio::test]
async fn config_files_resolution() -> Result<(), AppError> {
	let mut config = Config::from_file(PathBuf::from("tests/paths_resolution/root_config.toml"))?;

	assert_eq!(
		config
			.vars
			.shift_remove("hello")
			.expect("Error in paths resolution test"),
		Value::String("there".to_string())
	);

	assert_eq!(
		config
			.vars
			.shift_remove("general")
			.expect("Error in paths resolution test"),
		Value::String("kenobi".to_string())
	);

	Ok(())
}
