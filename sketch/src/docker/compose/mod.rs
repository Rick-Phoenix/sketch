use std::{
	cmp::Ordering,
	collections::{BTreeMap, BTreeSet},
};

use indexmap::{IndexMap, IndexSet};
use merge::Merge;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
	docker::compose::service::{DockerServicePreset, ServiceData},
	merge_btree_maps,
	serde_utils::SingleValue,
};

pub mod service;

use crate::{
	Extensible, GenError, Preset, StringBTreeMap, merge_index_sets, merge_nested,
	merge_optional_btree_maps, merge_optional_btree_sets, merge_presets, overwrite_if_some,
	serde_utils::{ListOrMap, StringOrList, StringOrNum, StringOrSortedList},
};

/// A preset for Docker Compose files.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Default, Merge)]
#[serde(default)]
pub struct ComposePreset {
	/// The list of extended presets.
	#[merge(strategy = merge_index_sets)]
	pub extends_presets: IndexSet<String>,

	#[serde(flatten)]
	#[merge(strategy = merge_nested)]
	pub config: ComposeFile,
}

impl Extensible for ComposePreset {
	fn get_extended(&self) -> &IndexSet<String> {
		&self.extends_presets
	}
}

impl ComposePreset {
	pub fn process_data(
		self,
		id: &str,
		store: &IndexMap<String, Self>,
		services_store: &IndexMap<String, DockerServicePreset>,
	) -> Result<ComposeFile, GenError> {
		let mut processed_ids: IndexSet<String> = IndexSet::new();

		// Must not skip here in case of no extended presets, because services must be processed regardless
		let merged_preset = if self.extends_presets.is_empty() {
			self
		} else {
			merge_presets(Preset::DockerCompose, id, self, store, &mut processed_ids)?
		};

		let mut config = merged_preset.config;

		for (_, service_data) in config.services.iter_mut() {
			match service_data {
				ServiceData::Id(id) => {
					let preset = services_store
						.get(id)
						.ok_or(GenError::PresetNotFound {
							kind: Preset::DockerService,
							name: id.clone(),
						})?
						.clone();

					*service_data = ServiceData::Config(preset.process_data(id, services_store)?);
				}
				ServiceData::Config(config) => {
					if !config.extends_presets.is_empty() {
						*service_data = ServiceData::Config(
							config
								.clone()
								.process_data("__inlined", services_store)?,
						);
					}
				}
			};
		}

		Ok(config)
	}
}

/// Configuration settings for a Docker Compose file.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, Default, Merge)]
#[merge(strategy = overwrite_if_some)]
#[serde(default)]
pub struct ComposeFile {
	/// The top-level name property is defined by the Compose Specification as the project name to be used if you don't set one explicitly.
	///
	/// See more: https://docs.docker.com/reference/compose-file/version-and-name/#name-top-level-element
	#[serde(skip_serializing_if = "Option::is_none")]
	pub name: Option<String>,

	/// Requires: Docker Compose 2.20.0 and later
	///
	/// The include top-level section is used to define the dependency on another Compose application, or sub-domain. Each path listed in the include section is loaded as an individual Compose application model, with its own project directory, in order to resolve relative paths.
	///
	/// See more: https://docs.docker.com/reference/compose-file/include/
	#[serde(skip_serializing_if = "Option::is_none")]
	#[merge(strategy = merge_optional_btree_sets)]
	pub include: Option<BTreeSet<Include>>,

	/// Defines the services for the Compose application.
	///
	/// See more: https://docs.docker.com/reference/compose-file/services/
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	#[merge(strategy = merge_btree_maps)]
	pub services: BTreeMap<String, ServiceData>,

	/// Defines or references configuration data that is granted to services in your Compose application.
	///
	/// See more: https://docs.docker.com/reference/compose-file/configs/
	#[serde(skip_serializing_if = "Option::is_none")]
	#[merge(strategy = merge_optional_btree_maps)]
	pub configs: Option<BTreeMap<String, TopLevelConfig>>,

	/// The top-level models section declares AI models that are used by your Compose application. These models are typically pulled as OCI artifacts, run by a model runner, and exposed as an API that your service containers can consume.
	///
	/// See more: https://docs.docker.com/reference/compose-file/models/
	#[serde(skip_serializing_if = "Option::is_none")]
	#[merge(strategy = merge_optional_btree_maps)]
	pub models: Option<BTreeMap<String, TopLevelModel>>,

