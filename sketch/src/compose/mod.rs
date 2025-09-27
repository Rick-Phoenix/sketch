use std::{
  cmp::Ordering,
  collections::{BTreeMap, BTreeSet},
  fmt,
};

use indexmap::IndexMap;
use merge::Merge;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
  merge_optional_btree_maps, merge_optional_btree_sets, merge_optional_vecs, overwrite_if_some,
  StringBTreeMap,
};

/// Configuration settings for a Docker Compose file.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, Default, Merge)]
#[merge(strategy = overwrite_if_some)]
pub struct Compose {
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
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_btree_maps)]
  pub services: Option<BTreeMap<String, Service>>,

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

impl Compose {
  pub fn new() -> Self {
    Default::default()
  }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema, PartialOrd, Ord)]
#[serde(untagged)]
pub enum StringOrList {
  String(String),
  List(Vec<String>),
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema, PartialOrd, Ord)]
#[serde(untagged)]
pub enum StringOrNum {
  Num(i64),
  String(String),
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema, PartialOrd, Ord)]
#[serde(untagged)]
pub enum ListOrMap {
  List(BTreeSet<String>),
  Map(BTreeMap<String, String>),
}

fn merge_list_or_map(left: &mut Option<ListOrMap>, right: Option<ListOrMap>) {
  if let Some(right) = right {
    if let Some(left_data) = left {
      match left_data {
        ListOrMap::List(left_list) => match right {
          ListOrMap::List(right_list) => left_list.extend(right_list),
          _ => {}
        },
        ListOrMap::Map(left_map) => match right {
          ListOrMap::Map(right_map) => left_map.extend(right_map),
          _ => {}
        },
      }
    } else {
      *left = Some(right);
    }
  }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema, PartialOrd, Ord)]
#[serde(untagged)]
pub enum StringOrSortedList {
  String(String),
  List(BTreeSet<String>),
}

fn merge_string_or_sorted_list(
  left: &mut Option<StringOrSortedList>,
  right: Option<StringOrSortedList>,
) {
  if let Some(right) = right {
    if let Some(left_data) = left {
      if let StringOrSortedList::List(left_list) = left_data && let StringOrSortedList::List(right_list) = right  {
        left_list.extend(right_list);
      } else {
        *left = Some(right);
      }
    } else {
      *left = Some(right);
    }
  }
}

impl Default for StringOrSortedList {
  fn default() -> Self {
    Self::String(String::new())
  }
}

/// Compose application or sub-projects to be included.
///
/// See more: https://docs.docker.com/reference/compose-file/include/#long-syntax
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, JsonSchema, Default, Eq)]
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
    Some(self.cmp(&other))
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

/// Defines a service for a Compose application.
///
/// See more: https://docs.docker.com/reference/compose-file/services/
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, JsonSchema, Default, Merge)]
#[merge(strategy = overwrite_if_some)]
pub struct Service {
  /// `extends` lets you share common configurations among different files, or even different projects entirely.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/services/#extends
  #[serde(skip_serializing_if = "Option::is_none")]
  pub extends: Option<Extends>,

  /// A string that specifies a custom container name, rather than a name generated by default. Compose does not scale a service beyond one container if the Compose file specifies a container_name. Attempting to do so results in an error.
  ///
  /// container_name follows the regex format of [a-zA-Z0-9][a-zA-Z0-9_.-]+
  ///
  /// See more: https://docs.docker.com/reference/compose-file/services/#container_name
  #[serde(skip_serializing_if = "Option::is_none")]
  pub container_name: Option<String>,

  /// A custom host name to use for the service container. It must be a valid RFC 1123 hostname.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub hostname: Option<String>,

  /// Specifies the image to start the container from. See more: https://docs.docker.com/reference/compose-file/services/#image
  #[serde(skip_serializing_if = "Option::is_none")]
  pub image: Option<String>,

  /// Specifies the build configuration for creating a container image from source, as defined in the [Compose Build Specification](https://docs.docker.com/reference/compose-file/build/).
  #[serde(skip_serializing_if = "Option::is_none", rename = "build")]
  #[merge(strategy = merge_build_step)]
  pub build_: Option<BuildStep>,

