use std::{collections::HashMap, path::PathBuf};

use regex::Regex;
use tera::{Error, Map, Value};

fn extract_string<'a>(filter_name: &'a str, value: &'a Value) -> Result<&'a str, Error> {
  value.as_str().ok_or(Error::call_filter(
    filter_name,
    format!("Value `{}` is not a string", value.to_string()),
  ))
}

pub(crate) fn is_file(path: &Value, _: &HashMap<String, Value>) -> Result<Value, Error> {
  let path = PathBuf::from(extract_string("is_file", path)?);

  Ok(path.is_file().into())
}

pub(crate) fn is_dir(path: &Value, _: &HashMap<String, Value>) -> Result<Value, Error> {
  let path = PathBuf::from(extract_string("is_dir", path)?);

  Ok(path.is_dir().into())
}

pub(crate) fn basename(path: &Value, _: &HashMap<String, Value>) -> Result<Value, Error> {
  let path = PathBuf::from(extract_string("basename", path)?);

  match path.file_name() {
    Some(basename) => Ok(Value::String(basename.to_string_lossy().to_string())),
    None => Err(Error::call_filter(
      "basename",
      format!("Could not get the basename for `{}`", path.display()),
    )),
  }
}

pub(crate) fn parent_dir(path: &Value, _: &HashMap<String, Value>) -> Result<Value, Error> {
  let path = PathBuf::from(extract_string("parent_dir", path)?);

  match path.parent() {
    Some(parent) => Ok(Value::String(parent.to_string_lossy().to_string())),
    None => Err(Error::call_filter(
      "parent_dir",
      format!("Could not get the parent dir for `{}`", path.display()),
    )),
  }
}

pub(crate) fn capture(text: &Value, args: &HashMap<String, Value>) -> Result<Value, Error> {
  let pattern = extract_string(
    "capture",
    args.get("regex").ok_or(Error::call_filter(
      "capture",
      format!("Could not find the `regex` argument"),
    ))?,
  )?;

  let regex = Regex::new(&pattern.to_string()).map_err(|e| {
    Error::call_filter(
      "capture",
      format!("Regex creation error for `{}`: {}", pattern, e),
    )
  })?;

  let text = extract_string("capture", text)?;

  let mut captured_groups: Map<String, Value> = Map::new();

  if let Some(caps) = regex.captures(&text) {
    for group in regex.capture_names() {
      if let Some(name) = group {
        if let Some(captured_text) = caps.name(name) {
          captured_groups.insert(name.to_string(), captured_text.as_str().to_string().into());
        }
      }
    }
  }

  Ok(captured_groups.into())
}