	/// The named networks for the Compose application.
	///
	/// See more: https://docs.docker.com/reference/compose-file/networks/
	#[serde(skip_serializing_if = "Option::is_none")]
	#[merge(strategy = merge_optional_btree_maps)]
	pub networks: Option<BTreeMap<String, TopLevelNetwork>>,

	/// The named secrets for the Compose application.
	///
	/// See more: https://docs.docker.com/reference/compose-file/secrets/
	#[serde(skip_serializing_if = "Option::is_none")]
	#[merge(strategy = merge_optional_btree_maps)]
	pub secrets: Option<BTreeMap<String, TopLevelSecret>>,

	/// The named volumes for the Compose application.
	///
	/// See more: https://docs.docker.com/reference/compose-file/volumes/
	#[serde(skip_serializing_if = "Option::is_none")]
	#[merge(strategy = merge_optional_btree_maps)]
	pub volumes: Option<BTreeMap<String, TopLevelVolume>>,

	#[serde(flatten, skip_serializing_if = "Option::is_none")]
	#[merge(strategy = merge_optional_btree_maps)]
	pub extensions: Option<BTreeMap<String, Value>>,
}

impl ComposeFile {
	pub fn new() -> Self {
		Default::default()
	}
}

/// Compose application or sub-projects to be included.
///
/// See more: https://docs.docker.com/reference/compose-file/include/#long-syntax
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, JsonSchema, Default, Eq)]
#[serde(default)]
pub struct IncludeSettings {
	/// Defines the location of the Compose file(s) to be parsed and included into the local Compose model.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub path: Option<StringOrSortedList>,

	/// Defines a base path to resolve relative paths set in the Compose file. It defaults to the directory of the included Compose file.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub project_directory: Option<String>,

	/// Defines an environment file(s) to use to define default values when interpolating variables in the Compose file being parsed. It defaults to .env file in the project_directory for the Compose file being parsed.
	///
	/// See more: https://docs.docker.com/reference/compose-file/include/#env_file
	#[serde(skip_serializing_if = "Option::is_none")]
	pub env_file: Option<StringOrList>,
}

impl PartialOrd for IncludeSettings {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for IncludeSettings {
	fn cmp(&self, other: &Self) -> Ordering {
		self.path.cmp(&other.path)
	}
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, JsonSchema, PartialOrd, Ord, Eq)]
#[serde(untagged)]
pub enum Include {
	Short(String),
	Long(IncludeSettings),
}

