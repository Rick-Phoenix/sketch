use super::*;

/// Workspace settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, Merge)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default, rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct Workspace {
	/// Relative paths of crates in this workspace.
	pub members: BTreeSet<String>,

	/// Members to operate on when in the workspace root.
	///
	/// When specified, `default-members` must expand to a subset of `members`.
	#[serde(skip_serializing_if = "BTreeSet::is_empty")]
	pub default_members: BTreeSet<String>,

	/// Settings that can be inherited by packages in this workspace.
	#[serde(skip_serializing_if = "Option::is_none")]
	#[merge(with = merge_options)]
	pub package: Option<PackageTemplate>,

	/// Ignore these dirs
	#[serde(skip_serializing_if = "BTreeSet::is_empty")]
	pub exclude: BTreeSet<String>,

	/// Custom settings for the workspace.
	#[serde(skip_serializing_if = "BTreeMap::is_empty")]
	pub metadata: BTreeMap<String, Value>,

	/// The resolver to use for the workspace.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub resolver: Option<Resolver>,

	/// Dependencies that can be inherited by packages in the workspace.
	#[serde(skip_serializing_if = "BTreeMap::is_empty")]
	#[merge(with = merge_btree_maps)]
	pub dependencies: BTreeMap<String, Dependency>,

	/// Workspace-level lint groups, which can be inherited by packages.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub lints: Option<Lints>,
}

impl AsTomlValue for Workspace {
	fn as_toml_value(&self) -> Item {
		let mut table = Table::new();

		add_value!(self, table => resolver);

		add_string_list!(self, table => members, default_members, exclude);

		add_value!(self, table => package, lints);

		if !self.metadata.is_empty() {
			let mut metadata = Table::from_iter(self.metadata.iter().filter_map(|(k, v)| {
				json_to_standard_table(v).map(|v| (toml_edit::Key::from(k), v))
			}));

			metadata.set_implicit(true);

			table["metadata"] = metadata.into();
		}

		if !self.dependencies.is_empty() {
			table["dependencies"] = Table::from_iter(
				self.dependencies
					.iter()
					.map(|(name, dep)| (toml_edit::Key::from(name), dep.as_toml_value())),
			)
			.into();
		}

		table.into()
	}
}

/// Settings for lint groups in a workspace/package.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Merge)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct Lints {
	/// rustc lints
	#[serde(skip_serializing_if = "BTreeMap::is_empty")]
	pub rust: BTreeMap<String, LintKind>,

	/// Clippy lints
	#[serde(skip_serializing_if = "BTreeMap::is_empty")]
	pub clippy: BTreeMap<String, LintKind>,
}

impl AsTomlValue for Lints {
	fn as_toml_value(&self) -> Item {
		let mut table = Table::new();

		table.set_implicit(true);

		add_table!(self, table => rust, clippy);

		table.into()
	}
}

/// Lint group entry
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
pub enum LintKind {
	/// Simple definition, with lint level (i.e. "allow")
	Simple(LintLevel),
	/// Detailed definintion
	Detailed(Lint),
}

impl AsTomlValue for LintKind {
	fn as_toml_value(&self) -> Item {
		match self {
			Self::Simple(lev) => lev.as_toml_value(),
			Self::Detailed(det) => det.as_toml_value(),
		}
	}
}

/// Properties that can be inherited via `{ workspace = true }` by member packages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, Merge)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default, rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct PackageTemplate {
	/// See <https://crates.io/category_slugs>
	#[serde(skip_serializing_if = "BTreeSet::is_empty")]
	pub categories: BTreeSet<String>,

	/// Description for a package.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,

	/// Link to the documentation
	#[serde(skip_serializing_if = "Option::is_none")]
	pub documentation: Option<String>,

	/// Opt-in to new Rust behaviors
	#[serde(skip_serializing_if = "Option::is_none")]
	pub edition: Option<Edition>,

	/// Don't publish these files, relative to workspace
	#[serde(skip_serializing_if = "BTreeSet::is_empty")]
	pub exclude: BTreeSet<String>,

	/// Homepage URL
	#[serde(skip_serializing_if = "Option::is_none")]
	pub homepage: Option<String>,

	/// Publish these files, relative to workspace
	#[serde(skip_serializing_if = "BTreeSet::is_empty")]
	pub include: BTreeSet<String>,

	/// Keywords to use for a package
	#[serde(skip_serializing_if = "BTreeSet::is_empty")]
	pub keywords: BTreeSet<String>,

	/// License
	#[serde(skip_serializing_if = "Option::is_none")]
	pub license: Option<String>,

	/// Block publishing or choose custom registries
	#[serde(skip_serializing_if = "Option::is_none")]
	pub publish: Option<Publish>,

	/// Opt-out or custom path, relative to workspace
	#[serde(skip_serializing_if = "Option::is_none")]
	pub readme: Option<OptionalFile>,

	/// (HTTPS) repository URL
	#[serde(skip_serializing_if = "Option::is_none")]
	pub repository: Option<String>,

	/// Minimum required rustc version in format `1.99`
	#[serde(skip_serializing_if = "Option::is_none")]
	pub rust_version: Option<String>,

	/// Package version semver
	#[serde(skip_serializing_if = "Option::is_none")]
	pub version: Option<String>,
}

impl AsTomlValue for PackageTemplate {
	fn as_toml_value(&self) -> Item {
		let mut table = Table::new();

		add_value!(self, table => publish, edition, readme);

		add_string_list!(self, table => categories, exclude, include, keywords);

		add_string!(self, table =>
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
