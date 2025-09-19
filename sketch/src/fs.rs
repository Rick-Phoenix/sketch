use std::{
  env::current_dir,
  ffi::OsStr,
  fs::{create_dir_all, read_to_string, File},
  io::Write,
  path::{Path, PathBuf},
};

use serde::{de::DeserializeOwned, Serialize};

use crate::GenError;

pub fn get_extension(file: &Path) -> &OsStr {
  file
    .extension()
    .unwrap_or_else(|| panic!("File `{}` has no extension", file.display()))
}

pub fn serialize_toml<T: Serialize>(item: &T, path: &Path) -> Result<(), GenError> {
  let mut output_file = open_file_for_writing(path)?;

  let content = toml::to_string_pretty(item).map_err(|e| GenError::SerializationError {
    file: path.to_path_buf(),
    error: e.to_string(),
  })?;

  output_file
    .write_all(&content.as_bytes())
    .map_err(|e| GenError::WriteError {
      path: path.to_path_buf(),
      source: e,
    })?;

  Ok(())
}

pub fn serialize_yaml<T: Serialize>(item: &T, path: &Path) -> Result<(), GenError> {
  let output_file = open_file_for_writing(path)?;

  serde_yaml_ng::to_writer(output_file, item).map_err(|e| GenError::SerializationError {
    file: path.to_path_buf(),
    error: e.to_string(),
  })
}

pub fn serialize_json<T: Serialize>(item: &T, path: &Path) -> Result<(), GenError> {
  let output_file = open_file_for_writing(path)?;

  serde_json::to_writer_pretty(output_file, item).map_err(|e| GenError::SerializationError {
    file: path.to_path_buf(),
    error: e.to_string(),
  })
}

pub fn deserialize_toml<T: DeserializeOwned>(path: &Path) -> Result<T, GenError> {
  let contents = read_to_string(path).map_err(|e| GenError::ReadError {
    path: path.to_path_buf(),
    source: e,
  })?;

  toml::from_str(&contents).map_err(|e| GenError::DeserializationError {
    file: path.to_path_buf(),
    error: e.to_string(),
  })
}

pub fn deserialize_json<T: DeserializeOwned>(path: &Path) -> Result<T, GenError> {
  let file = open_file(path)?;

  serde_json::from_reader(file).map_err(|e| GenError::DeserializationError {
    file: path.to_path_buf(),
    error: e.to_string(),
  })
}

pub fn deserialize_yaml<T: DeserializeOwned>(path: &Path) -> Result<T, GenError> {
  let file = open_file(path)?;

  serde_yaml_ng::from_reader(file).map_err(|e| GenError::DeserializationError {
    file: path.to_path_buf(),
    error: e.to_string(),
  })
}

pub fn open_file(path: &Path) -> Result<File, GenError> {
  File::open(path).map_err(|e| GenError::ReadError {
    path: path.to_path_buf(),
    source: e,
  })
}

pub fn open_file_for_writing(path: &Path) -> Result<File, GenError> {
  File::create(path).map_err(|e| GenError::WriteError {
    path: path.to_path_buf(),
    source: e,
  })
}

pub fn open_file_if_overwriting(no_overwrite: bool, path: &Path) -> Result<File, GenError> {
  if !no_overwrite {
    File::create(&path).map_err(|e| GenError::WriteError {
      path: path.to_path_buf(),
      source: e,
    })
  } else {
    File::create_new(&path).map_err(|e| match e.kind() {
      std::io::ErrorKind::AlreadyExists => GenError::Custom(format!(
        "The file `{}` already exists. Set `no_overwrite` to false to overwrite existing files",
        path.display()
      )),
      _ => GenError::WriteError {
        path: path.to_path_buf(),
        source: e,
      },
    })
  }
}

pub(crate) fn create_all_dirs(path: &Path) -> Result<(), GenError> {
  create_dir_all(path).map_err(|e| {
    GenError::Custom(format!(
      "Could not create the parent dirs for `{}`: {}",
      path.display(),
      e
    ))
  })
}

pub(crate) fn get_abs_path(path: &Path) -> Result<PathBuf, GenError> {
  path
    .canonicalize()
    .map_err(|e| GenError::PathCanonicalization {
      path: path.into(),
      source: e,
    })
}

pub(crate) fn get_parent_dir(path: &Path) -> &Path {
  path
    .parent()
    .unwrap_or_else(|| panic!("Could not get the parent directory of '{}'", path.display()))
}

pub(crate) fn get_cwd() -> PathBuf {
  current_dir().expect("Could not get the cwd")
}

pub(crate) fn get_relative_path(base: &Path, target: &Path) -> Result<PathBuf, GenError> {
  let canonical_base = base
    .canonicalize()
    .map_err(|e| GenError::PathCanonicalization {
      path: base.to_path_buf(),
      source: e,
    })?;

  let canonical_target = target
    .canonicalize()
    .map_err(|e| GenError::PathCanonicalization {
      path: target.to_path_buf(),
      source: e,
    })?;

  let base_components: Vec<_> = canonical_base.components().collect();
  let target_components: Vec<_> = canonical_target.components().collect();

  let mut common_ancestor_len = 0;
  for (a, b) in base_components.iter().zip(target_components.iter()) {
    if a == b {
      common_ancestor_len += 1;
    } else {
      break;
    }
  }

  let mut relative_path = PathBuf::new();

  for _ in common_ancestor_len..base_components.len() {
    relative_path.push("..");
  }

  for component in target_components.iter().skip(common_ancestor_len) {
    relative_path.push(component);
  }

  Ok(relative_path)
}
