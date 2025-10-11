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

macro_rules! unwrap_variant {
  ($enm:ident, $variant:ident, $origin:expr) => {
    if let $enm::$variant(v) = $origin {
      v
    } else {
      unreachable!()
    }
  };
}