impl Default for Include {
	fn default() -> Self {
		Self::Short(Default::default())
	}
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum Ulimit {
	Single(StringOrNum),
	SoftHard {
		soft: StringOrNum,
		hard: StringOrNum,
	},
}

/// Network configuration for the Compose application.
///
/// See more: https://docs.docker.com/reference/compose-file/networks/
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct TopLevelNetwork {
	/// If set to true, it specifies that this network’s lifecycle is maintained outside of that of the application. Compose doesn't attempt to create these networks, and returns an error if one doesn't exist.
	///
	/// See more: https://docs.docker.com/reference/compose-file/networks/#external
	#[serde(skip_serializing_if = "Option::is_none")]
	pub external: Option<bool>,
	/// Custom name for this network.
	///
	/// See more: https://docs.docker.com/reference/compose-file/networks/#name

	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	pub name: Option<String>,

	/// By default, Compose provides external connectivity to networks. internal, when set to true, lets you create an externally isolated network.
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	pub internal: Option<bool>,

	/// Specifies which driver should be used for this network. Compose returns an error if the driver is not available on the platform.
	///
	/// For more information on drivers and available options, see [Network drivers](https://docs.docker.com/engine/network/drivers/).
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	pub driver: Option<String>,

	/// If `attachable` is set to `true`, then standalone containers should be able to attach to this network, in addition to services. If a standalone container attaches to the network, it can communicate with services and other standalone containers that are also attached to the network.
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	pub attachable: Option<bool>,

	/// Can be used to disable IPv4 address assignment.
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	enable_ipv4: Option<bool>,

	/// Enables IPv6 address assignment.
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	pub enable_ipv6: Option<bool>,

	/// Specifies a custom IPAM configuration.
	///
	/// See more: https://docs.docker.com/reference/compose-file/networks/#ipam
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	pub ipam: Option<Ipam>,

	/// A list of options as key-value pairs to pass to the driver. These options are driver-dependent.
	///
	/// Consult the [network drivers documentation](https://docs.docker.com/engine/network/) for more information.
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	pub driver_opts: Option<BTreeMap<String, Option<SingleValue>>>,

	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	pub labels: Option<ListOrMap>,
}

/// Specifies a custom IPAM configuration.
///
/// See more: https://docs.docker.com/reference/compose-file/networks/#ipam
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct Ipam {
	/// Custom IPAM driver, instead of the default.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub driver: Option<String>,

	/// A list with zero or more configuration elements.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub config: Option<BTreeSet<IpamConfig>>,

	/// Driver-specific options as a key-value mapping.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub options: Option<StringBTreeMap>,
}

/// IPAM specific configurations.
///
/// See more: https://docs.docker.com/reference/compose-file/networks/#ipam
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct IpamConfig {
	/// Subnet in CIDR format that represents a network segment
	#[serde(skip_serializing_if = "Option::is_none")]
	pub subnet: Option<String>,

	/// Range of IPs from which to allocate container IPs.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub ip_range: Option<String>,

	/// IPv4 or IPv6 gateway for the master subnet
	#[serde(skip_serializing_if = "Option::is_none")]
	pub gateway: Option<String>,

	/// Auxiliary IPv4 or IPv6 addresses used by Network driver.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub aux_addresses: Option<StringBTreeMap>,
}

impl PartialOrd for IpamConfig {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for IpamConfig {
	fn cmp(&self, other: &Self) -> Ordering {
		self.subnet.cmp(&other.subnet)
	}
}

/// Specifies a service discovery method for external clients connecting to a service. See more: https://docs.docker.com/reference/compose-file/deploy/#endpoint_mode
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum EndpointMode {
	/// Assigns the service a virtual IP (VIP) that acts as the front end for clients to reach the service on a network. Platform routes requests between the client and nodes running the service, without client knowledge of how many nodes are participating in the service or their IP addresses or ports.
	Vip,

	/// Platform sets up DNS entries for the service such that a DNS query for the service name returns a list of IP addresses (DNS round-robin), and the client connects directly to one of these.
	Dnsrr,
}

/// Defines the replication model used to run a service or job. See more: https://docs.docker.com/reference/compose-file/deploy/#mode
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum DeployMode {
	/// Ensures exactly one task continuously runs per physical node until stopped.
	Global,

	/// Continuously runs a specified number of tasks across nodes until stopped (default).
	Replicated,

	/// Executes a defined number of tasks until a completion state (exits with code 0)'.
	/// Total tasks are determined by replicas.
	/// Concurrency can be limited using the max-concurrent option (CLI only).
	ReplicatedJob,

	/// Executes one task per physical node with a completion state (exits with code 0).
	/// Automatically runs on new nodes as they are added.
	GlobalJob,
}

/// Compose Deploy Specification https://docs.docker.com/reference/compose-file/deploy
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct Deploy {
	/// Specifies a service discovery method for external clients connecting to a service. See more: https://docs.docker.com/reference/compose-file/deploy/#endpoint_mode
	#[serde(skip_serializing_if = "Option::is_none")]
	pub endpoint_mode: Option<EndpointMode>,

	/// Defines the replication model used to run a service or job. See more: https://docs.docker.com/reference/compose-file/deploy/#mode
	#[serde(skip_serializing_if = "Option::is_none")]
	pub mode: Option<DeployMode>,

	/// If the service is replicated (which is the default), replicas specifies the number of containers that should be running at any given time. See more: https://docs.docker.com/reference/compose-file/deploy/#replicas
	#[serde(skip_serializing_if = "Option::is_none")]
	pub replicas: Option<i64>,

	/// Specifies metadata for the service. These labels are only set on the service and not on any containers for the service. This assumes the platform has some native concept of "service" that can match the Compose application model. See more: https://docs.docker.com/reference/compose-file/deploy/#labels
	#[serde(skip_serializing_if = "Option::is_none")]
	pub labels: Option<ListOrMap>,

	/// Configures how the service should be rolled back in case of a failing update. See more: https://docs.docker.com/reference/compose-file/deploy/#rollback_config
	#[serde(skip_serializing_if = "Option::is_none")]
	pub rollback_config: Option<RollbackConfig>,

	/// Configures how the service should be updated. Useful for configuring rolling updates. See more: https://docs.docker.com/reference/compose-file/deploy/#update_config
	#[serde(skip_serializing_if = "Option::is_none")]
	pub update_config: Option<UpdateConfig>,

	/// Configures physical resource constraints for container to run on platform. See more: https://docs.docker.com/reference/compose-file/deploy/#resources
	#[serde(skip_serializing_if = "Option::is_none")]
	pub resources: Option<Resources>,

	/// Configures if and how to restart containers when they exit. If restart_policy is not set, Compose considers the restart field set by the service configuration. See more: https://docs.docker.com/reference/compose-file/deploy/#restart_policy
	#[serde(skip_serializing_if = "Option::is_none")]
	pub restart_policy: Option<RestartPolicy>,

	/// Specifies constraints and preferences for the platform to select a physical node to run service containers. See more: https://docs.docker.com/reference/compose-file/deploy/#placement
	#[serde(skip_serializing_if = "Option::is_none")]
	pub placement: Option<Placement>,
}

/// Resource constraints and reservations for the service.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct Limits {
	/// Limit for how much of the available CPU resources, as number of cores, a container can use.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub cpus: Option<StringOrNum>,

