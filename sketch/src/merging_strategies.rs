use std::{
  collections::{BTreeMap, BTreeSet},
  hash::Hash,
};

use indexmap::{IndexMap, IndexSet};
use merge::Merge;

pub(crate) fn is_default<T: Default + PartialEq>(v: &T) -> bool {
  v == &T::default()
}

pub(crate) fn merge_if_not_default<T: Default + PartialEq>(left: &mut T, right: T) {
  if !is_default(&right) {
    *left = right
  }
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

pub(crate) fn merge_nested<T: Merge>(left: &mut T, right: T) {
  left.merge(right);
}

pub(crate) fn overwrite_always<T>(left: &mut T, right: T) {
  *left = right
}

pub(crate) fn merge_btree_maps<T>(left: &mut BTreeMap<String, T>, right: BTreeMap<String, T>) {
  left.extend(right)
}

pub(crate) fn merge_index_maps<T>(left: &mut IndexMap<String, T>, right: IndexMap<String, T>) {
  left.extend(right)
}

pub(crate) fn merge_sets<T>(left: &mut BTreeSet<T>, right: BTreeSet<T>)
where
  T: Ord,
{
  left.extend(right)
}

pub(crate) fn merge_index_sets<T>(left: &mut IndexSet<T>, right: IndexSet<T>)
where
  T: Eq + Hash,
{
  left.extend(right)
}

pub(crate) fn overwrite_option<T>(left: &mut Option<T>, right: Option<T>) {
  if let Some(new) = right {
    *left = Some(new)
  }
}

pub(crate) fn merge_optional_vecs<T>(left: &mut Option<Vec<T>>, right: Option<Vec<T>>)
where
  T: PartialEq,
{
  if let Some(right_data) = right {
    if let Some(left_data) = left {
      for item in right_data {
        if !left_data.contains(&item) {
          left_data.push(item);
        }
      }
    } else {
      *left = Some(right_data)
    }
  }
}

pub(crate) fn merge_optional_maps<T>(
  left: &mut Option<BTreeMap<String, T>>,
  right: Option<BTreeMap<String, T>>,
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