  /// With the depends_on attribute, you can control the order of service startup and shutdown. It is useful if services are closely coupled, and the startup sequence impacts the application's functionality.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/services/#depends_on
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_depends_on)]
  pub depends_on: Option<DependsOn>,

  /// Overrides the default command declared by the container image, for example by Dockerfile's CMD.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/services/#command
  #[serde(skip_serializing_if = "Option::is_none")]
  pub command: Option<StringOrList>,

  /// Declares the default entrypoint for the service container. This overrides the ENTRYPOINT instruction from the service's Dockerfile.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/services/#entrypoint
  #[serde(skip_serializing_if = "Option::is_none")]
  pub entrypoint: Option<StringOrList>,

  /// Declares a check that's run to determine whether or not the service containers are "healthy".
  ///
  /// See more: https://docs.docker.com/reference/compose-file/services/#healthcheck
  #[serde(skip_serializing_if = "Option::is_none")]
  pub healthcheck: Option<Healthcheck>,

  /// Defines the (incoming) port or a range of ports that Compose exposes from the container. These ports must be accessible to linked services and should not be published to the host machine. Only the internal container ports can be specified.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/services/#expose
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_btree_sets)]
  pub expose: Option<BTreeSet<StringOrNum>>,

  /// One or more files that contain environment variables to be passed to the containers.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/services/#env_file
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_envfile)]
  pub env_file: Option<Envfile>,

  /// Defines environment variables set in the container. environment can use either an array or a map. Any boolean values; true, false, yes, no, should be enclosed in quotes to ensure they are not converted to True or False by the YAML parser.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/services/#environment
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_list_or_map)]
  pub environment: Option<ListOrMap>,

  /// Defines annotations for the container. annotations can use either an array or a map.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_list_or_map)]
  pub annotations: Option<ListOrMap>,

  /// Requires: Docker Compose 2.20.0 and later
  ///
  /// When attach is defined and set to false Compose does not collect service logs, until you explicitly request it to.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub attach: Option<bool>,

  /// Defines a set of configuration options to set block I/O limits for a service.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/services/#blkio_config
  #[serde(skip_serializing_if = "Option::is_none")]
  pub blkio_config: Option<BlkioSettings>,

  /// Specifies additional container [capabilities](https://man7.org/linux/man-pages/man7/capabilities.7.html) as strings.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_btree_sets)]
  pub cap_add: Option<BTreeSet<String>>,

  /// Specifies container [capabilities](https://man7.org/linux/man-pages/man7/capabilities.7.html) to drop as strings.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_btree_sets)]
  pub cap_drop: Option<BTreeSet<String>>,

  /// Requires: Docker Compose 2.15.0 and later
  ///
  /// Specifies the cgroup namespace to join. When unset, it is the container runtime's decision to select which cgroup namespace to use, if supported.
  ///
  /// host: Runs the container in the Container runtime cgroup namespace.
  /// private: Runs the container in its own private cgroup namespace.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cgroup: Option<Cgroup>,

  /// Specifies an optional parent cgroup for the container.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cgroup_parent: Option<String>,

  /// Lets services adapt their behaviour without the need to rebuild a Docker image.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/services/#configs
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_btree_sets)]
  pub configs: Option<BTreeSet<ServiceConfigOrSecret>>,

  /// The number of usable CPUs for service container.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cpu_count: Option<StringOrNum>,

  /// The usable percentage of the available CPUs.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cpu_percent: Option<StringOrNum>,

  /// Configures CPU CFS (Completely Fair Scheduler) period when a platform is based on Linux kernel.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cpu_period: Option<StringOrNum>,

  /// Configures CPU CFS (Completely Fair Scheduler) quota when a platform is based on Linux kernel.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cpu_quota: Option<StringOrNum>,

  /// A service container's relative CPU weight versus other containers.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cpu_shares: Option<StringOrNum>,

  /// Configures CPU allocation parameters for platforms with support for real-time scheduler. It can be either an integer value using microseconds as unit or a duration.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cpu_rt_period: Option<StringOrNum>,

  /// Configures CPU allocation parameters for platforms with support for real-time scheduler. It can be either an integer value using microseconds as unit or a duration.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cpu_rt_runtime: Option<StringOrNum>,

  /// The number of (potentially virtual) CPUs to allocate to service containers. This is a fractional number. 0.000 means no limit.
  ///
  /// When set, cpus must be consistent with the cpus attribute in the Deploy Specification.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cpus: Option<StringOrNum>,

  /// The explicit CPUs in which to permit execution. Can be a range 0-3 or a list 0,1
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cpuset: Option<String>,

  /// Configures the credential spec for a managed service account.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/services/#credential_spec
  #[serde(skip_serializing_if = "Option::is_none")]
  pub credential_spec: Option<CredentialSpec>,

  /// Specifies the configuration for the deployment and lifecycle of services, as defined in the [Compose Deploy Specification](https://docs.docker.com/reference/compose-file/deploy)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub deploy: Option<Deploy>,

  /// Specifies the development configuration for maintaining a container in sync with source.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/develop
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_development_settings)]
  pub develop: Option<DevelopmentSettings>,

  /// A list of device cgroup rules for this container. The format is the same format the Linux kernel specifies in the Control [Groups Device Whitelist Controller]().
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_btree_sets)]
  pub device_cgroup_rules: Option<BTreeSet<String>>,

  /// Defines a list of device mappings for created containers.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/services/#devices
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_btree_sets)]
  pub devices: Option<BTreeSet<DeviceMapping>>,

  /// Custom DNS servers to set on the container network interface configuration. It can be a single value or a list.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_string_or_sorted_list)]
  pub dns: Option<StringOrSortedList>,

  /// Custom DNS options to be passed to the container’s DNS resolver (/etc/resolv.conf file on Linux).
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_btree_sets)]
  pub dns_opt: Option<BTreeSet<String>>,

  /// Custom DNS search domains to set on container network interface configuration. It can be a single value or a list.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_string_or_sorted_list)]
  pub dns_search: Option<StringOrSortedList>,

  /// A custom domain name to use for the service container. It must be a valid RFC 1123 hostname.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub domainname: Option<String>,

  /// Requires: Docker Compose 2.27.1 and later
  ///
  /// Specifies a list of options as key-value pairs to pass to the driver. These options are driver-dependent.
  /// Consult the [network drivers documentation](https://docs.docker.com/engine/network/) for more information.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_btree_maps)]
  pub driver_opts: Option<BTreeMap<String, StringOrNum>>,

  /// `external_links` link service containers to services managed outside of your Compose application. `external_links` define the name of an existing service to retrieve using the platform lookup mechanism.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_btree_sets)]
  pub external_links: Option<BTreeSet<String>>,

  /// Adds hostname mappings to the container network interface configuration (/etc/hosts for Linux).
  ///
  /// See more: https://docs.docker.com/reference/compose-file/services/#extra_hosts
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_extra_hosts)]
  pub extra_hosts: Option<ExtraHosts>,

  /// Requires: Docker Compose 2.30.0 and later
  ///
  /// Specifies GPU devices to be allocated for container usage.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_gpus)]
  pub gpus: Option<Gpus>,

  /// Additional groups, by name or number, which the user inside the container must be a member of.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/services/#group_add
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_btree_sets)]
  pub group_add: Option<BTreeSet<StringOrNum>>,

  /// Runs an init process (PID 1) inside the container that forwards signals and reaps processes. Set this option to true to enable this feature for the service.
  ///
  /// The init binary that is used is platform specific.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub init: Option<bool>,

  /// ipc configures the IPC isolation mode set by the service container.
  ///
  /// shareable: Gives the container its own private IPC namespace, with a possibility to share it with other containers.
  ///
  /// service:{name}: Makes the container join another container's (shareable) IPC namespace.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/services/#ipc
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ipc: Option<String>,

  /// Container isolation technology to use. Supported values are platform-specific.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub isolation: Option<String>,

  /// Add metadata to containers. You can use either an array or a map.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/services/#labels
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_list_or_map)]
  pub labels: Option<ListOrMap>,

  /// Requires: Docker Compose 2.32.2 and later
  ///
  ///The label_file attribute lets you load labels for a service from an external file or a list of files. This provides a convenient way to manage multiple labels without cluttering the Compose file.
  /// See more: https://docs.docker.com/reference/compose-file/services/#label_file
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_string_or_sorted_list)]
  pub label_file: Option<StringOrSortedList>,

  /// Defines a network link to containers in another service. Either specify both the service name and a link alias (SERVICE:ALIAS), or just the service name.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/services/#links
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_btree_sets)]
  pub links: Option<BTreeSet<String>>,

  /// Defines the logging configuration for the service.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/services/#logging
  #[serde(skip_serializing_if = "Option::is_none")]
  pub logging: Option<LoggingSettings>,

  /// Sets a Mac address for the service container.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/services/#mac_address
  #[serde(skip_serializing_if = "Option::is_none")]
  pub mac_address: Option<String>,

  /// Memory limit for the container. A string value can use suffix like '2g' for 2 gigabytes.
  ///
  /// When set, mem_limit must be consistent with the limits.memory attribute in the [Deploy Specification](https://docs.docker.com/reference/compose-file/deploy/#memory).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub mem_limit: Option<StringOrNum>,

  /// Configures a reservation on the amount of memory a container can allocate, set as a string expressing a [byte value](https://docs.docker.com/reference/compose-file/extension/#specifying-byte-values).
  ///
  /// When set, mem_reservation must be consistent with the reservations.memory attribute in the [Deploy Specification](https://docs.docker.com/reference/compose-file/deploy/#memory).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub mem_reservation: Option<StringOrNum>,

  /// Defines as a percentage, a value between 0 and 100, for the host kernel to swap out anonymous memory pages used by a container.
  ///
  ///  0: Turns off anonymous page swapping.
  /// 100: Sets all anonymous pages as swappable.
  ///
  /// The default value is platform specific.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub mem_swappiness: Option<u8>,

  /// Defines the amount of memory the container is allowed to swap to disk.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/services/#memswap_limit
  #[serde(skip_serializing_if = "Option::is_none")]
  pub memswap_limit: Option<StringOrNum>,

  /// Requires: Docker Compose 2.38.0 and later
  ///
  /// Defines which AI models the service should use at runtime. Each referenced model must be defined under the [`models` top-level element](https://docs.docker.com/reference/compose-file/models/).
  ///
  /// See more: https://docs.docker.com/reference/compose-file/services/#models
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_service_models)]
  pub models: Option<ServiceModels>,

  /// Sets a service container's network mode.
  ///
  /// none: Turns off all container networking.
  /// host: Gives the container raw access to the host's network interface.
  /// service:{name}: Gives the container access to the specified container by referring to its service name.
  /// container:{name}: Gives the container access to the specified container by referring to its container ID.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/services/#network_mode
  #[serde(skip_serializing_if = "Option::is_none")]
  pub network_mode: Option<NetworkMode>,

  /// The networks attribute defines the networks that service containers are attached to, referencing entries under the networks top-level element.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/services/#networks
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_service_networks)]
  pub networks: Option<ServiceNetworks>,

  /// If `oom_kill_disable` is set, Compose configures the platform so it won't kill the container in case of memory starvation.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub oom_kill_disable: Option<bool>,

  /// Tunes the preference for containers to be killed by platform in case of memory starvation. Value must be within -1000,1000 range.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub oom_score_adj: Option<i32>,

  /// Sets the PID mode for container created by Compose. Supported values are platform specific.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub pid: Option<String>,

  /// Tune a container's PIDs limit. Set to -1 for unlimited PIDs.
  ///
  /// When set, pids_limit must be consistent with the pids attribute in the [Deploy Specification](https://docs.docker.com/reference/compose-file/deploy/#pids).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub pids_limit: Option<i64>,

  /// The target platform the containers for the service run on. It uses the os[/arch[/variant]] syntax.
  ///
  /// The values of os, arch, and variant must conform to the convention used by the OCI Image Spec.
  ///
  /// Compose uses this attribute to determine which version of the image is pulled and/or on which platform the service’s build is performed.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub platform: Option<String>,

  /// Used to define the port mappings between the host machine and the containers.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/services/#ports
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_btree_sets)]
  pub ports: Option<BTreeSet<Port>>,

  /// Requires: Docker Compose 2.30.0 and later
  ///
  ///Defines a sequence of lifecycle hooks to run after a container has started. The exact timing of when the command is run is not guaranteed.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/services/#post_start
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_vecs)]
  pub post_start: Option<Vec<ServiceHook>>,

  /// Defines a sequence of lifecycle hooks to run before the container is stopped. These hooks won't run if the container stops by itself or is terminated suddenly.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_vecs)]
  pub pre_stop: Option<Vec<ServiceHook>>,

  /// Configures the service container to run with elevated privileges. Support and actual impacts are platform specific.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub privileged: Option<bool>,

  /// Defines a list of named profiles for the service to be enabled under. If unassigned, the service is always started but if assigned, it is only started if the profile is activated.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/services/#profiles
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_btree_sets)]
  pub profiles: Option<BTreeSet<String>>,

  /// Defines a service that Compose won't manage directly. Compose delegated the service lifecycle to a dedicated or third-party component.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/services/#provider
  #[serde(skip_serializing_if = "Option::is_none")]
  pub provider: Option<Provider>,

  /// Defines the decisions Compose makes when it starts to pull images.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/services/#pull_policy
  #[serde(skip_serializing_if = "Option::is_none")]
  pub pull_policy: Option<PullPolicy>,

  /// Time after which to refresh the image. Used with pull_policy=refresh.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub pull_refresh_after: Option<String>,

  /// Configures the service container to be created with a read-only filesystem.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub read_only: Option<bool>,

  /// Defines the policy that the platform applies on container termination.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/services/#restart
  #[serde(skip_serializing_if = "Option::is_none")]
  pub restart: Option<Restart>,

  /// Specifies which runtime to use for the service’s containers.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/services/#runtime
  #[serde(skip_serializing_if = "Option::is_none")]
  pub runtime: Option<String>,

  /// Specifies the default number of containers to deploy for this service. When both are set, scale must be consistent with the replicas attribute in the [Deploy Specification](https://docs.docker.com/reference/compose-file/deploy/#replicas).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub scale: Option<u64>,

  /// The secrets attribute grants access to sensitive data defined by the secrets top-level element on a per-service basis. Services can be granted access to multiple secrets.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/services/#secrets
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_btree_sets)]
  pub secrets: Option<BTreeSet<ServiceConfigOrSecret>>,

  /// Overrides the default labeling scheme for each container.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/services/#security_opt
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_btree_sets)]
  pub security_opt: Option<BTreeSet<String>>,

  /// Size of /dev/shm. A string value can use suffix like '2g' for 2 gigabytes.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub shm_size: Option<StringOrNum>,

  /// Configures a service's container to run with an allocated stdin. This is the same as running a container with the -i flag. For more information, see [Keep stdin open](https://docs.docker.com/reference/cli/docker/container/run/#interactive).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub stdin_open: Option<bool>,

  /// Specifies how long Compose must wait when attempting to stop a container if it doesn't handle SIGTERM (or whichever stop signal has been specified with stop_signal), before sending SIGKILL. It's specified as a duration.
  ///
  /// Default value is 10 seconds for the container to exit before sending SIGKILL.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub stop_grace_period: Option<String>,

  /// The signal that Compose uses to stop the service containers. If unset containers are stopped by Compose by sending SIGTERM.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub stop_signal: Option<String>,

  /// Defines storage driver options for a service.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_btree_maps)]
  pub storage_opt: Option<BTreeMap<String, Value>>,

  /// Defines kernel parameters to set in the container. sysctls can use either an array or a map.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/services/#sysctls
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_list_or_map)]
  pub sysctls: Option<ListOrMap>,

  /// Mounts a temporary file system inside the container. It can be a single value or a list.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/services/#tmpfs
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tmpfs: Option<StringOrList>,

  /// Configures a service's container to run with a TTY. This is the same as running a container with the -t or --tty flag. For more information, see [Allocate a pseudo-TTY](https://docs.docker.com/reference/cli/docker/container/run/#tty).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tty: Option<bool>,

  /// Overrides the default ulimits for a container. It's specified either as an integer for a single limit or as mapping for soft/hard limits.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/services/#ulimits
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_btree_maps)]
  pub ulimits: Option<BTreeMap<String, Ulimit>>,

  /// When `use_api_socket` is set, the container is able to interact with the underlying container engine through the API socket. Your credentials are mounted inside the container so the container acts as a pure delegate for your commands relating to the container engine. Typically, commands ran by container can pull and push to your registry.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub use_api_socket: Option<bool>,

  /// Overrides the user used to run the container process. The default is set by the image, for example Dockerfile USER. If it's not set, then root.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub user: Option<String>,

  /// Sets the user namespace for the service. Supported values are platform specific and may depend on platform configuration.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub userns_mode: Option<String>,

  /// /// Configures the UTS namespace mode set for the service container. When unspecified it is the runtime's decision to assign a UTS namespace, if supported.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub uts: Option<Uts>,

  /// The volumes attribute define mount host paths or named volumes that are accessible by service containers.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/services/#volumes
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_vecs)]
  pub volumes: Option<Vec<ServiceVolume>>,

  /// Mounts all of the volumes from another service or container. You can optionally specify read-only access ro or read-write rw. If no access level is specified, then read-write access is used.
  ///
  /// You can also mount volumes from a container that is not managed by Compose by using the container: prefix.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_vecs)]
  pub volumes_from: Option<Vec<String>>,

  /// Overrides the container's working directory which is specified by the image, for example Dockerfile's WORKDIR.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub working_dir: Option<String>,
}

