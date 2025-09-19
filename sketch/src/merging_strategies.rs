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
