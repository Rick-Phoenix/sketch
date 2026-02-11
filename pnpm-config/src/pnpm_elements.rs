use super::*;

/// When a user installs your package, warnings are emitted if packages specified in "peerDependencies" are not already installed. The "peerDependenciesMeta" field serves to provide more information on how your peer dependencies are utilized. Most commonly, it allows peer dependencies to be marked as optional. Metadata for this field is specified with a simple hash of the package name to a metadata object.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default)]
pub struct PeerDependencyMeta {
	/// Specifies that this peer dependency is optional and should not be installed automatically.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub optional: Option<bool>,

	#[serde(flatten, skip_serializing_if = "BTreeMap::is_empty")]
	pub extras: JsonValueBTreeMap,
}

/// Determines how pnpm resolves dependencies. See more: https://pnpm.io/settings#resolutionmode
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Copy, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum ResolutionMode {
	/// Dependencies will be resolved to their highest versions.
	#[serde(rename = "highest")]
	Highest,

	/// Direct dependencies will be resolved to their lowest versions.
	#[serde(rename = "lowest-direct")]
	LowestDirect,

	/// When resolutionMode is set to time-based, dependencies will be resolved the following way:
	/// Direct dependencies will be resolved to their lowest versions. So if there is foo@^1.1.0 in the dependencies, then 1.1.0 will be installed.
	/// Subdependencies will be resolved from versions that were published before the last direct dependency was published.
	#[serde(rename = "time-based")]
	TimeBased,
}

/// Configure how versions of packages installed to a package.json file get prefixed. See more: https://pnpm.io/settings#saveprefix
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Copy, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum SavePrefix {
	/// Do not allow upgrades.
	#[serde(rename = "")]
	Exact,
	/// Allows patch upgrades.
	#[serde(rename = "~")]
	Patch,
	/// Allows minor upgrades.
	#[serde(rename = "^")]
	Minor,
}

/// This setting controls how dependencies that are linked from the workspace are added to package.json. See more: https://pnpm.io/settings#saveworkspaceprotocol
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Copy, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum SaveWorkspaceProtocol {
	Rolling,
	#[serde(untagged)]
	Bool(bool),
}

/// If this is enabled, locally available packages are linked to node_modules instead of being downloaded from the registry. See more: https://pnpm.io/settings#linkworkspacepackages
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Copy, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum LinkWorkspacePackages {
	Deep,
	#[serde(untagged)]
	Bool(bool),
}

/// This setting allows the checking of the state of dependencies before running scripts. See more: https://pnpm.io/settings#verifydepsbeforerun
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Copy, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum VerifyDepsBeforeRun {
	/// Automatically runs install if node_modules is not up to date.
	Install,
	/// Prints a warning if node_modules is not up to date.
	Warn,
	/// Throws an error if node_modules is not up to date.
	Error,
	/// Prompts the user for permission to run install if node_modules is not up to date.
	Prompt,
	#[serde(untagged)]
	Bool(bool),
}

/// Controls colors in the output. See more: https://pnpm.io/settings#no-color
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Copy, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum Color {
	/// Ignore the difference between terminals and pipes.
	Always,
	/// Output uses colors when the standard output is a terminal or TTY
	Auto,
	/// Turns off colors. This is the setting used by --no-color.
	Never,
}

/// Any logs at or higher than the given level will be shown. See more: https://pnpm.io/settings#loglevel
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Copy, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum LogLevel {
	Debug,
	Info,
	Warn,
	Error,
}

/// Controls the way packages are imported from the store (if you want to disable symlinks inside node_modules, then you need to change the nodeLinker setting, not this one). See more: https://pnpm.io/settings#packageimportmethod
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Copy, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum PackageImportMethod {
	/// Try to clone packages from the store. If cloning is not supported then hardlink packages from the store. If neither cloning nor linking is possible, fall back to copying.
	Auto,
	/// Hard link packages from the store.
	Hardlink,
	/// Clone (AKA copy-on-write or reference link) packages from the store.
	Clone,
	/// Try to clone packages from the store. If cloning is not supported then fall back to copying.
	#[serde(rename = "clone-or-copy")]
	CloneOrCopy,
	/// Copy packages from the store.
	Copy,
}

/// Defines what linker should be used for installing Node packages. See more: https://pnpm.io/settings#nodelinker
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Copy, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum NodeLinker {
	/// Dependencies are symlinked from a virtual store at node_modules/.pnpm
	Isolated,
	/// A flat node_modules without symlinks is created.
	Hoisted,
	/// No node_modules. Plug'n'Play is an innovative strategy for Node that is used by Yarn Berry. It is recommended to also set symlink setting to false when using pnp as your linker.
	Pnp,
}