/// With the depends_on attribute, you can control the order of service startup and shutdown. It is useful if services are closely coupled, and the startup sequence impacts the application's functionality.
///
/// See more: https://docs.docker.com/reference/compose-file/services/#depends_on
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum DependsOn {
  Simple(Vec<String>),

  Conditional(IndexMap<String, DependsOnSettings>),
}

fn merge_depends_on(left: &mut Option<DependsOn>, right: Option<DependsOn>) {
  if let Some(right) = right {
    if let Some(left_data) = left {
      if let DependsOn::Simple(left_list) = left_data && let DependsOn::Simple(right_list) = right {
          left_list.extend(right_list);
        } else if let DependsOn::Conditional(left_list) = left_data && let DependsOn::Conditional(right_list) = right {
          left_list.extend(right_list);
        } else {
          *left = Some(right);
      }
    } else {
      *left = Some(right);
    }
  }
}

impl Default for DependsOn {
  fn default() -> Self {
    Self::Simple(Default::default())
  }
}

impl DependsOn {
  pub fn is_empty(&self) -> bool {
    match self {
      Self::Simple(v) => v.is_empty(),
      Self::Conditional(m) => m.is_empty(),
    }
  }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DependsOnCondition {
  /// Waits until the service has completed successfully.
  ServiceCompletedSuccessfully,
  /// Waits until the service is healthy (as defined by its healthcheck).
  ServiceHealthy,
  /// Waits until the service has started.
  ServiceStarted,
}

/// With the depends_on attribute, you can control the order of service startup and shutdown. It is useful if services are closely coupled, and the startup sequence impacts the application's functionality.
///
/// See more: https://docs.docker.com/reference/compose-file/services/#depends_on
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema)]
pub struct DependsOnSettings {
  /// Condition to wait for.
  pub condition: DependsOnCondition,

