use std::path::PathBuf;

use tera::{Error, Value};

pub(crate) fn is_file(path: Option<&Value>, _: &[Value]) -> Result<bool, Error> {
  let path = path
    .ok_or(Error::call_filter(
      "is_file",
      format!("Cannot establish if `None` is a file"),
    ))?
    .to_string();

  Ok(PathBuf::from(path).is_file())
}

pub(crate) fn is_dir(path: Option<&Value>, _: &[Value]) -> Result<bool, Error> {
  let path = path
    .ok_or(Error::call_filter(
      "is_dir",
      format!("Cannot establish if `None` is a directory"),
    ))?
    .to_string();

  Ok(PathBuf::from(path).is_dir())
}
