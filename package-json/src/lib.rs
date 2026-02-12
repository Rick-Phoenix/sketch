use merge_it::*;
#[cfg(feature = "pnpm")]
use pnpm_config::PnpmWorkspace;
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Display;

mod package_json_elements;
pub use package_json_elements::*;

type JsonValueBTreeMap = BTreeMap<String, Value>;
type StringBTreeMap = BTreeMap<String, String>;

/// A struct representing the contents of a `package.json` file.
#[derive(Debug, Deserialize, Serialize, Merge, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct PackageJson {
	/// The name of the package.
	pub name: Option<String>,

	/// If set to true, then npm will refuse to publish it.
	#[merge(with = overwrite_if_true)]
	pub private: bool,

	/// Version must be parsable by node-semver, which is bundled with npm as a dependency.
	#[merge(with = overwrite_if_not_default)]
	pub version: String,

	/// When set to `module`, the type field allows a package to specify all .js files within are ES modules. If the `type` field is omitted or set to `commonjs`, all .js files are treated as CommonJS.
	#[serde(rename = "type")]
	#[merge(skip)]
	pub type_: JsPackageType,

	/// Allows packages within a directory to depend on one another using direct linking of local files. Additionally, dependencies within a workspace are hoisted to the workspace root when possible to reduce duplication. Note: It's also a good idea to set `private` to true when using this feature.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub workspaces: Option<BTreeSet<String>>,

	/// A map of shell scripts to launch from the root of the package.
	pub scripts: StringBTreeMap,

	/// The author of this package.
	#[serde(skip_serializing_if = "Option::is_none")]
	#[cfg(feature = "presets")]
	pub author: Option<Person>,
	#[cfg(not(feature = "presets"))]
	pub author: Option<PersonData>,

	/// This helps people discover your package, as it's listed in 'npm search'.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,

	/// You should specify a license for your package so that people know how they are permitted to use it, and any restrictions you're placing on it.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub license: Option<String>,

	/// Used to inform about ways to help fund development of the package.
	/// You can specify an object containing a URL that provides up-to-date information about ways to help fund development of your package, a string URL, or an array of objects and string URLs.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub funding: Option<Funding>,

	/// Specify the place where your code lives. This is helpful for people who want to contribute.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub repository: Option<Repository>,

	/// This helps people discover your package, as it's listed in 'npm search'.
	#[serde(skip_serializing_if = "BTreeSet::is_empty")]
	pub keywords: BTreeSet<String>,

	/// The url to the project homepage.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub homepage: Option<String>,

	/// The single path for this package's binary, or a map of several binaries.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub bin: Option<Bin>,

	/// The 'files' field is an array of files to include in your project. If you name a folder in the array, then it will also include the files inside that folder.
	#[serde(skip_serializing_if = "BTreeSet::is_empty")]
	pub files: BTreeSet<String>,

	/// The `exports` field is used to restrict external access to non-exported module files, also enables a module to import itself using `name`.
	#[serde(skip_serializing_if = "BTreeMap::is_empty")]
	pub exports: BTreeMap<String, Exports>,

	/// Defines which package manager is expected to be used when working on the current project. This field is currently experimental and needs to be opted-in; see https://nodejs.org/api/corepack.html
	#[serde(alias = "package_manager")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub package_manager: Option<String>,

	#[cfg(feature = "pnpm")]
	/// Configuration settings for pnpm.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub pnpm: Option<Box<PnpmWorkspace>>,

	/// Overrides is used to support selective version overrides using npm, which lets you define custom package versions or ranges inside your dependencies. For yarn, use resolutions instead. See: https://docs.npmjs.com/cli/v9/configuring-npm/package-json#overrides
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	pub overrides: JsonValueBTreeMap,

	/// Catalog of dependencies to use with `bun`
	///
	/// See more: https://bun.com/docs/install/catalogs#1-define-catalogs-in-root-package-json
	#[serde(skip_serializing_if = "BTreeMap::is_empty")]
	pub catalog: StringBTreeMap,

	/// Named catalogs to use with `bun`
	///
	/// See more: https://bun.com/docs/install/catalogs#catalog-vs-catalogs
	#[serde(skip_serializing_if = "BTreeMap::is_empty")]
	#[merge(with = merge_btree_maps)]
	pub catalogs: BTreeMap<String, StringBTreeMap>,

	/// Dependencies are specified with a simple hash of package name to version range. The version range is a string which has one or more space-separated descriptors. Dependencies can also be identified with a tarball or git URL.
	pub dependencies: StringBTreeMap,

	/// Specifies dependencies that are required for the development and testing of the project. These dependencies are not needed in the production environment.
	// Necessary to have both camelCase and snake_case
	#[serde(alias = "dev_dependencies")]
	pub dev_dependencies: StringBTreeMap,

	/// Specifies dependencies that are required by the package but are expected to be provided by the consumer of the package.
	#[serde(alias = "peer_dependencies")]
	#[serde(skip_serializing_if = "BTreeMap::is_empty")]
	pub peer_dependencies: StringBTreeMap,

	/// When a user installs your package, warnings are emitted if packages specified in "peerDependencies" are not already installed. The "peerDependenciesMeta" field serves to provide more information on how your peer dependencies are utilized. Most commonly, it allows peer dependencies to be marked as optional. Metadata for this field is specified with a simple hash of the package name to a metadata object.
	#[serde(alias = "peer_dependencies_meta")]
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	pub peer_dependencies_meta: BTreeMap<String, PeerDependencyMeta>,

	/// Specifies dependencies that are optional for your project. These dependencies are attempted to be installed during the npm install process, but if they fail to install, the installation process will not fail.
	#[serde(alias = "optional_dependencies")]
	#[serde(skip_serializing_if = "BTreeMap::is_empty")]
	pub optional_dependencies: StringBTreeMap,

	/// Array of package names that will be bundled when publishing the package.
	#[serde(alias = "bundle_dependencies")]
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	pub bundle_dependencies: BTreeSet<String>,

	/// The main field is a module ID that is the primary entry point to your program.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub main: Option<String>,

	/// Specifies the package's entrypoint for packages that work in browsers.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub browser: Option<String>,

	/// Indicates the structure of your package.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub directories: Option<Directories>,

	/// The url to your project's issue tracker and / or the email address to which issues should be reported. These are helpful for people who encounter issues with your package.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub bugs: Option<Bugs>,

	/// A list of people who contributed to this package.
	#[serde(skip_serializing_if = "BTreeSet::is_empty")]
	#[cfg(feature = "presets")]
	pub contributors: BTreeSet<Person>,
	#[cfg(not(feature = "presets"))]
	pub contributors: BTreeSet<PersonData>,

	/// A list of people who maintain this package.
	#[serde(skip_serializing_if = "BTreeSet::is_empty")]
	#[cfg(feature = "presets")]
	pub maintainers: BTreeSet<Person>,
	#[cfg(not(feature = "presets"))]
	pub maintainers: BTreeSet<PersonData>,

	/// Specify either a single file or an array of filenames to put in place for the man program to find.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub man: Option<Man>,

	/// An object that can be used to set configuration parameters used in package scripts that persist across upgrades.
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	pub config: JsonValueBTreeMap,

	/// A set of config values that will be used at publish-time. It's especially handy if you want to set the tag, registry or access, so that you can ensure that a given package is not tagged with "latest", published to the global public registry or that a scoped module is private by default.
	#[serde(alias = "publish_config")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub publish_config: Option<PublishConfig>,

	/// Defines which tools and versions are expected to be used.
	#[serde(skip_serializing_if = "BTreeMap::is_empty")]
	pub engines: StringBTreeMap,

	/// Specify which operating systems your module will run on.
	#[serde(skip_serializing_if = "BTreeSet::is_empty")]
	pub os: BTreeSet<String>,

	/// Specify that your code only runs on certain cpu architectures.
	#[serde(skip_serializing_if = "BTreeSet::is_empty")]
	pub cpu: BTreeSet<String>,

	#[serde(skip_serializing_if = "BTreeMap::is_empty")]
	#[serde(flatten)]
	pub metadata: JsonValueBTreeMap,
}

