use futures::StreamExt;
mod pnpm_elements;

use futures::stream;
pub use pnpm_elements::*;

use crate::{
	ts::{package_json::*, *},
	versions::*,
	*,
};

/// A preset for a `pnpm-workspace.yaml` file configuration.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default, Eq, Merge)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default)]
pub struct PnpmPreset {
	/// The list of extended presets.
	pub extends_presets: IndexSet<String>,

	#[serde(flatten)]
	pub config: PnpmWorkspace,
}

impl Extensible for PnpmPreset {
	fn get_extended(&self) -> &IndexSet<String> {
		&self.extends_presets
	}
}

impl PnpmPreset {
	pub fn process_data(
		self,
		id: &str,
		store: &IndexMap<String, Self>,
	) -> Result<PnpmWorkspace, GenError> {
		if self.extends_presets.is_empty() {
			return Ok(self.config);
		}

		let mut processed_ids: IndexSet<String> = IndexSet::new();

		let merged_preset =
			merge_presets(Preset::PnpmWorkspace, id, self, store, &mut processed_ids)?;

		Ok(merged_preset.config)
	}
}

/// A struct representing a pnpm-workspace.yaml config.
///
/// See more: https://pnpm.io/settings
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default, Eq, Merge)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct PnpmWorkspace {
	/// Glob patterns for the directories containing the packages for this workspace.
	#[serde(skip_serializing_if = "BTreeSet::is_empty")]
	pub packages: BTreeSet<String>,

	/// When set to true, pnpm will remove unused catalog entries during installation.
	///
	/// See more: https://pnpm.io/settings#cleanupunusedcatalogs
	#[serde(alias = "cleanup_unused_catalogs")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub cleanup_unused_catalogs: Option<bool>,

	/// The dependencies to store in the unnamed (default) catalog.
	#[serde(skip_serializing_if = "BTreeMap::is_empty")]
	pub catalog: StringBTreeMap,

	/// A map of named catalogs and the dependencies listed in them.
	#[serde(skip_serializing_if = "BTreeMap::is_empty")]
	#[merge(with = merge_nested_maps)]
	pub catalogs: BTreeMap<String, StringBTreeMap>,

	/// A list of package names that are allowed to be executed during installation. Only packages listed in this array will be able to run install scripts. If onlyBuiltDependenciesFile and neverBuiltDependencies are not set, this configuration option will default to blocking all install scripts.
	///
	/// See more: https://pnpm.io/settings#onlybuiltdependencies
	#[serde(alias = "only_built_dependencies")]
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	pub only_built_dependencies: BTreeSet<String>,

	/// Specifies a JSON file that lists the only packages permitted to run installation scripts during the pnpm install process.
	///
	/// See more: https://pnpm.io/settings#onlybuiltdependenciesfile
	#[serde(alias = "only_built_dependencies_file")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub only_built_dependencies_file: Option<PathBuf>,

	/// A list of dependencies to run builds for. See more: https://pnpm.io/settings#neverbuiltdependencies
	#[serde(alias = "never_built_dependencies")]
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	pub never_built_dependencies: BTreeSet<String>,

	/// A list of package names that should not be built during installation.
	///
	/// See more: https://pnpm.io/settings#ignoredbuiltdependencies
	#[serde(alias = "ignored_built_dependencies")]
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	pub ignored_built_dependencies: BTreeSet<String>,

	/// If set to true, all build scripts (e.g. preinstall, install, postinstall) from dependencies will run automatically, without requiring approval.
	///
	/// See more: https://pnpm.io/settings#dangerouslyallowallbuilds
	#[serde(alias = "dangerously_allow_all_builds")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub dangerously_allow_all_builds: Option<bool>,

	/// This field allows you to instruct pnpm to override any dependency in the dependency graph. This is useful for enforcing all your packages to use a single version of a dependency, backporting a fix, replacing a dependency with a fork, or removing an unused dependency.
	///
	/// See more: https://pnpm.io/settings#overrides
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	pub overrides: StringBTreeMap,

	/// Configuration for package updates.
	///
	/// See more: https://pnpm.io/settings#updateconfig
	#[serde(alias = "update_config")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub update_config: Option<UpdateConfig>,

	/// It specifies the number of minutes that must pass after a version is published before pnpm will install it. For example, setting `minimumReleaseAge: 1440` ensures that only packages released at least one day ago can be installed.
	///
	/// See more: https://pnpm.io/settings#minimumreleaseage
	#[serde(alias = "minimum_release_age")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub minimum_release_age: Option<usize>,

	/// If you set `minimumReleaseAge` but need to disable this restriction for certain dependencies, you can list them under the `minimumReleaseAgeExclude` setting.
	///
	/// See more: https://pnpm.io/settings#minimumreleaseageexclude
	#[serde(alias = "minimum_release_age_exclude")]
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	pub minimum_release_age_exclude: BTreeSet<String>,

	/// The packageExtensions fields offer a way to extend the existing package definitions with additional information. For example, if react-redux should have react-dom in its peerDependencies but it has not, it is possible to patch react-redux using packageExtensions.
	///
	/// See more: https://pnpm.io/settings#packageextensions
	#[serde(alias = "package_extensions")]
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	pub package_extensions: BTreeMap<String, PackageExtension>,

	/// Rules for peer dependencies.
	///
	/// See more: https://pnpm.io/settings#peerdependencyrules
	#[serde(alias = "peer_dependency_rules")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub peer_dependency_rules: Option<PeerDependencyRules>,

	/// A list of deprecated versions that the warnings are suppressed.
	///
	/// See more: https://pnpm.io/settings#alloweddeprecatedversions
	#[serde(alias = "allowed_deprecated_versions")]
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	pub allowed_deprecated_versions: StringBTreeMap,

	/// A list of dependencies that are patched.
	///
	/// See more: https://pnpm.io/settings#patcheddependencies
	#[serde(alias = "patched_dependencies")]
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	pub patched_dependencies: StringBTreeMap,

	/// When true, installation won't fail if some of the patches from the `patchedDependencies` field were not applied. Previously named `allowNonAppliedPatches`.
	///
	/// See more: https://pnpm.io/settings#allowunusedpatches
	#[serde(alias = "allow_unused_patches")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub allow_unused_patches: Option<bool>,

	/// Default is undefined. Errors out when a patch with an exact version or version range fails. Ignores failures from name-only patches. When true, prints a warning instead of failing when any patch cannot be applied. When false, errors out for any patch failure.
	///
	/// See more: https://pnpm.io/settings#ignorepatchfailures
	#[serde(alias = "ignore_patch_failures")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub ignore_patch_failures: Option<bool>,

	/// Config dependencies allow you to share and centralize configuration files, settings, and hooks across multiple projects. They are installed before all regular dependencies ('dependencies', 'devDependencies', 'optionalDependencies'), making them ideal for setting up custom hooks, patches, and catalog entries.
	///
	/// See more: https://pnpm.io/config-dependencies
	#[serde(alias = "config_dependencies")]
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	pub config_dependencies: StringBTreeMap,

	/// Settings for the `pnpm audit` command.
	///
	/// See more: https://pnpm.io/settings#auditconfig
	#[serde(alias = "audit_config")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub audit_config: Option<AuditConfig>,

	/// Scripts listed in this array will be required in each project of the workspace. Otherwise, pnpm -r run <script name> will fail.
	///
	/// See more: https://pnpm.io/settings#requiredscripts
	#[serde(alias = "required_scripts")]
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	pub required_scripts: BTreeSet<String>,

	/// Specifies architectures for which you'd like to install optional dependencies, even if they don't match the architecture of the system running the install.
	///
	/// See more: https://pnpm.io/settings#supportedarchitectures
	#[serde(alias = "supported_architectures")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub supported_architectures: Option<SupportedArchitectures>,

	/// A list of optional dependencies that the install should be skipped.
	///
	/// See more: https://pnpm.io/settings#ignoredoptionaldependencies
	#[serde(alias = "ignored_optional_dependencies")]
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	pub ignored_optional_dependencies: BTreeSet<String>,

	/// Instructions for the runtime, such as the node version to use.
	///
	/// See more: https://pnpm.io/settings#executionenvnodeversion
	#[serde(alias = "execution_env")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub execution_env: Option<ExecutionEnv>,

	/// When true, all dependencies are hoisted to node_modules/.pnpm/node_modules.
	///
	/// See more: https://pnpm.io/settings#hoist
	#[serde(alias = "hoist")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub hoist: Option<bool>,

	/// When true, packages from the workspaces are symlinked to either <workspace_root>/node_modules/.pnpm/node_modules or to <workspace_root>/node_modules depending on other hoisting settings (hoistPattern and publicHoistPattern).
	///
	/// See more: https://pnpm.io/settings#hoistworkspacepackages
	#[serde(alias = "hoist_workspace_packages")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub hoist_workspace_packages: Option<bool>,

	/// Tells pnpm which packages should be hoisted to node_modules/.pnpm/node_modules.
	///
	/// See more: https://pnpm.io/settings#hoistpattern
	#[serde(alias = "hoist_pattern")]
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	pub hoist_pattern: BTreeSet<String>,

	/// Unlike hoistPattern, which hoists dependencies to a hidden modules directory inside the virtual store, publicHoistPattern hoists dependencies matching the pattern to the root modules directory.
	///
	/// See more: https://pnpm.io/settings#publichoistpattern
	#[serde(alias = "public_hoist_pattern")]
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	pub public_hoist_pattern: BTreeSet<String>,

	/// By default, pnpm creates a semistrict node_modules, meaning dependencies have access to undeclared dependencies but modules outside of node_modules do not.
	///
	/// See more: https://pnpm.io/settings#shamefullyhoist
	#[serde(alias = "shamefully_hoist")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub shamefully_hoist: Option<bool>,

	/// The directory in which dependencies will be installed (instead of node_modules).
	///
	/// See more: https://pnpm.io/settings#modulesdir
	#[serde(alias = "modules_dir")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub modules_dir: Option<PathBuf>,

	/// Defines what linker should be used for installing Node packages.
	///
	/// See more: https://pnpm.io/settings#nodelinker
	#[serde(alias = "node_linker")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub node_linker: Option<NodeLinker>,

	/// When symlink is set to false, pnpm creates a virtual store directory without any symlinks. It is a useful setting together with nodeLinker=pnp.
	///
	/// See more: https://pnpm.io/settings#symlink
	#[serde(skip_serializing_if = "Option::is_none")]
	pub symlink: Option<bool>,

	/// When false, pnpm will not write any files to the modules directory (node_modules).
	///
	/// See more: https://pnpm.io/settings#enablemodulesdir
	#[serde(alias = "enable_modules_dir")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub enable_modules_dir: Option<bool>,

	/// The directory with links to the store.
	///
	/// See more: https://pnpm.io/settings#virtualstoredir
	#[serde(alias = "virtual_store_dir")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub virtual_store_dir: Option<PathBuf>,

	/// Sets the maximum allowed length of directory names inside the virtual store directory (node_modules/.pnpm).
	///
	/// See more: https://pnpm.io/settings#virtualstoredirmaxlength
	#[serde(alias = "virtual_store_dir_max_length")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub virtual_store_dir_max_length: Option<usize>,

	/// Controls the way packages are imported from the store (if you want to disable symlinks inside node_modules, then you need to change the nodeLinker setting, not this one).
	///
	/// See more: https://pnpm.io/settings#packageimportmethod
	#[serde(alias = "package_import_method")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub package_import_method: Option<PackageImportMethod>,

	/// The time in minutes after which orphan packages from the modules directory should be removed.
	///
	/// See more: https://pnpm.io/settings#modulescachemaxage
	#[serde(alias = "modules_cache_max_age")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub modules_cache_max_age: Option<usize>,

	/// The time in minutes after which dlx cache expires.
	///
	/// See more: https://pnpm.io/settings#dlxcachemaxage
	#[serde(alias = "dlx_cache_max_age")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub dlx_cache_max_age: Option<usize>,

	/// The location where all the packages are saved on the disk.
	///
	/// See more: https://pnpm.io/settings#storedir
	#[serde(alias = "store_dir")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub store_dir: Option<PathBuf>,

	/// By default, if a file in the store has been modified, the content of this file is checked before linking it to a project's node_modules.
	///
	/// See more: https://pnpm.io/settings#verifystoreintegrity
	#[serde(alias = "verify_store_integrity")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub verify_store_integrity: Option<bool>,

	/// Some registries allow the exact same content to be published under different package names and/or versions.
	///
	/// See more: https://pnpm.io/settings#strictstorepkgcontentcheck
	#[serde(alias = "strict_store_pkg_content_check")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub strict_store_pkg_content_check: Option<bool>,

	/// When enabled, node_modules contains only symlinks to a central virtual store, rather than to node_modules/.pnpm.
	///
	/// See more: https://pnpm.io/settings#enableglobalvirtualstore
	#[serde(alias = "enable_global_virtual_store")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub enable_global_virtual_store: Option<bool>,

	/// When set to false, pnpm won't read or generate a pnpm-lock.yaml file.
	///
	/// See more: https://pnpm.io/settings#lockfile
	#[serde(alias = "lockfile")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub lockfile: Option<bool>,

	/// When set to true and the available pnpm-lock.yaml satisfies the package.json dependencies directive, a headless installation is performed.
	///
	/// See more: https://pnpm.io/settings#preferfrozenlockfile
	#[serde(alias = "prefer_frozen_lockfile")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub prefer_frozen_lockfile: Option<bool>,

	/// Add the full URL to the package's tarball to every entry in pnpm-lock.yaml.
	///
	/// See more: https://pnpm.io/settings#lockfileincludetarballurl
	#[serde(alias = "lockfile_include_tarball_url")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub lockfile_include_tarball_url: Option<bool>,

	/// When set to true, the generated lockfile name after installation will be named based on the current branch name to completely avoid merge conflicts.
	///
	/// See more: https://pnpm.io/settings#gitbranchlockfile
	#[serde(alias = "git_branch_lockfile")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub git_branch_lockfile: Option<bool>,

	/// This configuration matches the current branch name to determine whether to merge all git branch lockfile files.
	///
	/// See more: https://pnpm.io/settings#mergegitbranchlockfilesbranchpattern
	#[serde(alias = "merge_git_branch_lockfiles_branch_pattern")]
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	pub merge_git_branch_lockfiles_branch_pattern: BTreeSet<String>,

	/// Max length of the peer IDs suffix added to dependency keys in the lockfile. If the suffix is longer, it is replaced with a hash.
	///
	/// See more: https://pnpm.io/settings#peerssuffixmaxlength
	#[serde(alias = "peers_suffix_max_length")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub peers_suffix_max_length: Option<usize>,

	/// The base URL of the npm package registry (trailing slash included).
	///
	/// See more: https://pnpm.io/settings#registry
	#[serde(skip_serializing_if = "Option::is_none")]
	pub registry: Option<String>,

	/// The Certificate Authority signing certificate that is trusted for SSL connections to the registry.
	///
	/// See more: https://pnpm.io/settings#ca
	#[serde(skip_serializing_if = "Option::is_none")]
	pub ca: Option<String>,

	/// A path to a file containing one or multiple Certificate Authority signing certificates.
	///
	/// See more: https://pnpm.io/settings#cafile
	#[serde(rename = "cafile", alias = "ca_file")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub ca_file: Option<PathBuf>,

	/// A client certificate to pass when accessing the registry.
	///
	/// See more: https://pnpm.io/settings#cert
	#[serde(skip_serializing_if = "Option::is_none")]
	pub cert: Option<String>,

	/// A client key to pass when accessing the registry.
	///
	/// See more: https://pnpm.io/settings#key
	#[serde(skip_serializing_if = "Option::is_none")]
	pub key: Option<String>,

	/// When fetching dependencies that are Git repositories, if the host is listed in this setting, pnpm will use shallow cloning to fetch only the needed commit, not all the history.
	///
	/// See more: https://pnpm.io/settings#gitshallowhosts
	#[serde(alias = "git_shallow_hosts")]
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	pub git_shallow_hosts: BTreeSet<String>,

	/// A proxy to use for outgoing HTTPS requests. If the HTTPS_PROXY, https_proxy, HTTP_PROXY or http_proxy environment variables are set, their values will be used instead.
	///
	/// See more: https://pnpm.io/settings#https-proxy
	#[serde(alias = "https_proxy")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub https_proxy: Option<String>,

	/// A proxy to use for outgoing http requests. If the HTTP_PROXY or http_proxy environment variables are set, proxy settings will be honored by the underlying request library.
	///
	/// See more: https://pnpm.io/settings#proxy
	#[serde(skip_serializing_if = "Option::is_none")]
	pub proxy: Option<String>,

	/// The IP address of the local interface to use when making connections to the npm registry.
	///
	/// See more: https://pnpm.io/settings#local-address
	#[serde(alias = "local_address")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub local_address: Option<String>,

	/// The maximum number of connections to use per origin (protocol/host/port combination).
	///
	/// See more: https://pnpm.io/settings#maxsockets
	#[serde(rename = "maxsockets", alias = "max_sockets")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub max_sockets: Option<usize>,

	/// A comma-separated string of domain extensions that a proxy should not be used for.
	///
	/// See more: https://pnpm.io/settings#noproxy
	#[serde(rename = "noproxy", alias = "no_proxy")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub no_proxy: Option<String>,

	/// Whether or not to do SSL key validation when making requests to the registry via HTTPS.
	///
	/// See more: https://pnpm.io/settings#strict-ssl
	#[serde(alias = "strict_ssl")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub strict_ssl: Option<bool>,

	/// Controls the maximum number of HTTP(S) requests to process simultaneously.
	///
	/// See more: https://pnpm.io/settings#networkconcurrency
	#[serde(alias = "network_concurrency")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub network_concurrency: Option<usize>,

	/// How many times to retry if pnpm fails to fetch from the registry.
	///
	/// See more: https://pnpm.io/settings#fetchretries
	#[serde(alias = "fetch_retries")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub fetch_retries: Option<usize>,

	/// The exponential factor for retry backoff.
	///
	/// See more: https://pnpm.io/settings#fetchretryfactor
	#[serde(alias = "fetch_retry_factor")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub fetch_retry_factor: Option<usize>,

	/// The minimum (base) timeout for retrying requests.
	///
	/// See more: https://pnpm.io/settings#fetchretrymintimeout
	#[serde(alias = "fetch_retry_min_timeout")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub fetch_retry_min_timeout: Option<usize>,

	/// The maximum fallback timeout to ensure the retry factor does not make requests too long.
	///
	/// See more: https://pnpm.io/settings#fetchretrymaxtimeout
	#[serde(alias = "fetch_retry_max_timeout")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub fetch_retry_max_timeout: Option<usize>,

	/// The maximum amount of time to wait for HTTP requests to complete.
	///
	/// See more: https://pnpm.io/settings#fetchtimeout
	#[serde(alias = "fetch_timeout")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub fetch_timeout: Option<usize>,

	/// When true, any missing non-optional peer dependencies are automatically installed.
	///
	/// See more: https://pnpm.io/settings#autoinstallpeers
	#[serde(alias = "auto_install_peers")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub auto_install_peers: Option<bool>,

	/// When this setting is set to true, packages with peer dependencies will be deduplicated after peers resolution.
	///
	/// See more: https://pnpm.io/settings#dedupepeerdependents
	#[serde(alias = "dedupe_peer_dependents")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub dedupe_peer_dependents: Option<bool>,

	/// If this is enabled, commands will fail if there is a missing or invalid peer dependency in the tree.
	///
	/// See more: https://pnpm.io/settings#strictpeerdependencies
	#[serde(alias = "strict_peer_dependencies")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub strict_peer_dependencies: Option<bool>,

	/// When enabled, dependencies of the root workspace project are used to resolve peer dependencies of any projects in the workspace.
	///
	/// See more: https://pnpm.io/settings#resolvepeersfromworkspaceroot
	#[serde(alias = "resolve_peers_from_workspace_root")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub resolve_peers_from_workspace_root: Option<bool>,

	/// Controls colors in the output.
	///
	/// See more: https://pnpm.io/settings#no-color
	#[serde(skip_serializing_if = "Option::is_none")]
	pub color: Option<Color>,

	/// Any logs at or higher than the given level will be shown.
	///
	/// See more: https://pnpm.io/settings#loglevel
	#[serde(rename = "loglevel", alias = "log_level")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub log_level: Option<LogLevel>,

	/// Experimental option that enables beta features of the CLI.
	///
	/// See more: https://pnpm.io/settings#usebetacli
	#[serde(alias = "use_beta_cli")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub use_beta_cli: Option<bool>,

	/// If this is enabled, the primary behaviour of pnpm install becomes that of pnpm install -r, meaning the install is performed on all workspace or subdirectory packages.
	///
	/// See more: https://pnpm.io/settings#recursiveinstall
	#[serde(alias = "recursive_install")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub recursive_install: Option<bool>,

	/// If this is enabled, pnpm will not install any package that claims to not be compatible with the current Node version.
	///
	/// See more: https://pnpm.io/settings#enginestrict
	#[serde(alias = "engine_strict")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub engine_strict: Option<bool>,

	/// The location of the npm binary that pnpm uses for some actions, like publishing.
	///
	/// See more: https://pnpm.io/settings#npmpath
	#[serde(alias = "npm_path")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub npm_path: Option<PathBuf>,

	/// If this setting is disabled, pnpm will not fail if a different package manager is specified in the packageManager field of package.json. When enabled, only the package name is checked (since pnpm v9.2.0), so you can still run any version of pnpm regardless of the version specified in the packageManager field.
	///
	/// See more: https://pnpm.io/settings#packagemanagerstrict
	#[serde(alias = "package_manager_strict")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub package_manager_strict: Option<bool>,

	/// When enabled, pnpm will fail if its version doesn't exactly match the version specified in the packageManager field of package.json.
	///
	/// See more: https://pnpm.io/settings#packagemanagerstrictversion
	#[serde(alias = "package_manager_strict_version")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub package_manager_strict_version: Option<bool>,

	/// When enabled, pnpm will automatically download and run the version of pnpm specified in the packageManager field of package.json.
	///
	/// See more: https://pnpm.io/settings#managepackagemanagerversions
	#[serde(alias = "manage_package_manager_versions")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub manage_package_manager_versions: Option<bool>,

	/// Do not execute any scripts defined in the project package.json and its dependencies.
	///
	/// See more: https://pnpm.io/settings#ignorescripts
	#[serde(alias = "ignore_scripts")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub ignore_scripts: Option<bool>,

	/// Do not execute any scripts of the installed packages. Scripts of the projects are executed.
	///
	/// See more: https://pnpm.io/settings#ignoredepscripts
	#[serde(alias = "ignore_dep_scripts")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub ignore_dep_scripts: Option<bool>,

	/// The maximum number of child processes to allocate simultaneously to build node_modules.
	///
	/// See more: https://pnpm.io/settings#childconcurrency
	#[serde(alias = "child_concurrency")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub child_concurrency: Option<usize>,

	/// Use and cache the results of (pre/post)install hooks.
	///
	/// See more: https://pnpm.io/settings#sideeffectscache
	#[serde(alias = "size_effects_cache")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub size_effects_cache: Option<bool>,

	/// Only use the side effects cache if present, do not create it for new packages.
	///
	/// See more: https://pnpm.io/settings#sideeffectscachereadonly
	#[serde(alias = "size_effects_cache_read_only")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub size_effects_cache_read_only: Option<bool>,

	/// Set to true to enable UID/GID switching when running package scripts. If set explicitly to false, then installing as a non-root user will fail.
	///
	/// See more: https://pnpm.io/settings#unsafeperm
	#[serde(alias = "unsafe_perm")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub unsafe_perm: Option<bool>,

	/// Options to pass through to Node.js via the NODE_OPTIONS environment variable.
	///
	/// See more: https://pnpm.io/settings#nodeoptions
	#[serde(alias = "node_options")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub node_options: Option<String>,

	/// This setting allows the checking of the state of dependencies before running scripts.
	///
	/// See more: https://pnpm.io/settings#verifydepsbeforerun
	#[serde(alias = "verify_deps_before_run")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub verify_deps_before_run: Option<VerifyDepsBeforeRun>,

	/// When strictDepBuilds is enabled, the installation will exit with a non-zero exit code if any dependencies have unreviewed build scripts (aka postinstall scripts).
	///
	/// See more: https://pnpm.io/settings#strictdepbuilds
	#[serde(alias = "strict_dep_builds")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub strict_dep_builds: Option<bool>,

	/// Specifies which exact Node.js version should be used for the project's runtime.
	///
	/// See more: https://pnpm.io/settings#usenodeversion
	#[serde(alias = "use_node_version")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub use_node_version: Option<String>,

	/// The Node.js version to use when checking a package's engines setting.
	///
	/// See more: https://pnpm.io/settings#nodeversion
	#[serde(alias = "node_version")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub node_version: Option<String>,

	/// If this is enabled, locally available packages are linked to node_modules instead of being downloaded from the registry.
	///
	/// See more: https://pnpm.io/settings#linkworkspacepackages
	#[serde(alias = "link_workspace_packages")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub link_workspace_packages: Option<LinkWorkspacePackages>,

	/// Enables hard-linking of all local workspace dependencies instead of symlinking them.
	///
	/// See more: https://pnpm.io/settings#injectworkspacepackages
	#[serde(alias = "inject_workspace_packages")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub inject_workspace_packages: Option<bool>,

	/// Injected workspace dependencies are collections of hardlinks, which don't add or remove the files when their sources change.
	///
	/// See more: https://pnpm.io/settings#syncinjecteddepsafterscripts
	#[serde(alias = "sync_injected_deps_after_scripts")]
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	pub sync_injected_deps_after_scripts: BTreeSet<String>,

	/// If this is enabled, local packages from the workspace are preferred over packages from the registry, even if there is a newer version of the package in the registry.
	///
	/// See more: https://pnpm.io/settings#preferworkspacepackages
	#[serde(alias = "prefer_workspace_packages")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub prefer_workspace_packages: Option<bool>,

	/// If this is enabled, pnpm creates a single pnpm-lock.yaml file in the root of the workspace.
	///
	/// See more: https://pnpm.io/settings#sharedworkspacelockfile
	#[serde(alias = "shared_workspace_lockfile")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub shared_workspace_lockfile: Option<bool>,

	/// This setting controls how dependencies that are linked from the workspace are added to package.json.
	///
	/// See more: https://pnpm.io/settings#saveworkspaceprotocol
	#[serde(alias = "save_workspace_protocol")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub save_workspace_protocol: Option<SaveWorkspaceProtocol>,

	/// When executing commands recursively in a workspace, execute them on the root workspace project as well.
	///
	/// See more: https://pnpm.io/settings#includeworkspaceroot
	#[serde(alias = "include_workspace_root")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub include_workspace_root: Option<bool>,

	/// When set to true, no workspace cycle warnings will be printed.
	///
	/// See more: https://pnpm.io/settings#ignoreworkspacecycles
	#[serde(alias = "ignore_workspace_cycles")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub ignore_workspace_cycles: Option<bool>,

	/// Adding a new dependency to the root workspace package fails, unless the --ignore-workspace-root-check or -w flag is used.
	#[serde(alias = "ignore_workspace_root_check")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub ignore_workspace_root_check: Option<bool>,

	/// When set to true, installation will fail if the workspace has cycles.
	///
	/// See more: https://pnpm.io/settings#disallowworkspacecycles
	#[serde(alias = "disallow_workspace_cycles")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub disallow_workspace_cycles: Option<bool>,

	/// By default, pnpm deploy will try creating a dedicated lockfile from a shared lockfile for deployment. If this setting is set to true, the legacy deploy behavior will be used.
	///
	/// See more: https://pnpm.io/settings#forcelegacydeploy
	#[serde(alias = "force_legacy_deploy")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub force_legacy_deploy: Option<bool>,

	/// Configure how versions of packages installed to a package.json file get prefixed.
	///
	/// See more: https://pnpm.io/settings#saveprefix
	#[serde(alias = "save_prefix")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub save_prefix: Option<SavePrefix>,

	/// If you pnpm add a package and you don't provide a specific version, then it will install the package at the version registered under the tag from this setting.
	///
	/// See more: https://pnpm.io/settings#tag
	#[serde(skip_serializing_if = "Option::is_none")]
	pub tag: Option<String>,

	/// Specify a custom directory to store global packages.
	///
	/// See more: https://pnpm.io/settings#globaldir
	#[serde(alias = "global_dir")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub global_dir: Option<PathBuf>,

	/// Allows to set the target directory for the bin files of globally installed packages.
	///
	/// See more: https://pnpm.io/settings#globalbindir
	#[serde(alias = "global_bin_dir")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub global_bin_dir: Option<PathBuf>,

	/// The location where all the packages are saved on the disk.
	///
	/// See more: https://pnpm.io/settings#statedir
	#[serde(alias = "state_dir")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub state_dir: Option<PathBuf>,

	/// The location of the cache (package metadata and dlx).
	///
	/// See more: https://pnpm.io/settings#cachedir
	#[serde(alias = "cache_dir")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub cache_dir: Option<PathBuf>,

	/// When true, all the output is written to stderr.
	///
	/// See more: https://pnpm.io/settings#usestderr
	#[serde(alias = "use_stderr")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub use_stderr: Option<bool>,

	/// When true, pnpm will check for updates to the installed packages and notify the user.
	///
	/// See more: https://pnpm.io/settings#updatenotifier
	#[serde(alias = "update_notifier")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub update_notifier: Option<bool>,

	/// Create symlinks to executables in node_modules/.bin instead of command shims. This setting is ignored on Windows, where only command shims work.
	///
	/// See more: https://pnpm.io/settings#prefersymlinkedexecutables
	#[serde(alias = "prefer_symlinked_executabled")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub prefer_symlinked_executabled: Option<bool>,

	/// During installation the dependencies of some packages are automatically patched. If you want to disable this, set this config to false.
	///
	/// See more: https://pnpm.io/settings#ignorecompatibilitydb
	#[serde(alias = "ignore_compatibility_db")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub ignore_compatibility_db: Option<bool>,

	/// Determines how pnpm resolves dependencies.
	///
	/// See more: https://pnpm.io/settings#resolutionmode
	#[serde(alias = "resolution_mode")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub resolution_mode: Option<ResolutionMode>,

	/// Set this to true if the registry that you are using returns the `time` field in the abbreviated metadata.
	///
	/// See more: https://pnpm.io/settings#registrysupportstimefield
	#[serde(alias = "registry_supports_time_field")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub registry_supports_time_field: Option<bool>,

	/// When false, the NODE_PATH environment variable is not set in the command shims.
	///
	/// See more: https://pnpm.io/settings#extendnodepath
	#[serde(alias = "extend_node_path")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub extend_node_path: Option<bool>,

	/// When deploying a package or installing a local package, all files of the package are copied.
	///
	/// See more: https://pnpm.io/settings#deployallfiles
	#[serde(alias = "deploy_all_files")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub deploy_all_files: Option<bool>,

	/// When set to true, dependencies that are already symlinked to the root node_modules directory of the workspace will not be symlinked to subproject node_modules directories.
	///
	/// See more: https://pnpm.io/settings#dedupedirectdeps
	#[serde(alias = "dedupe_direct_deps")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub dedupe_direct_deps: Option<bool>,

	/// When this setting is enabled, dependencies that are injected will be symlinked from the workspace whenever possible.
	///
	/// See more: https://pnpm.io/settings#dedupeinjecteddeps
	#[serde(alias = "dedupe_injected_deps")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub dedupe_injected_deps: Option<bool>,

	/// When enabled, a fast check will be performed before proceeding to installation. This way a repeat install or an install on a project with everything up-to-date becomes a lot faster.
	///
	/// See more: https://pnpm.io/settings#optimisticrepeatinstall
	#[serde(alias = "optimistic_repeat_install")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub optimistic_repeat_install: Option<bool>,

	/// Check if current branch is your publish branch, clean, and up-to-date with remote.
	///
	/// See more: https://pnpm.io/cli/publish#configuration
	#[serde(alias = "git_checks")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub git_checks: Option<bool>,

	/// The primary branch of the repository which is used for publishing the latest changes.
	///
	/// See more: https://pnpm.io/cli/publish#configuration
	#[serde(alias = "publish_branch")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub publish_branch: Option<String>,

	/// The location of the local pnpmfile.
	///
	/// See more: https://pnpm.io/settings#pnpmfile
	#[serde(rename = "pnpmfile", alias = "pnpm_file")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub pnpm_file: Option<PathBuf>,

	/// The location of a global pnpmfile. A global pnpmfile is used by all projects during installation.
	///
	/// See more: https://pnpm.io/settings#globalpnpmfile
	#[serde(rename = "globalPnpmfile", alias = "global_pnpm_file")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub global_pnpm_file: Option<PathBuf>,

	/// .pnpmfile.cjs will be ignored. Useful together with --ignore-scripts when you want to make sure that no script gets executed during install.
	///
	/// See more: https://pnpm.io/settings#ignorepnpmfile
	#[serde(rename = "ignorePnpmfile", alias = "ignore_pnpm_file")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub ignore_pnpm_file: Option<bool>,

	/// The generated patch file will be saved to this directory.
	///
	/// See more: https://pnpm.io/cli/patch-commit
	#[serde(alias = "patches_dir")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub patches_dir: Option<PathBuf>,

	/// When true, pnpm will run any pre/post scripts automatically.
	///
	/// See more: https://pnpm.io/settings#enableprepostscripts
	#[serde(alias = "enable_pre_post_scripts")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub enable_pre_post_scripts: Option<bool>,

	/// The shell to use for scripts run with the pnpm run command.
	///
	/// See more: https://pnpm.io/settings#scriptshell
	#[serde(alias = "script_shell")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub script_shell: Option<String>,

	/// When true, pnpm will use a JavaScript implementation of a bash-like shell to execute scripts.
	///
	/// See more: https://pnpm.io/settings#shellemulator
	#[serde(alias = "shell_emulator")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub shell_emulator: Option<bool>,

	/// Saved dependencies will be configured with an exact version rather than using pnpm's default semver range operator.
	///
	/// See more: https://pnpm.io/cli/add#--save-exact--e
	#[serde(alias = "save_exact")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub save_exact: Option<bool>,

	#[serde(flatten)]
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	pub extra: JsonValueBTreeMap,
}

