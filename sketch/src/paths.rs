use std::{
  env::current_dir,
  path::{Path, PathBuf},
};

use crate::GenError;

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
      path: base.to_path_buf(),
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