/// Instructions for the runtime, such as the node version to use. See more: https://pnpm.io/settings#executionenvnodeversion
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ExecutionEnv {
	/// Specifies which exact Node.js version should be used for the project's runtime.
	#[serde(alias = "node_version")]
	pub node_version: Option<String>,
}

/// Specifies architectures for which you'd like to install optional dependencies, even if they don't match the architecture of the system running the install. See more: https://pnpm.io/settings#supportedarchitectures
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct SupportedArchitectures {
	#[serde(alias = "cpu")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub cpu: Option<BTreeSet<String>>,

	#[serde(alias = "libc")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub libc: Option<BTreeSet<String>>,

	#[serde(alias = "os")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub os: Option<BTreeSet<String>>,
}

/// Settings for the `pnpm audit` command. See more: https://pnpm.io/settings#auditconfig
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct AuditConfig {
	/// A list of CVE IDs that will be ignored by `pnpm audit`.
	#[serde(alias = "ignore_cves")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub ignore_cves: Option<BTreeSet<String>>,

	/// A list of GHSA Codes that will be ignored by `pnpm audit`.
	#[serde(alias = "ignore_ghas")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub ignore_ghas: Option<BTreeSet<String>>,
}

/// Configuration for package updates. See more: https://pnpm.io/settings#updateconfig
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct UpdateConfig {
	/// A list of packages that should be ignored when running `pnpm outdated` or `pnpm update --latest`.
	#[serde(alias = "ignore_dependencies")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub ignore_dependencies: Option<BTreeSet<String>>,
}

/// Rules for peer dependencies. See more: https://pnpm.io/settings#peerdependencyrules
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct PeerDependencyRules {
	/// Unmet peer dependency warnings will not be printed for peer dependencies of the specified range.
	#[serde(alias = "allowed_versions")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub allowed_versions: Option<StringBTreeMap>,

	/// A list of package name patterns, any peer dependency matching the pattern will be resolved from any version, regardless of the range specified in peerDependencies.
	#[serde(alias = "allow_any")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub allow_any: Option<BTreeSet<String>>,

	/// pnpm will not print warnings about missing peer dependencies from this list.
	#[serde(alias = "ignore_missing")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub ignore_missing: Option<BTreeSet<String>>,
}

/// Package extensions offer a way to extend the existing package definitions with additional information. For example, if react-redux should have react-dom in its peerDependencies but it has not, it is possible to patch react-redux using packageExtensions. See more: https://pnpm.io/settings#packageextensions
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct PackageExtension {
	/// Dependencies are specified with a simple hash of package name to version range. The version range is a string which has one or more space-separated descriptors. Dependencies can also be identified with a tarball or git URL.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub dependencies: Option<StringBTreeMap>,

	/// Specifies dependencies that are required for the development and testing of the project. These dependencies are not needed in the production environment.
	#[serde(alias = "dev_dependencies")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub dev_dependencies: Option<StringBTreeMap>,

	/// Specifies dependencies that are optional for your project. These dependencies are attempted to be installed during the npm install process, but if they fail to install, the installation process will not fail.
	#[serde(alias = "optional_dependencies")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub optional_dependencies: Option<StringBTreeMap>,

	/// Specifies dependencies that are required by the package but are expected to be provided by the consumer of the package.
	#[serde(alias = "peer_dependencies")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub peer_dependencies: Option<StringBTreeMap>,

	/// When a user installs your package, warnings are emitted if packages specified in "peerDependencies" are not already installed. The "peerDependenciesMeta" field serves to provide more information on how your peer dependencies are utilized. Most commonly, it allows peer dependencies to be marked as optional. Metadata for this field is specified with a simple hash of the package name to a metadata object.
	#[serde(alias = "peer_dependencies_meta")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub peer_dependencies_meta: Option<BTreeMap<String, PeerDependencyMeta>>,
}

/// Controlls if and how dependencies are added to the default catalog, when running pnpm add. See more: https://pnpm.io/settings#catalogmode
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Copy, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum CatalogMode {
	/// (default) - does not automatically add dependencies to the catalog.
	Manual,
	/// Only allows dependency versions from the catalog. Adding a dependency outside the catalog's version range will cause an error.
	Strict,
	/// Prefers catalog versions, but will fall back to direct dependencies if no compatible version is found.
	Prefer,
}