	/// Limit on the amount of memory a container can allocate (e.g., '1g', '1024m').
	#[serde(skip_serializing_if = "Option::is_none")]
	pub memory: Option<String>,

	/// Maximum number of PIDs available to the container.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub pids: Option<StringOrNum>,
}

/// Resource reservations for the service containers.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema, Default)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct Reservations {
	/// Reservation for how much of the available CPU resources, as number of cores, a container can use.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub cpus: Option<StringOrNum>,

	/// Reservation on the amount of memory a container can allocate (e.g., '1g', '1024m').
	#[serde(skip_serializing_if = "Option::is_none")]
	pub memory: Option<String>,

	/// User-defined resources to reserve.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub generic_resources: Option<BTreeSet<GenericResource>>,

	/// Device reservations for the container.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub devices: Option<BTreeSet<Device>>,
}

/// User-defined resources for services, allowing services to reserve specialized hardware resources.
#[derive(
	Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema, Default, PartialOrd, Ord,
)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct GenericResource {
	/// Specification for discrete (countable) resources.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub discrete_resource_spec: Option<DiscreteResourceSpec>,
}

/// Specification for discrete (countable) resources.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema, Default)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct DiscreteResourceSpec {
	/// Type of resource (e.g., 'GPU', 'FPGA', 'SSD').
	#[serde(skip_serializing_if = "Option::is_none")]
	pub kind: Option<String>,

	/// Number of resources of this kind to reserve.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub value: Option<StringOrNum>,
}

impl PartialOrd for DiscreteResourceSpec {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for DiscreteResourceSpec {
	fn cmp(&self, other: &Self) -> Ordering {
		self.kind.cmp(&other.kind)
	}
}

/// Device reservations for containers, allowing services to access specific hardware devices.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Device {
	/// Device driver to use (e.g., 'nvidia').
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	pub driver: Option<String>,

	/// Number of devices of this type to reserve.
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	pub count: Option<StringOrNum>,

	/// List of specific device IDs to reserve.
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	pub device_ids: Option<BTreeSet<String>>,

	/// List of capabilities the device needs to have (e.g., 'gpu', 'compute', 'utility').
	#[serde(skip_serializing_if = "BTreeSet::is_empty")]
	pub capabilities: BTreeSet<String>,

	/// Driver-specific options for the device.
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	pub options: Option<ListOrMap>,
}

impl PartialOrd for Device {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for Device {
	fn cmp(&self, other: &Self) -> Ordering {
		self.driver.cmp(&other.driver)
	}
}

/// Specifies constraints and preferences for the platform to select a physical node to run service containers. See more: https://docs.docker.com/reference/compose-file/deploy/#placement
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema, Default)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct Placement {
	/// Defines a required property the platform's node must fulfill to run the service container.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub constraints: Option<BTreeSet<String>>,

	/// Defines a strategy (currently spread is the only supported strategy) to spread tasks evenly over the values of the datacenter node label.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub preferences: Option<BTreeSet<Preferences>>,
}

/// Defines a strategy (currently spread is the only supported strategy) to spread tasks evenly over the values of the datacenter node label.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct Preferences {
	pub spread: String,
}

/// Configures physical resource constraints for container to run on platform.
///
/// See more: https://docs.docker.com/reference/compose-file/deploy/#resources
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct Resources {
	/// The platform must prevent the container from allocating more resources.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub limits: Option<Limits>,
	/// The platform must guarantee the container can allocate at least the configured amount.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub reservations: Option<Reservations>,
}

/// The condition that should trigger a restart.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum RestartPolicyCondition {
	/// Containers are not automatically restarted regardless of the exit status.
	None,

	/// The container is restarted if it exits due to an error, which manifests as a non-zero exit code.
	OnFailure,

	/// (default) Containers are restarted regardless of the exit status.
	Any,
}

