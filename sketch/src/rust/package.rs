use super::*;

/// The `[package]` section of the [`Manifest`]. This is where crate properties are.
///
/// Note that most of these properties can be inherited from a workspace, and therefore not available just from reading a single `Cargo.toml`. See [`Manifest::inherit_workspace`].
///
/// You can replace `Metadata` generic type with your own
/// to parse into something more useful than a generic toml `Value`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Merge, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct Package {
	/// Careful: some names are uppercase, case-sensitive. `-` changes to `_` when used as a Rust identifier.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub name: Option<String>,

	/// Must parse as semver, e.g. "1.9.0"
	///
	/// This field may have unknown value when using workspace inheritance,
	/// and when the `Manifest` has been loaded without its workspace.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub version: Option<Inheritable<String>>,

	/// Package's edition opt-in. Use [`Package::edition()`] to read it.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub edition: Option<Inheritable<Edition>>,

	/// MSRV 1.x (beware: does not require semver formatting)
	#[serde(skip_serializing_if = "Option::is_none")]
	pub rust_version: Option<Inheritable<String>>,

	/// Build script definition
	#[serde(skip_serializing_if = "Option::is_none")]
	pub build: Option<OptionalFile>,

	/// Workspace this package is a member of (`None` if it's implicit)
	#[serde(skip_serializing_if = "Option::is_none")]
	pub workspace: Option<PathBuf>,

	/// It doesn't link to anything
	#[serde(skip_serializing_if = "Option::is_none")]
	pub links: Option<String>,

	/// A short blurb about the package. This is not rendered in any format when
	/// uploaded to crates.io (aka this is not markdown).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<Inheritable<String>>,

	/// Project's homepage
	#[serde(skip_serializing_if = "Option::is_none")]
	pub homepage: Option<Inheritable<String>>,

	/// Path to your custom docs. Unnecssary if you rely on docs.rs.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub documentation: Option<Inheritable<String>>,

	/// This points to a file under the package root (relative to this `Cargo.toml`).
	/// implied if README.md, README.txt or README exists.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub readme: Option<Inheritable<OptionalFile>>,

	/// Up to 5, for search
	#[serde(skip_serializing_if = "Inheritable::is_default")]
	#[merge(with = merge_inheritable_set)]
	pub keywords: Inheritable<BTreeSet<String>>,

	/// This is a list of up to five categories where this crate would fit.
	/// e.g. `["command-line-utilities", "development-tools::cargo-plugins"]`
	#[serde(skip_serializing_if = "Inheritable::is_default")]
	#[merge(with = merge_inheritable_set)]
	pub categories: Inheritable<BTreeSet<String>>,

	/// Don't publish these files
	#[serde(skip_serializing_if = "Inheritable::is_default")]
	#[merge(with = merge_inheritable_set)]
	pub exclude: Inheritable<BTreeSet<String>>,

	/// Publish these files
	#[serde(skip_serializing_if = "Inheritable::is_default")]
	#[merge(with = merge_inheritable_set)]
	pub include: Inheritable<BTreeSet<String>>,

	/// e.g. "MIT"
	#[serde(skip_serializing_if = "Option::is_none")]
	pub license: Option<Inheritable<String>>,

	/// If `license` is not standard
	#[serde(skip_serializing_if = "Option::is_none")]
	pub license_file: Option<Inheritable<PathBuf>>,

	/// (HTTPS) URL to crate's repository
	#[serde(skip_serializing_if = "Option::is_none")]
	pub repository: Option<Inheritable<String>>,

	/// The default binary to run by cargo run.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub default_run: Option<String>,

	/// Discover binaries from the file system
	///
	/// This may be incorrectly set to `true` if the crate uses 2015 edition and has explicit `[[bin]]` sections
	#[serde(skip_serializing_if = "Option::is_none")]
	pub autobins: Option<bool>,

	/// Discover libraries from the file system
	#[serde(skip_serializing_if = "Option::is_none")]
	pub autolib: Option<bool>,

	/// Discover examples from the file system
	///
	/// This may be incorrectly set to `true` if the crate uses 2015 edition and has explicit `[[example]]` sections
	#[serde(skip_serializing_if = "Option::is_none")]
	pub autoexamples: Option<bool>,

	/// Discover tests from the file system
	///
	/// This may be incorrectly set to `true` if the crate uses 2015 edition and has explicit `[[test]]` sections
	#[serde(skip_serializing_if = "Option::is_none")]
	pub autotests: Option<bool>,

	/// Discover benchmarks from the file system
	///
	/// This may be incorrectly set to `true` if the crate uses 2015 edition and has explicit `[[bench]]` sections
	#[serde(skip_serializing_if = "Option::is_none")]
	pub autobenches: Option<bool>,

	/// Disable publishing or select custom registries.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub publish: Option<Inheritable<Publish>>,

	/// The feature resolver version.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub resolver: Option<Resolver>,

	/// Arbitrary metadata of any type, an extension point for 3rd party tools.
	#[serde(skip_serializing_if = "BTreeMap::is_empty")]
	pub metadata: BTreeMap<String, Value>,
}

impl AsTomlValue for Package {
	fn as_toml_value(&self) -> Item {
		let mut table = Table::new();

		add_string!(self, table => name, links);

		add_value!(self, table => version, edition, rust_version, build, description, homepage, documentation, readme, license, repository, default_run, publish, resolver);

		if let Some(license_file) = &self.license_file {
			table["license-file"] = match license_file {
				Inheritable::Workspace { workspace } => {
					InlineTable::from_iter([("workspace", *workspace)]).into()
				}
				Inheritable::Set(path) => path.to_string_lossy().as_ref().into(),
			};
		}

		macro_rules! add_set {
			($($names:ident),*) => {
				$(
					if !self.$names.is_default() {
						table[stringify!($names)] = match &self.$names {
							Inheritable::Workspace { workspace } => {
								InlineTable::from_iter([("workspace", *workspace)]).into()
							}
							Inheritable::Set(set) => {
								let mut array = Array::from_iter(set);
								format_array(&mut array);

								array.into()
							}
						}
					}
				)*
			};
		}

		add_set!(categories, keywords, exclude, include);

		if let Some(path) = &self.workspace {
			table["workspace"] = path.to_string_lossy().as_ref().into();
		}

		add_if_false!(self, table => autobins, autolib, autoexamples, autotests, autobenches);

		if !self.metadata.is_empty() {
			let mut metadata = Table::from_iter(self.metadata.iter().filter_map(|(k, v)| {
				json_to_standard_table(v).map(|v| (toml_edit::Key::from(k), v))
			}));

			metadata.set_implicit(true);

			table["metadata"] = metadata.into();
		}

		table.into()
	}
}