  /// Whether to restart dependent services when this service is restarted.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub restart: Option<bool>,

  /// Whether the dependency is required for the dependent service to start. (default: true)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub required: Option<bool>,
}

/// A logging driver for Docker.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum LoggingDriver {
  /// Logs are stored in a custom format designed for minimal overhead.
  Local,
  /// The logs are formatted as JSON. The default logging driver for Docker.
  JsonFile,
  /// Writes logging messages to the syslog facility. The syslog daemon must be running on the host machine.
  Syslog,
  /// Writes log messages to journald. The journald daemon must be running on the host machine.
  Journald,
  /// Writes log messages to a Graylog Extended Log Format (GELF) endpoint such as Graylog or Logstash.
  Gelf,
  /// Writes log messages to fluentd (forward input). The fluentd daemon must be running on the host machine.
  Fluentd,
  /// Writes log messages to Amazon CloudWatch Logs.
  Awslogs,
  /// Writes log messages to splunk using the HTTP Event Collector.
  Splunk,
  /// Writes log messages as Event Tracing for Windows (ETW) events. Only available on Windows platforms.
  Etwlogs,
  /// Writes log messages to Google Cloud Platform (GCP) Logging.
  Gcplogs,
}

/// Defines the logging configuration.
///
/// See more: https://docs.docker.com/engine/logging/configure/
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct LoggingSettings {
  /// Logging driver to use, such as 'json-file', 'syslog', 'journald', etc.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub driver: Option<LoggingDriver>,

  /// Options for the logging driver.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub options: Option<BTreeMap<String, Option<StringOrNum>>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum PortMode {
  Host,
  Ingress,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
  Tcp,
  Udp,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, Eq, PartialOrd, Ord)]
#[serde(untagged)]
pub enum Port {
  Num(u64),
  String(String),
  Data(PortSettings),
}

/// Settings for a port mapping.
///
/// See more: https://docs.docker.com/reference/compose-file/services/#long-syntax-4
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, Eq, PartialOrd, Ord)]
pub struct PortSettings {
  /// A human-readable name for this port mapping.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,

  /// The host IP to bind to.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub host_ip: Option<String>,

  /// The port inside the container.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub target: Option<StringOrNum>,

  /// The publicly exposed port.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub published: Option<StringOrNum>,

  /// The port binding mode, either 'host' for publishing a host port or 'ingress' for load balancing.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub mode: Option<PortMode>,

  /// The port protocol (tcp or udp).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub protocol: Option<Protocol>,

  /// The application protocol (TCP/IP level 4 / OSI level 7) this port is used for. This is optional and can be used as a hint for Compose to offer richer behavior for protocols that it understands.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub app_protocol: Option<String>,
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

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum BuildStep {
  /// Path to the build context. Can be a relative path or a URL.
  Simple(String),
  /// Configuration options for building the service's image.
  Advanced(AdvancedBuildStep),
}

fn merge_build_step(left: &mut Option<BuildStep>, right: Option<BuildStep>) {
  if let Some(right) = right {
    if let Some(left_data) = left {
      if let BuildStep::Advanced(left_data) = left_data && let BuildStep::Advanced(right_data) = right {
        left_data.merge(right_data);
      } else {
        *left = Some(right);
      }
    } else {
      *left = Some(right);
    }
  }
}

/// Specifies the build configuration for creating a container image from source, as defined in the [Compose Build Specification](https://docs.docker.com/reference/compose-file/build/).
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq, JsonSchema, Default, Merge)]
#[serde(deny_unknown_fields)]
#[serde(default)]
#[merge(strategy = overwrite_if_some)]
pub struct AdvancedBuildStep {
  /// Defines either a path to a directory containing a Dockerfile, or a URL to a Git repository.
  ///
  /// When the value supplied is a relative path, it is interpreted as relative to the project directory. Compose warns you about the absolute path used to define the build context as those prevent the Compose file from being portable.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/build/#context
  #[serde(skip_serializing_if = "Option::is_none")]
  pub context: Option<String>,

  /// Requires: Docker Compose 2.17.0 and later
  ///
  /// Defines a list of named contexts the image builder should use during image build. Can be a mapping or a list.
  /// See more: https://docs.docker.com/reference/compose-file/build/#additional_contexts
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_list_or_map)]
  pub additional_contexts: Option<ListOrMap>,

  /// Sets an alternate Dockerfile. A relative path is resolved from the build context. Compose warns you about the absolute path used to define the Dockerfile as it prevents Compose files from being portable.
  ///
  /// When set, dockerfile_inline attribute is not allowed and Compose rejects any Compose file having both set.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub dockerfile: Option<String>,

  /// Requires: Docker Compose 2.17.0 and later
  ///
  /// dockerfile_inline defines the Dockerfile content as an inlined string in a Compose file. When set, the dockerfile attribute is not allowed and Compose rejects any Compose file having both set.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub dockerfile_inline: Option<String>,

  /// Define build arguments, that is Dockerfile ARG values.
  ///
  /// Cache location syntax follows the global format [NAME|type=TYPE[,KEY=VALUE]]. Simple NAME is actually a shortcut notation for type=registry,ref=NAME.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/build/#args
  #[serde(skip_serializing_if = "Option::is_none")]
  pub args: Option<ListOrMap>,

  /// Defines a list of sources the image builder should use for cache resolution.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/build/#cache_from
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_vecs)]
  pub cache_from: Option<Vec<String>>,

  /// Defines a list of export locations to be used to share build cache with future builds.
  ///
  /// Cache location syntax follows the global format [NAME|type=TYPE[,KEY=VALUE]]. Simple NAME is actually a shortcut notation for type=registry,ref=NAME.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/build/#cache_to
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_vecs)]
  pub cache_to: Option<Vec<String>>,

  /// Requires: Docker Compose 2.27.1 and later
  ///
  /// Defines extra privileged entitlements to be allowed during the build.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_btree_sets)]
  pub entitlements: Option<BTreeSet<String>>,

  /// Adds hostname mappings at build-time. Use the same syntax as [extra_hosts](https://docs.docker.com/reference/compose-file/services/#extra_hosts).
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_extra_hosts)]
  pub extra_hosts: Option<ExtraHosts>,

  /// Specifies a build’s container isolation technology. Supported values are platform specific.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub isolation: Option<String>,

  /// Add metadata to the resulting image. Can be set either as an array or a map.
  ///
  /// It's recommended that you use reverse-DNS notation to prevent your labels from conflicting with other software.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_list_or_map)]
  pub labels: Option<ListOrMap>,

  /// Network mode to use for the build. Options include 'default', 'none', 'host', or a network name.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/build/#network
  #[serde(skip_serializing_if = "Option::is_none")]
  pub network: Option<String>,

  /// Disables image builder cache and enforces a full rebuild from source for all image layers. This only applies to layers declared in the Dockerfile, referenced images can be retrieved from local image store whenever tag has been updated on registry (see [pull](https://docs.docker.com/reference/compose-file/build/#pull)).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub no_cache: Option<bool>,

  /// Defines a list of target [platforms](https://docs.docker.com/reference/compose-file/services/#platform).
  ///
  /// See more: https://docs.docker.com/reference/compose-file/build/#platforms
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_btree_sets)]
  pub platforms: Option<BTreeSet<String>>,

  /// Requires: Docker Compose 2.15.0 and later
  ///
  /// Configures the service image to build with elevated privileges. Support and actual impacts are platform specific.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub privileged: Option<bool>,

  /// Requires: Docker Compose 2.39.0 and later
  ///
  /// Configures the builder to add a provenance attestation to the published image.
  ///
  /// The value can be either a boolean to enable/disable provenance attestation, or a key=value string to set provenance configuration. You can use this to select the level of detail to be included in the provenance attestation by setting the mode parameter.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub provenance: Option<bool>,

  /// Requires the image builder to pull referenced images (FROM Dockerfile directive), even if those are already available in the local image store.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub pull: Option<bool>,

  /// Requires: Docker Compose 2.39.0 and later
  ///
  /// Configures the builder to add a provenance attestation to the published image.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/build/#sbom
  #[serde(skip_serializing_if = "Option::is_none")]
  pub sbom: Option<bool>,

  /// Grants access to sensitive data defined by secrets on a per-service build basis.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/build/#secrets
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_btree_sets)]
  pub secrets: Option<BTreeSet<ServiceConfigOrSecret>>,

  /// SSH agent socket or keys to expose to the build. Format is either a string or a list of 'default|<id>[=<socket>|<key>[,<key>]]'.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/build/#ssh
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_list_or_map)]
  pub ssh: Option<ListOrMap>,

  /// Size of /dev/shm for the build container. A string value can use suffix like '2g' for 2 gigabytes.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/build/#shm_size
  #[serde(skip_serializing_if = "Option::is_none")]
  pub shm_size: Option<StringOrNum>,

  /// Defines a list of tag mappings that must be associated to the build image. This list comes in addition to the image property defined in the service section.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_btree_sets)]
  pub tags: Option<BTreeSet<String>>,

  /// Defines the stage to build as defined inside a multi-stage Dockerfile.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub target: Option<String>,

  /// Requires: Docker Compose 2.23.1 and later
  ///
  /// ulimits overrides the default ulimits for a container. It's specified either as an integer for a single limit or as mapping for soft/hard limits.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/build/#ulimits
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_btree_maps)]
  pub ulimits: Option<BTreeMap<String, Ulimit>>,
}