/// Configures if and how to restart containers when they exit. If restart_policy is not set, Compose considers the restart field set by the service configuration.
///
/// See more: https://docs.docker.com/reference/compose-file/deploy/#restart_policy
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema, Default)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct RestartPolicy {
	/// The condition that should trigger a restart.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub condition: Option<RestartPolicyCondition>,

	/// How long to wait between restart attempts, specified as a duration. The default is 0, meaning restart attempts can occur immediately.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub delay: Option<String>,

	/// The maximum number of failed restart attempts allowed before giving up. (Default: unlimited retries.) A failed attempt only counts toward max_attempts if the container does not successfully restart within the time defined by window. For example, if max_attempts is set to 2 and the container fails to restart within the window on the first try, Compose continues retrying until two such failed attempts occur, even if that means trying more than twice.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub max_attempts: Option<i64>,

	/// The amount of time to wait after a restart to determine whether it was successful, specified as a duration (default: the result is evaluated immediately after the restart).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub window: Option<String>,
}

/// What to do if an update fails.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum UpdateFailureAction {
	Continue,
	Rollback,
	Pause,
}

/// What to do if an update fails.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum RollbackFailureAction {
	Continue,
	Pause,
}

/// What to do if an update fails.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum OperationsOrder {
	StartFirst,
	StopFirst,
}

/// Configures how the service should be rolled back in case of a failing update.
///
/// See more: https://docs.docker.com/reference/compose-file/deploy/#rollback_config
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, Default)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct RollbackConfig {
	/// The number of containers to rollback at a time.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub parallelism: Option<i64>,
	/// The time to wait between each container group's rollback
	#[serde(skip_serializing_if = "Option::is_none")]
	pub delay: Option<String>,
	/// What to do if a rollback fails. One of continue or pause (default pause)
	#[serde(skip_serializing_if = "Option::is_none")]
	pub failure_action: Option<RollbackFailureAction>,
	/// Duration after each task update to monitor for failure (ns|us|ms|s|m|h) (default 0s).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub monitor: Option<String>,
	/// Failure rate to tolerate during a rollback.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub max_failure_ratio: Option<f64>,

	/// Order of operations during rollbacks. One of stop-first (old task is stopped before starting new one), or start-first (new task is started first, and the running tasks briefly overlap) (default stop-first).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub order: Option<OperationsOrder>,
}

/// Configures how the service should be updated. Useful for configuring rolling updates.
///
/// See more: https://docs.docker.com/reference/compose-file/deploy/#update_config
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct UpdateConfig {
	/// The number of containers to update at a time.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub parallelism: Option<i64>,
	/// The time to wait between updating a group of containers.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub delay: Option<String>,
	/// What to do if an update fails. One of continue, rollback, or pause (default: pause).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub failure_action: Option<UpdateFailureAction>,
	/// Duration after each task update to monitor for failure (ns|us|ms|s|m|h) (default 0s).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub monitor: Option<String>,
	/// Failure rate to tolerate during an update.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub max_failure_ratio: Option<f64>,

	/// Order of operations during updates. One of stop-first (old task is stopped before starting new one), or start-first (new task is started first, and the running tasks briefly overlap) (default stop-first).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub order: Option<OperationsOrder>,
}

/// Secret configuration for the Compose application.
///
/// See more: https://docs.docker.com/reference/compose-file/secrets/
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum TopLevelSecret {
	/// Path to a file containing the secret value.
	File(String),
	/// Path to a file containing the secret value.
	Environment(String),
	#[serde(untagged)]
	External {
		/// Specifies that this secret already exists and was created outside of Compose.
		external: bool,
		/// Specifies the name of the external secret.
		name: String,
	},
}

/// Configuration for service configs or secrets, defining how they are mounted in the container.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema, PartialOrd, Ord)]
#[serde(untagged)]
pub enum ServiceConfigOrSecret {
	/// Name of the config or secret to grant access to.
	String(String),
	/// Detailed configuration for a config or secret.
	Advanced(ServiceConfigOrSecretSettings),
}

impl PartialOrd for ServiceConfigOrSecretSettings {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for ServiceConfigOrSecretSettings {
	fn cmp(&self, other: &Self) -> Ordering {
		self.source.cmp(&other.source)
	}
}

/// Configuration for service configs or secrets, defining how they are mounted in the container.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ServiceConfigOrSecretSettings {
	/// Name of the config or secret as defined in the top-level configs or secrets section.
	pub source: String,

