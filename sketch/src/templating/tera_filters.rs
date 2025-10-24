use std::{collections::HashMap, path::PathBuf};

use convert_case::{Case, Casing};
use globset::Glob;
use regex::Regex;
use semver::{Version, VersionReq};
use tera::{Error, Map, Value};
use walkdir::WalkDir;

use crate::fs::{get_abs_path, get_relative_path};

fn extract_string<'a>(filter_name: &'a str, value: &'a Value) -> Result<&'a str, Error> {
  value.as_str().ok_or(Error::call_filter(
    filter_name,
    format!("Value `{}` is not a string", value),
  ))
}

fn extract_string_arg<'a>(
  filter_name: &'a str,
  arg_name: &str,
  args: &'a HashMap<String, Value>,
) -> Result<&'a str, Error> {
  extract_string(
    filter_name,
    args.get(arg_name).ok_or(Error::call_filter(
      filter_name,
      format!("Required argument `{}` is missing", arg_name),
    ))?,
  )
}

pub(crate) fn to_toml(value: &Value, _: &HashMap<String, Value>) -> Result<Value, Error> {
  let output = toml::to_string_pretty(value)
    .map_err(|e| Error::call_filter("to_toml", format!("Could not serialize to toml: {e}")))?;

  Ok(output.into())
}

pub(crate) fn to_yaml(value: &Value, _: &HashMap<String, Value>) -> Result<Value, Error> {
  let output = serde_yaml_ng::to_string(value)
    .map_err(|e| Error::call_filter("to_yaml", format!("Could not serialize to yaml: {e}")))?;

  Ok(output.into())
}

pub(crate) fn strip_prefix(text: &Value, args: &HashMap<String, Value>) -> Result<Value, Error> {
  let text = extract_string("strip_prefix", text)?;

  let prefix = extract_string_arg("strip_prefix", "prefix", args)?;

  Ok(text.strip_prefix(prefix).unwrap_or(text).into())
}

pub(crate) fn strip_suffix(text: &Value, args: &HashMap<String, Value>) -> Result<Value, Error> {
  let text = extract_string("strip_suffix", text)?;

  let suffix = extract_string_arg("strip_suffix", "suffix", args)?;

  Ok(text.strip_suffix(suffix).unwrap_or(text).into())
}

pub(crate) fn glob(dir: &Value, args: &HashMap<String, Value>) -> Result<Value, Error> {
  let dir = PathBuf::from(extract_string("glob", dir)?);

  let glob_pattern = extract_string_arg("glob", "pattern", args)?;

  let mut files: Vec<String> = Vec::new();

  let globset = Glob::new(glob_pattern)
    .map_err(|e| {
      Error::call_filter(
        "glob",
        format!("Invalid glob pattern error for `{}`: {}", glob_pattern, e),
      )
    })?
    .compile_matcher();

  for entry in WalkDir::new(&dir)
    .into_iter()
    .filter_map(|e| e.ok())
    .filter(|e| e.file_type().is_file())
  {
    let path = entry.path().strip_prefix(&dir).unwrap();
    if globset.is_match(path) {
      files.push(path.to_string_lossy().to_string());
    }
  }

  Ok(files.into())
}

pub(crate) fn read_dir(path: &Value, _: &HashMap<String, Value>) -> Result<Value, Error> {
  let path = PathBuf::from(extract_string("read_dir", path)?);

  let mut files: Vec<String> = Vec::new();

  for entry in WalkDir::new(&path)
    .into_iter()
    .filter_map(|e| e.ok())
    .filter(|e| e.file_type().is_file())
  {
    files.push(
      entry
        .path()
        .strip_prefix(&path)
        .unwrap()
        .to_string_lossy()
        .to_string(),
    );
  }

  Ok(files.into())
}

pub(crate) fn matches_glob(path: &Value, args: &HashMap<String, Value>) -> Result<Value, Error> {
  let path = PathBuf::from(extract_string("matches_glob", path)?);

  let glob_pattern = extract_string_arg("matches_glob", "pattern", args)?;

  let globset = Glob::new(glob_pattern)
    .map_err(|e| {
      Error::call_filter(
        "matches_glob",
        format!("Invalid glob pattern error for `{}`: {}", glob_pattern, e),
      )
    })?
    .compile_matcher();

  Ok(globset.is_match(path).into())
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

  let starting_path: PathBuf = extract_string_arg("relative", "from", args)?.into();

  let relative_path =
    get_relative_path(&starting_path, &path).map_err(|e| Error::call_filter("relative", e))?;

  Ok(relative_path.to_string_lossy().to_string().into())
}

pub(crate) fn semver(text: &Value, _: &HashMap<String, Value>) -> Result<Value, Error> {
  let mut text = extract_string("semver", text)?;

  text = text.strip_prefix('v').unwrap_or(text);

  let version = Version::parse(text).map_err(|e| {
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

  let version = Version::parse(text).map_err(|e| {
    Error::call_filter(
      "matches_semver",
      format!("Could not parse `{}` as a semver: {}", text, e),
    )
  })?;

  let mut target_version_text = extract_string(
    "matches_semver",
    args.get("target").ok_or(Error::call_filter(
      "matches_semver",
      "Could not find the `target` argument",
    ))?,
  )?;

  target_version_text = target_version_text
    .strip_prefix('v')
    .unwrap_or(target_version_text);

  let target_version = VersionReq::parse(target_version_text).map_err(|e| {
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
  let pattern = extract_string_arg("capture", "regex", args)?;

  let regex = Regex::new(pattern).map_err(|e| {
    Error::call_filter(
      "capture",
      format!("Regex creation error for `{}`: {}", pattern, e),
    )
  })?;

  let text = extract_string("capture", text)?;

  let mut captured_groups: Map<String, Value> = Map::new();

  if let Some(caps) = regex.captures(text) {
    for name in regex.capture_names().flatten() {
      if let Some(captured_text) = caps.name(name) {
        captured_groups.insert(name.to_string(), captured_text.as_str().to_string().into());
      }
    }
  }

  Ok(captured_groups.into())
}

pub(crate) fn capture_many(text: &Value, args: &HashMap<String, Value>) -> Result<Value, Error> {
  let pattern = extract_string_arg("capture_many", "regex", args)?;

  let regex = Regex::new(pattern).map_err(|e| {
    Error::call_filter(
      "capture_many",
      format!("Regex creation error for `{}`: {}", pattern, e),
    )
  })?;

  let text = extract_string("capture_many", text)?;

  let mut all_captures: Vec<Map<String, Value>> = Vec::new();

  for cap in regex.captures_iter(text) {
    let mut captured_groups: Map<String, Value> = Map::new();

    for name in regex.capture_names().flatten() {
      if let Some(captured_text) = cap.name(name) {
        captured_groups.insert(name.to_string(), captured_text.as_str().to_string().into());
      }
    }

    if !captured_groups.is_empty() {
      all_captures.push(captured_groups);
    }
  }

  Ok(all_captures.into())
}