/// Configures the UTS namespace mode set for the service container. When unspecified it is the runtime's decision to assign a UTS namespace, if supported.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, JsonSchema, Eq)]
#[serde(rename = "kebab-case")]
pub enum Uts {
  /// Results in the container using the same UTS namespace as the host.
  Host,
}

/// Network configuration for the Compose application.
///
/// See more: https://docs.docker.com/reference/compose-file/networks/
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum TopLevelNetwork {
  External {
    /// If set to true, it specifies that this network’s lifecycle is maintained outside of that of the application. Compose doesn't attempt to create these networks, and returns an error if one doesn't exist.
    ///
    /// See more: https://docs.docker.com/reference/compose-file/networks/#external
    external: bool,
  },
  Config {
    /// Custom name for this network.
    ///
    /// See more: https://docs.docker.com/reference/compose-file/networks/#name
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,

    /// By default, Compose provides external connectivity to networks. internal, when set to true, lets you create an externally isolated network.
    #[serde(skip_serializing_if = "Option::is_none")]
    internal: Option<bool>,

    /// Specifies which driver should be used for this network. Compose returns an error if the driver is not available on the platform.
    ///
    /// For more information on drivers and available options, see [Network drivers](https://docs.docker.com/engine/network/drivers/).
    #[serde(skip_serializing_if = "Option::is_none")]
    driver: Option<String>,

    /// If `attachable` is set to `true`, then standalone containers should be able to attach to this network, in addition to services. If a standalone container attaches to the network, it can communicate with services and other standalone containers that are also attached to the network.
    #[serde(skip_serializing_if = "Option::is_none")]
    attachable: Option<bool>,

    /// Can be used to disable IPv4 address assignment.
    #[serde(skip_serializing_if = "Option::is_none")]
    enable_ipv4: Option<bool>,

    /// Enables IPv6 address assignment.
    #[serde(skip_serializing_if = "Option::is_none")]
    enable_ipv6: Option<bool>,

    /// Specifies a custom IPAM configuration.
    ///
    /// See more: https://docs.docker.com/reference/compose-file/networks/#ipam
    #[serde(skip_serializing_if = "Option::is_none")]
    ipam: Option<Ipam>,

    /// A list of options as key-value pairs to pass to the driver. These options are driver-dependent.
    ///
    /// Consult the [network drivers documentation](https://docs.docker.com/engine/network/) for more information.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    driver_opts: BTreeMap<String, Option<SingleValue>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    labels: Option<ListOrMap>,
  },
}

/// Specifies a custom IPAM configuration.
///
/// See more: https://docs.docker.com/reference/compose-file/networks/#ipam
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Ipam {
  /// Custom IPAM driver, instead of the default.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub driver: Option<String>,

  /// A list with zero or more configuration elements.
  #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
  pub config: BTreeSet<IpamConfig>,

  /// Driver-specific options as a key-value mapping.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub options: Option<StringBTreeMap>,
}

/// IPAM specific configurations.
///
/// See more: https://docs.docker.com/reference/compose-file/networks/#ipam
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema)]
#[serde(deny_unknown_fields)]
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
    Some(self.cmp(&other))
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

/// Declares a check that's run to determine whether or not the service containers are "healthy".
///
/// See more: https://docs.docker.com/reference/compose-file/services/#healthcheck
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct Healthcheck {
  /// Disable any container-specified healthcheck. Set to true to disable.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub disable: Option<bool>,

  /// The test to perform to check container health. Can be a string or a list. The first item is either NONE, CMD, or CMD-SHELL. If it's CMD, the rest of the command is exec'd. If it's CMD-SHELL, the rest is run in the shell.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub test: Option<StringOrList>,

  /// Time between running the check (e.g., '1s', '1m30s'). Default: 30s.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub interval: Option<String>,

  /// Start period for the container to initialize before starting health-retries countdown (e.g., '1s', '1m30s'). Default: 0s.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub start_period: Option<String>,

  /// Time between running the check during the start period (e.g., '1s', '1m30s'). Default: interval value.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub start_interval: Option<String>,

  /// Number of consecutive failures needed to consider the container as unhealthy. Default: 3.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub retries: Option<StringOrNum>,

  /// Maximum time to allow one check to run (e.g., '1s', '1m30s'). Default: 30s.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub timeout: Option<String>,
}

/// Resource constraints and reservations for the service.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct GenericResource {
  /// Specification for discrete (countable) resources.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub discrete_resource_spec: Option<DiscreteResourceSpec>,
}

/// Specification for discrete (countable) resources.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema, Default)]
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
    Some(self.cmp(&other))
  }
}

impl Ord for DiscreteResourceSpec {
  fn cmp(&self, other: &Self) -> Ordering {
    self.kind.cmp(&other.kind)
  }
}

/// Device reservations for containers, allowing services to access specific hardware devices.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct Device {
  /// Device driver to use (e.g., 'nvidia').
  #[serde(skip_serializing_if = "Option::is_none")]
  pub driver: Option<String>,

  /// Number of devices of this type to reserve.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub count: Option<StringOrNum>,

  /// List of specific device IDs to reserve.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub device_ids: Option<BTreeSet<String>>,

  /// List of capabilities the device needs to have (e.g., 'gpu', 'compute', 'utility').
  #[serde(skip_serializing_if = "BTreeSet::is_empty")]
  pub capabilities: BTreeSet<String>,

  /// Driver-specific options for the device.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub options: Option<ListOrMap>,
}

impl PartialOrd for Device {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(&other))
  }
}

impl Ord for Device {
  fn cmp(&self, other: &Self) -> Ordering {
    self.driver.cmp(&other.driver)
  }
}

/// Specifies constraints and preferences for the platform to select a physical node to run service containers. See more: https://docs.docker.com/reference/compose-file/deploy/#placement
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema, Default)]
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
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
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
    Some(self.cmp(&other))
  }
}

