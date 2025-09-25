use std::{collections::HashMap, path::PathBuf};

use convert_case::{Case, Casing};
use regex::Regex;
use semver::{Version, VersionReq};
use tera::{Error, Map, Value};

use crate::fs::{get_abs_path, get_relative_path};

fn extract_string<'a>(filter_name: &'a str, value: &'a Value) -> Result<&'a str, Error> {
  value.as_str().ok_or(Error::call_filter(
    filter_name,
    format!("Value `{}` is not a string", value.to_string()),
  ))
}

pub(crate) fn is_relative(path: &Value, _: &HashMap<String, Value>) -> Result<Value, Error> {
  let path = PathBuf::from(extract_string("is_relative", path)?);

  Ok(path.is_relative().into())
}

pub(crate) fn is_absolute(path: &Value, _: &HashMap<String, Value>) -> Result<Value, Error> {
  let path = PathBuf::from(extract_string("is_absolute", path)?);

  Ok(path.is_absolute().into())
}

pub(crate) fn absolute(path: &Value, _: &HashMap<String, Value>) -> Result<Value, Error> {
  let path = PathBuf::from(extract_string("absolute", path)?);

  let abs_path = get_abs_path(&path).map_err(|e| Error::call_filter("absolute", e))?;

  Ok(abs_path.to_string_lossy().to_string().into())
}

pub(crate) fn relative(path: &Value, args: &HashMap<String, Value>) -> Result<Value, Error> {
  let path = PathBuf::from(extract_string("relative", path)?);

  let starting_path: PathBuf = extract_string(
    "relative",
    args.get("from").ok_or(Error::call_filter(
      "relative",
      format!("Could not find the `from` argument"),
    ))?,
  )?
  .into();

  let relative_path =
    get_relative_path(&starting_path, &path).map_err(|e| Error::call_filter("relative", e))?;

  Ok(relative_path.to_string_lossy().to_string().into())
}

pub(crate) fn semver(text: &Value, _: &HashMap<String, Value>) -> Result<Value, Error> {
  let mut text = extract_string("semver", text)?;

  text = text.strip_prefix('v').unwrap_or(text);

  let version = Version::parse(&text).map_err(|e| {
    Error::call_filter(
      "semver",
      format!("Could not parse `{}` as a semver: {}", text, e),
    )
  })?;

  let mut data: Map<String, Value> = Map::new();

  data.insert("major".to_string(), version.major.into());
  data.insert("minor".to_string(), version.minor.into());
  data.insert("patch".to_string(), version.patch.into());

  Ok(data.into())
}

pub(crate) fn matches_semver(text: &Value, args: &HashMap<String, Value>) -> Result<Value, Error> {
  let mut text = extract_string("matches_semver", text)?;

  text = text.strip_prefix('v').unwrap_or(text);

  let version = Version::parse(&text).map_err(|e| {
    Error::call_filter(
      "matches_semver",
      format!("Could not parse `{}` as a semver: {}", text, e),
    )
  })?;

  let mut target_version_text = extract_string(
    "matches_semver",
    args.get("target").ok_or(Error::call_filter(
      "matches_semver",
      format!("Could not find the `target` argument"),
    ))?,
  )?;

  target_version_text = target_version_text
    .strip_prefix('v')
    .unwrap_or(target_version_text);

  let target_version = VersionReq::parse(&target_version_text).map_err(|e| {
    Error::call_filter(
      "matches_semver",
      format!(
        "Could not parse `{}` as a semver: {}",
        target_version_text, e
      ),
    )
  })?;

  Ok(target_version.matches(&version).into())
}

pub(crate) fn camel(text: &Value, _: &HashMap<String, Value>) -> Result<Value, Error> {
  let text = extract_string("camel", text)?;

  let converted = text.to_case(Case::Camel);

  Ok(converted.into())
}

pub(crate) fn snake(text: &Value, _: &HashMap<String, Value>) -> Result<Value, Error> {
  let text = extract_string("snake", text)?;

  let converted = text.to_case(Case::Snake);

  Ok(converted.into())
}

pub(crate) fn upper_snake(text: &Value, _: &HashMap<String, Value>) -> Result<Value, Error> {
  let text = extract_string("upper_snake", text)?;

  let converted = text.to_case(Case::UpperSnake);

  Ok(converted.into())
}

pub(crate) fn pascal(text: &Value, _: &HashMap<String, Value>) -> Result<Value, Error> {
  let text = extract_string("pascal", text)?;

  let converted = text.to_case(Case::Pascal);

  Ok(converted.into())
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

pub(crate) fn capture_many(text: &Value, args: &HashMap<String, Value>) -> Result<Value, Error> {
  let pattern = extract_string(
    "capture_many",
    args.get("regex").ok_or(Error::call_filter(
      "capture_many",
      format!("Could not find the `regex` argument"),
    ))?,
  )?;

  let regex = Regex::new(&pattern.to_string()).map_err(|e| {
    Error::call_filter(
      "capture_many",
      format!("Regex creation error for `{}`: {}", pattern, e),
    )
  })?;

  let text = extract_string("capture_many", text)?;

  let mut all_captures: Vec<Map<String, Value>> = Vec::new();

  for cap in regex.captures_iter(&text) {
    let mut captured_groups: Map<String, Value> = Map::new();

    for group in regex.capture_names() {
      if let Some(name) = group {
        if let Some(captured_text) = cap.name(name) {
          captured_groups.insert(name.to_string(), captured_text.as_str().to_string().into());
        }
      }
    }

    if !captured_groups.is_empty() {
      all_captures.push(captured_groups);
    }
  }

  Ok(all_captures.into())
}