impl Default for PackageJson {
	fn default() -> Self {
		Self {
			catalog: Default::default(),
			catalogs: Default::default(),
			name: None,
			private: true,
			#[cfg(feature = "pnpm")]
			pnpm: None,
			overrides: Default::default(),
			bin: None,
			funding: None,
			type_: JsPackageType::Module,
			version: "0.1.0".to_string(),
			dependencies: Default::default(),
			peer_dependencies_meta: Default::default(),
			dev_dependencies: Default::default(),
			scripts: Default::default(),
			metadata: Default::default(),
			repository: None,
			description: None,
			package_manager: Default::default(),
			config: Default::default(),
			publish_config: Default::default(),
			man: Default::default(),
			exports: Default::default(),
			files: Default::default(),
			engines: Default::default(),
			maintainers: Default::default(),
			contributors: Default::default(),
			author: None,
			license: Default::default(),
			bugs: Default::default(),
			os: Default::default(),
			cpu: Default::default(),
			keywords: Default::default(),
			homepage: Default::default(),
			main: Default::default(),
			browser: Default::default(),
			bundle_dependencies: Default::default(),
			peer_dependencies: Default::default(),
			optional_dependencies: Default::default(),
			workspaces: Default::default(),
			directories: Default::default(),
		}
	}
}