impl Ord for ServiceConfigOrSecretSettings {
  fn cmp(&self, other: &Self) -> Ordering {
    self.source.cmp(&other.source)
  }
}

/// Configuration for service configs or secrets, defining how they are mounted in the container.
#[derive(Clone, Default, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ServiceConfigOrSecretSettings {
  /// Name of the config or secret as defined in the top-level configs or secrets section.
  pub source: String,

  /// Path in the container where the config or secret will be mounted. Defaults to /<source> for configs and /run/secrets/<source> for secrets.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub target: Option<String>,

  /// UID of the file in the container. Default is 0 (root).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub uid: Option<String>,

  /// GID of the file in the container. Default is 0 (root).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub gid: Option<String>,

  /// File permission mode inside the container, in octal. Default is 0444 for configs and 0400 for secrets.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub mode: Option<StringOrNum>,
}

/// Defines the decisions Compose makes when it starts to pull images.
///
/// See more: https://docs.docker.com/reference/compose-file/services/#pull_policy
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum PullPolicy {
  /// Compose always pulls the image from the registry.
  Always,
  /// Compose builds the image. Compose rebuilds the image if it's already present.
  Build,
  /// Compose checks the registry for image updates if the last pull took place more than 24 hours ago.
  Daily,
  /// Compose pulls the image only if it's not available in the platform cache. This is the default option if you are not also using the [Compose Build Specification](https://docs.docker.com/reference/compose-file/build/). if_not_present is considered an alias for this value for backward compatibility. The latest tag is always pulled even when the missing pull policy is used.
  #[serde(alias = "if_not_present")]
  Missing,
  /// Compose doesn't pull the image from a registry and relies on the platform cached image. If there is no cached image, a failure is reported.
  Never,
  Refresh,
  /// Compose checks the registry for image updates if the last pull took place more than 7 days ago.
  Weekly,
  #[serde(untagged)]
  Other(String),
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum ServiceVolume {
  Simple(String),
  Advanced(ServiceVolumeSettings),
}

impl PartialOrd for ServiceVolume {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(&other))
  }
}

impl Ord for ServiceVolume {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    match self {
      ServiceVolume::Simple(s) => match other {
        ServiceVolume::Simple(other_s) => s.cmp(&other_s),
        ServiceVolume::Advanced(_) => Ordering::Greater,
      },
      ServiceVolume::Advanced(v) => match other {
        ServiceVolume::Simple(_) => Ordering::Less,
        ServiceVolume::Advanced(other_v) => v.type_.cmp(&other_v.type_),
      },
    }
  }
}

/// The mount type.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum VolumeType {
  /// For mounting host directories.
  Bind,

  /// For cluster volumes.
  Cluster,

  /// For named pipes.
  Npipe,

  /// For mounting from an image.
  Image,

  /// For temporary filesystems.
  Tmpfs,

  /// For names volumes.
  Volume,
}

/// Configuration for a service volume.
///
/// See more: https://docs.docker.com/reference/compose-file/services/#long-syntax-6
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ServiceVolumeSettings {
  /// The mount type.
  #[serde(rename = "type")]
  pub type_: VolumeType,

  /// Flag to set the volume as read-only.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub read_only: Option<bool>,

  /// The source of the mount, a path on the host for a bind mount, a docker image reference for an image mount, or the name of a volume defined in the top-level volumes key. Not applicable for a tmpfs mount.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub source: Option<String>,

  /// The path in the container where the volume is mounted.
  pub target: String,

  /// The consistency requirements for the mount. Available values are platform specific.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub consistency: Option<String>,

  /// Configuration specific to bind mounts.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub bind: Option<Bind>,

  /// Configuration specific to image mounts.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub image: Option<ImageVolumeSettings>,

  /// /// Configuration specific to tmpfs mounts.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tmpfs: Option<TmpfsSettings>,

  /// Configuration specific to volume mounts.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub volume: Option<VolumeSettings>,
}

/// The propagation mode for the bind mount
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Propagation {
  Private,
  Rprivate,
  Rshared,
  Rslave,
  Shared,
  Slave,
}

/// Recursively mount the source directory.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Recursive {
  Disabled,
  Enabled,
  Readonly,
  Writable,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema)]
pub enum SELinux {
  /// For shared content.
  #[serde(rename = "z")]
  Shared,

  /// For private, unshared content.
  #[serde(rename = "Z")]
  Unshared,
}

/// Configuration specific to bind mounts.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Bind {
  /// Create the host path if it doesn't exist.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub create_host_path: Option<bool>,

  /// The propagation mode for the bind mount:
  #[serde(skip_serializing_if = "Option::is_none")]
  pub propagation: Option<Propagation>,

  /// Recursively mount the source directory.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub recursive: Option<Recursive>,

  /// SELinux relabeling options: 'z' for shared content, 'Z' for private unshared content.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub selinux: Option<SELinux>,
}

/// Configuration specific to volume mounts.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VolumeSettings {
  /// Labels to apply to the volume.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub labels: Option<ListOrMap>,

  /// Flag to disable copying of data from a container when a volume is created.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub nocopy: Option<bool>,

  /// Path within the volume to mount instead of the volume root.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub subpath: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImageVolumeSettings {
  /// Path within the image to mount instead of the image root.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub subpath: Option<String>,
}

/// Configuration specific to tmpfs mounts.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TmpfsSettings {
  /// File mode of the tmpfs in octal.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub mode: Option<StringOrNum>,

  /// Size of the tmpfs mount in bytes.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub size: Option<StringOrNum>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd, JsonSchema)]
#[serde(untagged)]
pub enum SingleValue {
  String(String),
  Bool(bool),
  Int(i64),
  Float(f64),
}

impl fmt::Display for SingleValue {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Self::String(s) => f.write_str(s),
      Self::Bool(b) => write!(f, "{b}"),
      Self::Int(i) => write!(f, "{i}"),
      Self::Float(fl) => write!(f, "{fl}"),
    }
  }
}

/// Defines a set of configuration options to set block I/O limits for a service.
///
/// See more: https://docs.docker.com/reference/compose-file/services/#blkio_config
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default, JsonSchema)]
pub struct BlkioSettings {
  /// Limit read rate (bytes per second) from a device.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub device_read_bps: Option<BTreeSet<BlkioLimit>>,

  /// Limit read rate (IO per second) from a device.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub device_read_iops: Option<BTreeSet<BlkioLimit>>,

  /// Limit write rate (bytes per second) to a device.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub device_write_bps: Option<BTreeSet<BlkioLimit>>,

  /// Limit write rate (IO per second) to a device.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub device_write_iops: Option<BTreeSet<BlkioLimit>>,

  /// Block IO weight (relative weight) for the service, between 10 and 1000.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub weight: Option<StringOrNum>,

  /// Block IO weight (relative weight) for specific devices.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub weight_device: Option<BTreeSet<BlkioWeight>>,
}

/// Block IO limit for a specific device.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default, Eq, JsonSchema)]
pub struct BlkioLimit {
  /// Path to the device (e.g., '/dev/sda').
  #[serde(skip_serializing_if = "Option::is_none")]
  pub path: Option<String>,

  /// Rate limit in bytes per second or IO operations per second.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub rate: Option<StringOrNum>,
}

impl PartialOrd for BlkioLimit {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(&other))
  }
}

impl Ord for BlkioLimit {
  fn cmp(&self, other: &Self) -> Ordering {
    self
      .path
      .cmp(&other.path)
      .then_with(|| self.rate.cmp(&other.rate))
  }
}

/// Block IO weight for a specific device.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default, Eq, JsonSchema)]
pub struct BlkioWeight {
  /// Path to the device (e.g., '/dev/sda').
  #[serde(skip_serializing_if = "Option::is_none")]
  pub path: Option<String>,

