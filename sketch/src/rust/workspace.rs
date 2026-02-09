use super::*;
use std::{
	collections::{BTreeMap, BTreeSet},
	path::PathBuf,
};

use merge::Merge;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
	merge_btree_maps, merge_btree_sets, merge_optional_btree_maps, merge_optional_btree_sets,
	merge_optional_nested, overwrite_if_some,
	rust::{
		Dependency, Edition, Lint, LintLevel, OptionalFile, Publish, Resolver, merge_dependencies,
	},
};

/// A manifest can contain both a package and workspace-wide properties
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, JsonSchema, Merge)]
#[merge(strategy = overwrite_if_some)]
#[serde(rename_all = "kebab-case")]
pub struct Workspace {
	/// Relative paths of crates in here
	#[serde(default)]
	#[merge(strategy = merge_btree_sets)]
	pub members: BTreeSet<String>,

	/// Members to operate on when in the workspace root.
	///
	/// When specified, `default-members` must expand to a subset of `members`.
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	#[merge(strategy = merge_btree_sets)]
	pub default_members: BTreeSet<String>,

	/// Template for inheritance
	#[serde(skip_serializing_if = "Option::is_none")]
	#[merge(strategy = merge_optional_nested)]
	pub package: Option<PackageTemplate>,

	/// Ignore these dirs
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	#[merge(strategy = merge_btree_sets)]
	pub exclude: BTreeSet<String>,

	/// Shared info
	#[serde(skip_serializing_if = "BTreeMap::is_empty")]
	#[merge(strategy = merge_btree_maps)]
	pub metadata: BTreeMap<String, Value>,

	/// Compatibility setting
	#[serde(skip_serializing_if = "Option::is_none")]
	pub resolver: Option<Resolver>,

	/// Template for `needs_workspace_inheritance`
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	#[merge(strategy = merge_dependencies)]
	pub dependencies: BTreeMap<String, Dependency>,

	/// Workspace-level lint groups
	#[serde(default, skip_serializing_if = "Option::is_none")]
	#[merge(strategy = overwrite_if_some)]
	pub lints: Option<Lints>,
}

impl AsTomlValue for Workspace {
	fn as_toml_value(&self) -> Item {
		let mut table = Table::new();

		add_string_list!(self, table => members, default_members, exclude);

		add_value!(self, table => package, resolver, lints);

		table.into()
	}
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Merge)]
pub struct Lints {
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	#[merge(strategy = merge_btree_maps)]
	pub rust: BTreeMap<String, LintKind>,

	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	#[merge(strategy = merge_btree_maps)]
	pub clippy: BTreeMap<String, LintKind>,
}

impl AsTomlValue for Lints {
	fn as_toml_value(&self) -> Item {
		let mut table = Table::new();

		add_map!(self, table => rust, clippy);

		table.into()
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum LintKind {
	Single(LintLevel),
	Map(BTreeMap<String, Lint>),
}

impl AsTomlValue for LintKind {
	fn as_toml_value(&self) -> Item {
		match self {
			Self::Single(lev) => lev.as_toml_value().into(),
			Self::Map(map) => Table::from_iter(map.iter().map(|(k, v)| {
				(
					<&String as Into<toml_edit::Key>>::into(k),
					Item::from(v.as_toml_value()),
				)
			}))
			.into(),
		}
	}
}

/// Workspace can predefine properties that can be inherited via `{ workspace = true }` in its member packages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, JsonSchema, Merge)]
#[serde(rename_all = "kebab-case")]
#[merge(strategy = overwrite_if_some)]
#[non_exhaustive]
pub struct PackageTemplate {
	/// See <https://crates.io/category_slugs>
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	#[merge(strategy = merge_btree_sets)]
	pub categories: BTreeSet<String>,

	/// Multi-line text, some people use Markdown here
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,

	/// URL
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub documentation: Option<String>,

	/// Opt-in to new Rust behaviors
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub edition: Option<Edition>,

	/// Don't publish these files, relative to workspace
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	#[merge(strategy = merge_btree_sets)]
	pub exclude: BTreeSet<String>,

	/// Homepage URL
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub homepage: Option<String>,

	/// Publish these files, relative to workspace
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	#[merge(strategy = merge_btree_sets)]
	pub include: BTreeSet<String>,

	/// For search
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	#[merge(strategy = merge_btree_sets)]
	pub keywords: BTreeSet<String>,

	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub license: Option<String>,

	/// Block publishing or choose custom registries
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub publish: Option<Publish>,

	/// Opt-out or custom path, relative to workspace
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub readme: Option<OptionalFile>,

	/// (HTTPS) repository URL
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub repository: Option<String>,

	/// Minimum required rustc version in format `1.99`
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub rust_version: Option<String>,

	/// Package version semver
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub version: Option<String>,
}

impl AsTomlValue for PackageTemplate {
	fn as_toml_value(&self) -> Item {
		let mut table = Table::new();

		macro_rules! add_string_local {
			($($name:ident),*) => {
				add_string!(self, table => $($name),*);
			};
		}

		macro_rules! add_string_list_local {
			($($name:ident),*) => {
				add_string_list!(self, table => $($name),*);
			};
		}

		add_value!(self, table => publish);
		add_value!(self, table => readme);

		add_string_list_local!(categories, exclude, include, keywords);

		add_string_local!(
			description,
			documentation,
			homepage,
			license,
			repository,
			rust_version,
			version
		);

		table.into()
	}
}