	/// Path in the container where the config or secret will be mounted. Defaults to /<source> for configs and /run/secrets/<source> for secrets.
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	pub target: Option<String>,

	/// UID of the file in the container. Default is 0 (root).
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	pub uid: Option<String>,

	/// GID of the file in the container. Default is 0 (root).
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	pub gid: Option<String>,

	/// File permission mode inside the container, in octal. Default is 0444 for configs and 0400 for secrets.
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	pub mode: Option<StringOrNum>,
}

/// Defines or references configuration data that is granted to services in your Compose application.
///
/// See more: https://docs.docker.com/reference/compose-file/configs/
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default, Eq, JsonSchema)]
#[serde(default)]
pub struct TopLevelConfig {
	/// The name of the config object in the container engine to look up. This field can be used to reference configs that contain special characters. The name is used as is and will not be scoped with the project name.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub name: Option<String>,

	/// If set to true, external specifies that this config has already been created. Compose does not attempt to create it, and if it does not exist, an error occurs.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub external: Option<bool>,

	/// The content is created with the inlined value. Introduced in Docker Compose version 2.23.1.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub content: Option<String>,

	/// The config content is created with the value of an environment variable.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub environment: Option<String>,

	/// The config is created with the contents of the file at the specified path.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub file: Option<String>,
}

/// Adds hostname mappings to the container network interface configuration (/etc/hosts for Linux).
///
/// See more: https://docs.docker.com/reference/compose-file/services/#extra_hosts
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord, JsonSchema)]
#[serde(untagged)]
pub enum ExtraHosts {
	/// List of host:IP mappings in the format 'hostname:IP'.
	List(BTreeSet<String>),

	/// List mapping hostnames to IP addresses.
	Map(BTreeMap<String, StringOrSortedList>),
}

pub(crate) fn merge_extra_hosts(left: &mut Option<ExtraHosts>, right: Option<ExtraHosts>) {
	if let Some(right) = right {
		if let Some(left_data) = left {
			if let ExtraHosts::List(left_list) = left_data
				&& let ExtraHosts::List(right_list) = right
			{
				left_list.extend(right_list);
			} else if let ExtraHosts::Map(left_map) = left_data
				&& let ExtraHosts::Map(right_map) = right
			{
				left_map.extend(right_map);
			} else {
				*left = Some(right);
			}
		} else {
			*left = Some(right);
		}
	}
}

/// Language Model for the Compose application.
///
/// See more: https://docs.docker.com/reference/compose-file/models/
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct TopLevelModel {
	/// Language Model to run.
	pub model: String,

	/// Custom name for this model.
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	pub name: Option<String>,

	/// The context window size for the model.
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	pub context_size: Option<u64>,

	/// Raw runtime flags to pass to the inference engine.
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	pub runtime_flags: Option<Vec<String>>,
}

impl PartialOrd for TopLevelModel {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for TopLevelModel {
	fn cmp(&self, other: &Self) -> Ordering {
		self.name
			.cmp(&other.name)
			.then_with(|| self.model.cmp(&other.model))
	}
}

/// Volume configuration for the Compose application.
///
/// See more: https://docs.docker.com/reference/compose-file/volumes/
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct TopLevelVolume {
	/// If set to true, it specifies that this volume already exists on the platform and its lifecycle is managed outside of that of the application.
	///
	/// See more: https://docs.docker.com/reference/compose-file/volumes/#external
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub external: Option<bool>,

	/// Sets a custom name for a volume.
	///
	/// See more: https://docs.docker.com/reference/compose-file/volumes/#name
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	pub name: Option<String>,

	/// Specifies which volume driver should be used. If the driver is not available, Compose returns an error and doesn't deploy the application.
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	pub driver: Option<String>,

	/// Specifies a list of options as key-value pairs to pass to the driver for this volume. The options are driver-dependent.
	#[serde(skip_serializing_if = "BTreeMap::is_empty")]
	#[serde(default)]
	pub driver_opts: BTreeMap<String, SingleValue>,

	/// Labels are used to add metadata to volumes. You can use either an array or a dictionary.
	///
	/// It's recommended that you use reverse-DNS notation to prevent your labels from conflicting with those used by other software.
	///
	/// See more: https://docs.docker.com/reference/compose-file/volumes/#labels
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	pub labels: Option<ListOrMap>,
}