  /// Relative weight for the device, between 10 and 1000.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub weight: Option<StringOrNum>,
}

impl PartialOrd for BlkioWeight {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(&other))
  }
}

impl Ord for BlkioWeight {
  fn cmp(&self, other: &Self) -> Ordering {
    self
      .path
      .cmp(&other.path)
      .then_with(|| self.weight.cmp(&other.weight))
  }
}

/// Specify the cgroup namespace to join. Use 'host' to use the host's cgroup namespace, or 'private' to use a private cgroup namespace.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Cgroup {
  /// Use the host's cgroup namespace.
  Host,

  /// Use a private cgroup namespace.
  Private,
}

/// Defines or references configuration data that is granted to services in your Compose application.
///
/// See more: https://docs.docker.com/reference/compose-file/configs/
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default, Eq, JsonSchema)]
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

/// Configures the credential spec for a managed service account.
///
/// See more: https://docs.docker.com/reference/compose-file/services/#credential_spec
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default, Eq, JsonSchema)]
pub struct CredentialSpec {
  /// The name of the credential spec Config to use.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub config: Option<String>,

  /// Path to a credential spec file.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub file: Option<String>,

  /// Path to a credential spec in the Windows registry.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub registry: Option<String>,
}

/// Specifies the development configuration for maintaining a container in sync with source.
///
/// See more: https://docs.docker.com/reference/compose-file/develop
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default, Eq, JsonSchema)]
pub struct DevelopmentSettings {
  /// The watch attribute defines a list of rules that control automatic service updates based on local file changes. watch is a sequence, each individual item in the sequence defines a rule to be applied by Compose to monitor source code for changes.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub watch: Option<BTreeSet<WatchItem>>,
}

fn merge_development_settings(
  left: &mut Option<DevelopmentSettings>,
  right: Option<DevelopmentSettings>,
) {
  if let Some(right) = right && let Some(watch_right) = right.watch {
    let left_data = left.get_or_insert_default();
    let watch_left = left_data.watch.get_or_insert_default();
    watch_left.extend(watch_right);
  }
}

/// An element of the watch mode configuration.
///
/// See more: https://docs.docker.com/reference/compose-file/develop/#watch
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct WatchItem {
  /// Action to take when a change is detected.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/develop/#action
  pub action: WatchAction,

  /// Requires: Docker Compose 2.32.2 and later
  ///
  /// Only relevant when action is set to sync+exec. Like service hooks, exec is used to define the command to be run inside the container once it has started.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/develop/#exec
  #[serde(skip_serializing_if = "Option::is_none")]
  pub exec: Option<ServiceHook>,

  /// Patterns to exclude from watching.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/develop/#ignore
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ignore: Option<BTreeSet<String>>,

  /// It is sometimes easier to select files to be watched instead of declaring those that shouldn't be watched with ignore.
  ///
  /// See more: https://docs.docker.com/reference/compose-file/develop/#include
  #[serde(skip_serializing_if = "Option::is_none")]
  pub include: Option<BTreeSet<String>>,

  /// Defines the path to source code (relative to the project directory) to monitor for changes. Updates to any file inside the path, which doesn't match any ignore rule, triggers the configured action.
  pub path: String,

  /// Only applies when action is configured for sync. Files within path that have changes are synchronized with the container's filesystem, so that the latter is always running with up-to-date content.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub target: Option<String>,
}

impl PartialOrd for WatchItem {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(&other))
  }
}

impl Ord for WatchItem {
  fn cmp(&self, other: &Self) -> Ordering {
    self.action.cmp(&other.action)
  }
}

/// Action to take when a change is detected.
///
/// See more: https://docs.docker.com/reference/compose-file/develop/#action
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum WatchAction {
  Rebuild,
  Restart,
  Sync,
  #[serde(rename = "sync+restart")]
  SyncRestart,
  #[serde(rename = "sync+exec")]
  SyncExec,
}

/// Configuration for service lifecycle hooks, which are commands executed at specific points in a container's lifecycle.
///
/// See more: https://docs.docker.com/compose/how-tos/lifecycle/
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct ServiceHook {
  /// Whether to run the command with extended privileges.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub privileged: Option<bool>,

  /// User to run the command as.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub user: Option<String>,

  /// Working directory for the command.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub working_dir: Option<String>,

  /// Environment variables for the command.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub environment: Option<ListOrMap>,

  /// Command to execute as part of the hook.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub command: Option<StringOrList>,
}

/// A device mapping for a container.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord, JsonSchema)]
#[serde(untagged)]
pub enum DeviceMapping {
  String(String),
  Detailed(DeviceMappingSettings),
}

/// A device mapping for a container.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct DeviceMappingSettings {
  /// Path on the host to the device.
  pub source: String,

  /// Path in the container where the device will be mapped.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub target: Option<String>,

  /// Cgroup permissions for the device (rwm).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub permissions: Option<String>,
}

impl PartialOrd for DeviceMappingSettings {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(&other))
  }
}

impl Ord for DeviceMappingSettings {
  fn cmp(&self, other: &Self) -> Ordering {
    self.source.cmp(&other.source)
  }
}

/// Defines an environment file(s) to use to define default values when interpolating variables in the Compose file being parsed. It defaults to .env file in the project_directory for the Compose file being parsed.
///
/// See more: https://docs.docker.com/reference/compose-file/include/#env_file
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord, JsonSchema)]
#[serde(untagged)]
pub enum Envfile {
  /// Path to a file containing environment variables.
  Simple(String),
  List(Vec<EnvfileFormat>),
}

fn merge_envfile(left: &mut Option<Envfile>, right: Option<Envfile>) {
  if let Some(right) = right {
    if let Some(left_data) = left {
      match left_data {
        Envfile::Simple(_) => {
          *left = Some(right);
        }
        Envfile::List(list) => match right {
          Envfile::Simple(file) => list.push(EnvfileFormat::Simple(file)),
          Envfile::List(files) => list.extend(files),
        },
      }
    } else {
      *left = Some(right);
    }
  }
}

/// Defines an environment file(s) to use to define default values when interpolating variables in the Compose file being parsed. It defaults to .env file in the project_directory for the Compose file being parsed.
///
/// See more: https://docs.docker.com/reference/compose-file/include/#env_file
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord, JsonSchema)]
#[serde(untagged)]
pub enum EnvfileFormat {
  /// Path to a file containing environment variables.
  Simple(String),
  /// Detailed configuration for an environment file.
  Detailed(EnvFileDetailed),
}

/// Detailed configuration for an environment file.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord, JsonSchema)]
pub struct EnvFileDetailed {
  /// Path to the environment file.
  pub path: String,

  /// Format attribute lets you to use an alternative file formats for env_file. When not set, env_file is parsed according to Compose rules.
  pub format: Option<String>,

  /// Whether the file is required. If true and the file doesn't exist, an error will be raised. (default: true)
  pub required: Option<bool>,
}

