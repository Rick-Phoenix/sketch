use std::{
  collections::{BTreeMap, BTreeSet},
  hash::Hash,
};

use indexmap::{IndexMap, IndexSet};
use merge::Merge;

use crate::{GenError, Preset};

pub trait Extensible {
  fn get_extended(&self) -> &IndexSet<String>;
}

fn process_preset_id(
  id: &str,
  processed_ids: &mut IndexSet<String>,
  preset_kind: Preset,
) -> Result<(), GenError> {
  let was_absent = processed_ids.insert(id.to_string());

  if !was_absent {
    let chain: Vec<&str> = processed_ids.iter().map(|s| s.as_str()).collect();

    return Err(GenError::CircularDependency(format!(
      "Found circular {:?} dependency for '{}'. The full processed chain is: {}",
      preset_kind,
      id,
      chain.join(" -> ")
    )));
  }

  Ok(())
}

pub(crate) fn merge_presets<T: Merge + Extensible + Default + Clone>(
  preset_kind: Preset,
  current_id: &str,
  preset: T,
  store: &IndexMap<String, T>,
  processed_ids: &mut IndexSet<String>,
) -> Result<T, GenError> {
  process_preset_id(current_id, processed_ids, Preset::PackageJson)?;

  let presets_to_extend = preset.get_extended();

  if presets_to_extend.is_empty() {
    return Ok(preset);
  }

  let mut base: Option<T> = None;

  for id in presets_to_extend {
    let extend_target = store
      .get(id)
      .ok_or(GenError::PresetNotFound {
        kind: Preset::PackageJson,
        name: id.to_string(),
      })?
      .clone();

    let complete_target = merge_presets(preset_kind, id, extend_target, store, processed_ids)?;

    if let Some(aggregated) = base.as_mut() {
      aggregated.merge(complete_target);
    } else {
      base = Some(complete_target)
    }
  }

  // Can never be none due to the early exit
  let mut aggregated = base.unwrap();

  aggregated.merge(preset);

  Ok(aggregated)
}

pub(crate) fn merge_nested<T: Merge>(left: &mut T, right: T) {
  left.merge(right)
}

pub(crate) fn merge_optional_nested<T: Merge>(left: &mut Option<T>, right: Option<T>) {
  if let Some(right_data) = right {
    if let Some(left_data) = left {
      left_data.merge(right_data);
    } else {
      *left = Some(right_data);
    }
  }
}

pub(crate) fn merge_btree_maps<T>(left: &mut BTreeMap<String, T>, right: BTreeMap<String, T>) {
  left.extend(right)
}

pub(crate) fn merge_btree_sets<T>(left: &mut BTreeSet<T>, right: BTreeSet<T>)
where
  T: Ord,
{
  left.extend(right)
}

pub(crate) fn merge_index_maps<T>(left: &mut IndexMap<String, T>, right: IndexMap<String, T>) {
  left.extend(right)
}

pub(crate) fn merge_index_sets<T>(left: &mut IndexSet<T>, right: IndexSet<T>)
where
  T: Eq + Hash,
{
  left.extend(right)
}

pub(crate) fn merge_optional_index_sets<T>(
  left: &mut Option<IndexSet<T>>,
  right: Option<IndexSet<T>>,
) where
  T: Eq + Hash,
{
  if let Some(right_data) = right {
    if let Some(left_data) = left {
      for item in right_data.into_iter() {
        left_data.insert(item);
      }
    } else {
      *left = Some(right_data)
    }
  }
}

pub(crate) fn overwrite_if_some<T>(left: &mut Option<T>, right: Option<T>) {
  if let Some(new) = right {
    *left = Some(new)
  }
}

pub(crate) fn merge_optional_btree_sets<T>(
  left: &mut Option<BTreeSet<T>>,
  right: Option<BTreeSet<T>>,
) where
  T: Ord,
{
  if let Some(right_data) = right {
    if let Some(left_data) = left {
      left_data.extend(right_data);
    } else {
      *left = Some(right_data)
    }
  }
}

pub(crate) fn merge_optional_btree_maps<T>(
  left: &mut Option<BTreeMap<String, T>>,
  right: Option<BTreeMap<String, T>>,
) {
  if let Some(right_data) = right {
    if let Some(left_data) = left {
      for (key, val) in right_data.into_iter() {
        left_data.insert(key, val);
      }
    } else {
      *left = Some(right_data)
    }
  }
}

pub(crate) fn merge_optional_vecs<T>(left: &mut Option<Vec<T>>, right: Option<Vec<T>>) {
  if let Some(right_data) = right {
    if let Some(left_data) = left {
      left_data.extend(right_data);
    } else {
      *left = Some(right_data)
    }
  }
}
