macro_rules! path_to_str {
  ($path:expr) => {
    &$path.to_string_lossy().to_string()
  };
}

macro_rules! get_bin {
  () => {
    assert_cmd::Command::cargo_bin("sketch").expect("Failed to find the app binary")
  };
}