/// `extends` lets you share common configurations among different files, or even different projects entirely.
///
/// See more: https://docs.docker.com/reference/compose-file/services/#extends
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord, JsonSchema)]
#[serde(untagged)]
pub enum Extends {
  /// The name of the service to extend.
  Simple(String),
  Detailed {
    /// The name of the service to extend.
    service: String,

    /// The file path where the service to extend is defined.
    file: Option<String>,
  },
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

fn merge_extra_hosts(left: &mut Option<ExtraHosts>, right: Option<ExtraHosts>) {
  if let Some(right) = right {
    if let Some(left_data) = left {
      if let ExtraHosts::List(left_list) = left_data && let ExtraHosts::List(right_list) = right {
        left_list.extend(right_list);
      } else if let ExtraHosts::Map(left_map) = left_data && let ExtraHosts::Map(right_map) = right {
        left_map.extend(right_map);
      } else {
        *left = Some(right);
      }
    } else {
      *left = Some(right);
    }
  }
}

/// Requires: Docker Compose 2.30.0 and later
///
/// Specifies GPU devices to be allocated for container usage.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord, JsonSchema)]
pub enum Gpus {
  /// Use all available GPUs.
  #[serde(rename = "all")]
  All,
  /// List of specific GPU devices to use.
  #[serde(untagged)]
  List(BTreeSet<GpuSettings>),
}

fn merge_gpus(left: &mut Option<Gpus>, right: Option<Gpus>) {
  if let Some(right) = right {
    if let Some(left_data) = left {
      match left_data {
        Gpus::All => {}
        Gpus::List(left_gpus) => match right {
          Gpus::All => *left_data = Gpus::All,
          Gpus::List(right_gpus) => left_gpus.extend(right_gpus),
        },
      }
    } else {
      *left = Some(right);
    }
  }
}

/// Requires: Docker Compose 2.30.0 and later
///
/// Specifies GPU devices to be allocated for container usage.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct GpuSettings {
  /// List of capabilities the GPU needs to have (e.g., 'compute', 'utility').
  #[serde(skip_serializing_if = "Option::is_none")]
  pub capabilities: Option<BTreeSet<String>>,

  // Number of GPUs to use.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub count: Option<usize>,

  /// List of specific GPU device IDs to use.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub device_ids: Option<BTreeSet<String>>,

  /// GPU driver to use (e.g., 'nvidia').
  #[serde(skip_serializing_if = "Option::is_none")]
  pub driver: Option<String>,

  /// Driver-specific options for the GPU.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub options: Option<ListOrMap>,
}

impl PartialOrd for GpuSettings {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(&other))
  }
}

impl Ord for GpuSettings {
  fn cmp(&self, other: &Self) -> Ordering {
    self
      .driver
      .cmp(&other.driver)
      .then_with(|| self.count.cmp(&other.count))
  }
}

/// Requires: Docker Compose 2.38.0 and later
///
/// Defines which AI models the service should use at runtime. Each referenced model must be defined under the [`models` top-level element](https://docs.docker.com/reference/compose-file/models/).
///
/// See more: https://docs.docker.com/reference/compose-file/services/#models
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord, JsonSchema)]
#[serde(untagged)]
pub enum ServiceModels {
  List(BTreeSet<String>),
  Map(BTreeMap<String, ServiceModelSettings>),
}

fn merge_service_models(left: &mut Option<ServiceModels>, right: Option<ServiceModels>) {
  if let Some(right) = right {
    if let Some(left_data) = left {
      if let ServiceModels::List(left_list) = left_data && let ServiceModels::List(right_list) = right {
        left_list.extend(right_list);
      } else if let ServiceModels::Map(left_list) = left_data && let ServiceModels::Map(right_list) = right {
        left_list.extend(right_list);
      } else {
        *left = Some(right);
      }
    } else {
      *left = Some(right);
    }
  }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema)]
pub struct ServiceModelSettings {
  /// Environment variable set to AI model name.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub model_var: Option<String>,

  /// Environment variable set to AI model endpoint.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub endpoint_var: Option<String>,
}

impl PartialOrd for ServiceModelSettings {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(&other))
  }
}

impl Ord for ServiceModelSettings {
  fn cmp(&self, other: &Self) -> Ordering {
    self.model_var.cmp(&other.model_var)
  }
}

/// Sets a service container's network mode.
///
///
/// See more: https://docs.docker.com/reference/compose-file/services/#network_mode
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord, JsonSchema)]
pub enum NetworkMode {
  Bridge,
  /// Gives the container raw access to the host's network interface.
  Host,
  /// Turns off all container networking.
  None,
  #[serde(untagged)]
  Other(String),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum ServiceNetworks {
  List(BTreeSet<String>),
  Map(BTreeMap<String, ServiceNetworkSettings>),
}

fn merge_service_networks(left: &mut Option<ServiceNetworks>, right: Option<ServiceNetworks>) {
  if let Some(right) = right {
    if let Some(left_data) = left {
      if let ServiceNetworks::List(left_list) = left_data && let ServiceNetworks::List(right_list) = right {
        left_list.extend(right_list);
      } else if let ServiceNetworks::Map(left_list) = left_data && let ServiceNetworks::Map(right_list) = right {
        left_list.extend(right_list);
      } else {
        *left = Some(right);
      }
    } else {
      *left = Some(right);
    }
  }
}

/// The networks attribute defines the networks that service containers are attached to, referencing entries under the networks top-level element.
///
/// See more: https://docs.docker.com/reference/compose-file/services/#networks
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct ServiceNetworkSettings {
  /// Interface network name used to connect to network
  #[serde(skip_serializing_if = "Option::is_none")]
  pub interface_name: Option<String>,

  /// Alternative hostnames for this service on the network.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub aliases: Option<BTreeSet<String>>,

  /// Specify a static IPv4 address for this service on this network.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ipv4_address: Option<String>,

  /// Specify a static IPv6 address for this service on this network.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ipv6_address: Option<String>,

  /// Specify a MAC address for this service on this network.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub mac_address: Option<String>,

  /// Specify the priority for the network connection.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub priority: Option<i64>,

  /// Specify the gateway priority for the network connection.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub gw_priority: Option<i64>,

  /// List of link-local IPs.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub link_local_ips: Option<BTreeSet<String>>,

  /// Driver options for this network.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub driver_opts: Option<BTreeMap<String, SingleValue>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum ProviderOptions {
  Single(SingleValue),
  List(Vec<SingleValue>),
}

/// Defines a service that Compose won't manage directly. Compose delegated the service lifecycle to a dedicated or third-party component.
///
/// See more: https://docs.docker.com/reference/compose-file/services/#provider
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct Provider {
  /// External component used by Compose to manage setup and teardown lifecycle of the service.
  #[serde(rename = "type")]
  pub type_: String,

  /// Provider-specific options.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub options: Option<BTreeMap<String, ProviderOptions>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Restart {
  Always,
  No,
  OnFailure,
  UnlessStopped,
  #[serde(untagged)]
  Other(String),
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
  pub name: Option<String>,

  /// The context window size for the model.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub context_size: Option<u64>,

  /// Raw runtime flags to pass to the inference engine.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub runtime_flags: Option<Vec<String>>,
}

impl PartialOrd for TopLevelModel {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(&other))
  }
}

impl Ord for TopLevelModel {
  fn cmp(&self, other: &Self) -> Ordering {
    self
      .name
      .cmp(&other.name)
      .then_with(|| self.model.cmp(&other.model))
  }
}

/// Volume configuration for the Compose application.
///
/// See more: https://docs.docker.com/reference/compose-file/volumes/
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum TopLevelVolume {
  External {
    /// If set to true, it specifies that this volume already exists on the platform and its lifecycle is managed outside of that of the application.
    ///
    /// See more: https://docs.docker.com/reference/compose-file/volumes/#external
    external: bool,
  },
  Config {
    /// Sets a custom name for a volume.
    ///
    /// See more: https://docs.docker.com/reference/compose-file/volumes/#name
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,

    /// Specifies which volume driver should be used. If the driver is not available, Compose returns an error and doesn't deploy the application.
    #[serde(skip_serializing_if = "Option::is_none")]
    driver: Option<String>,

    /// Specifies a list of options as key-value pairs to pass to the driver for this volume. The options are driver-dependent.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    driver_opts: BTreeMap<String, SingleValue>,

    /// Labels are used to add metadata to volumes. You can use either an array or a dictionary.
    ///
    /// It's recommended that you use reverse-DNS notation to prevent your labels from conflicting with those used by other software.
    ///
    /// See more: https://docs.docker.com/reference/compose-file/volumes/#labels
    #[serde(skip_serializing_if = "Option::is_none")]
    labels: Option<ListOrMap>,
  },
}
