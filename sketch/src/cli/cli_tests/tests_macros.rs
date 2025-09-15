macro_rules! assert_dir_exists {
  ($path:expr) => {
    assert!($path.exists() && $path.is_dir())
  };
}

macro_rules! extract_tsconfig {
  ($path:expr) => {
    deserialize_json!(TsConfig, $path)
  };
}

macro_rules! extract_package_json {
  ($path:expr) => {
    deserialize_json!(PackageJson, $path)
  };
}

macro_rules! deserialize_json {
  ($ty:ty, $path:expr) => {{
    let file = File::open(PathBuf::from(&$path))
      .unwrap_or_else(|e| panic!("Failed to open {}: {}", $path.display(), e));
    let data: $ty = serde_json::from_reader(&file)
      .unwrap_or_else(|e| panic!("Failed to deserialize {}: {}", $path.display(), e));
    data
  }};
}

macro_rules! deserialize_yaml {
  ($ty:ty, $path:expr) => {{
    let file = File::open(PathBuf::from(&$path))
      .unwrap_or_else(|e| panic!("Failed to open {}: {}", $path.display(), e));
    let data: $ty = serde_yaml_ng::from_reader(&file)
      .unwrap_or_else(|e| panic!("Failed to deserialize {}: {}", $path.display(), e));
    data
  }};
}

macro_rules! get_bin {
  () => {
    assert_cmd::Command::cargo_bin("sketch").expect("Failed to find the app binary")
  };
}