impl PnpmWorkspace {
	/// A helper to add all [`PackageJson`] dependencies (dev, optional, peer, etc) marked with `catalog:` to the pnpm catalogs.
	pub async fn add_dependencies_to_catalog(
		&mut self,
		range_kind: VersionRange,
		package_json: &PackageJson,
	) -> Result<(), GenError> {
		let names_to_add: Vec<(String, Option<String>)> = package_json
			.dependencies
			.iter()
			.chain(package_json.dev_dependencies.iter())
			.chain(package_json.peer_dependencies.iter())
			.chain(package_json.optional_dependencies.iter())
			.filter_map(|(name, version)| match CATALOG_REGEX.captures(version) {
				Some(captures) => {
					let catalog_name = captures
						.name("name")
						.map(|n| n.as_str().to_string());
					Some((name.clone(), catalog_name))
				}
				None => None,
			})
			.collect();

		self.add_names_to_catalog(range_kind, names_to_add)
			.await
	}

	/// A helper to add several dependencies to one of this config's catalog.
	pub async fn add_names_to_catalog(
		&mut self,
		range_kind: VersionRange,
		entries: Vec<(String, Option<String>)>,
	) -> Result<(), GenError> {
		let handles = entries
			.into_iter()
			.map(|(name, catalog_name)| async move {
				let actual_latest = get_latest_npm_version(&name).await?;

				Ok((name, catalog_name, actual_latest))
			});

		let stream = stream::iter(handles).buffer_unordered(10);

		#[allow(clippy::type_complexity)]
		let results: Vec<Result<(String, Option<String>, String), GenError>> = stream.collect().await;

		for result in results {
			match result {
				Ok((name, catalog_name, actual_latest)) => {
					let target_catalog = if let Some(catalog_name) = catalog_name {
						self.catalogs
							.entry(catalog_name.as_str().to_string())
							.or_default()
					} else {
						&mut self.catalog
					};

					let range = range_kind.create(actual_latest);

					target_catalog.insert(name.clone(), range);
				}
				Err(e) => return Err(e),
			};
		}

		Ok(())
	}
}
