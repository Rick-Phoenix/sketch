use std::{
  collections::{BTreeMap, BTreeSet},
  fmt::Display,
  hash::Hash,
};

use askama::Template;
use futures::future;
use indexmap::{IndexMap, IndexSet};
use merge::Merge;
use serde::{Deserialize, Serialize};

use crate::{
  rendering::render_json_val, versions::get_latest_version, GenError, JsonValueBTreeMap, Preset,
  StringBTreeMap, VersionRange,
};

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum PackageJsonKind {
  Id(String),
  Config(Box<PackageJson>),
}

impl Default for PackageJson {
  fn default() -> Self {
    Self {
      package_name: "my-awesome-package".to_string(),
      extends: Default::default(),
      use_default_deps: true,
      dependencies: Default::default(),
      dev_dependencies: Default::default(),
      scripts: Default::default(),
      metadata: Default::default(),
      repository: None,
      description: Some("A package that solves all of your problems...".to_string()),
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
    }
  }
}

/// An enum representing valid formats for the `repository` field in a package.json file.
#[derive(Debug, Serialize, Deserialize, Template, Clone)]
#[template(path = "repository.j2")]
#[serde(untagged)]
pub enum Repository {
  Path(String),
  Data {
    #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
    type_: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    directory: Option<String>,
  },
}

/// A struct representing the `bugs` field in a package.json file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bugs {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub url: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub email: Option<String>,
}

/// The possible values used for representing an individual in a package.json file, which can be used to populate the `contributors` and `maintainers` fields.
/// If a plain string is used, it will be interpreted as an id for a [`Person`] that is stored in the global config.
#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, Ord, Eq)]
#[serde(untagged)]
#[derive(Clone)]
pub enum Person {
  Workspace(String),
  Data(PersonData),
}

/// A struct that matches how an individual's information is represented in a package.json file in the author, maintainers and contributors fields.
#[derive(
  Clone, Debug, Serialize, Deserialize, Default, Template, Ord, PartialEq, PartialOrd, Eq,
)]
#[template(path = "person.j2")]
pub struct PersonData {
  pub name: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub url: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub email: Option<String>,
}

/// A struct matching a value in the `exports` object inside a package.json file.
#[derive(Clone, Debug, Serialize, Deserialize, Template)]
#[template(path = "export_path.j2")]
#[serde(untagged)]
pub enum Exports {
  Path(String),
  Data {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    require: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    import: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    node: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    default: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    types: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[serde(flatten)]
    other: StringBTreeMap,
  },
}

/// A struct that matches the value of the `directories` field in a package.json file.
#[derive(Debug, Serialize, Deserialize)]
pub struct Directories {
  pub bin: Option<String>,
  pub doc: Option<String>,
  pub example: Option<String>,
  pub lib: Option<String>,
  pub man: Option<String>,
  pub test: Option<String>,
  pub other: Option<StringBTreeMap>,
}

/// A struct that matches the possible values for the `man` field of a package.json file.
#[derive(Clone, Debug, Serialize, Deserialize, Template)]
#[template(path = "man.j2")]
#[serde(untagged)]
pub enum Man {
  Path(String),
  List(Vec<String>),
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum PublishConfigAccess {
  Public,
  Restricted,
}

impl Display for PublishConfigAccess {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Public => write!(f, "public"),
      Self::Restricted => write!(f, "restricted"),
    }
  }
}

/// A struct that matches the `publishConfig` field in a package.json file.
#[derive(Clone, Debug, Serialize, Deserialize, Template)]
#[template(path = "publish_config.j2")]
pub struct PublishConfig {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub access: Option<PublishConfigAccess>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub tag: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub registry: Option<String>,
  #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
  pub other: StringBTreeMap,
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

/// A struct representing the contents of a package.json file.
#[derive(Debug, Deserialize, Serialize, Template, Merge, Clone)]
#[template(path = "package.json.j2")]
#[serde(default)]
pub struct PackageJson {
  #[merge(skip)]
  pub package_name: String,
  #[merge(strategy = merge_sets)]
  pub extends: BTreeSet<String>,
  #[merge(strategy = merge::bool::overwrite_false)]
  pub use_default_deps: bool,
  #[merge(strategy = merge_btree_maps)]
  pub dependencies: StringBTreeMap,
  #[merge(strategy = merge_btree_maps)]
  pub optional_dependencies: StringBTreeMap,
  #[merge(strategy = merge_btree_maps)]
  pub peer_dependencies: StringBTreeMap,
  #[merge(strategy = merge_btree_maps)]
  pub bundle_dependencies: StringBTreeMap,
  #[merge(strategy = merge_btree_maps)]
  pub dev_dependencies: StringBTreeMap,
  #[merge(strategy = merge_btree_maps)]
  pub scripts: StringBTreeMap,
  #[merge(strategy = overwrite_option)]
  pub repository: Option<Repository>,
  #[merge(strategy = overwrite_option)]
  pub description: Option<String>,
  #[merge(strategy = merge_sets)]
  pub keywords: BTreeSet<String>,
  #[merge(strategy = overwrite_option)]
  pub homepage: Option<String>,
  #[merge(strategy = overwrite_option)]
  pub bugs: Option<Bugs>,
  #[merge(strategy = overwrite_option)]
  pub license: Option<String>,
  #[merge(strategy = overwrite_option)]
  pub author: Option<Person>,
  #[merge(strategy = merge_sets)]
  pub contributors: BTreeSet<Person>,
  #[merge(strategy = merge_sets)]
  pub maintainers: BTreeSet<Person>,
  #[merge(strategy = merge_sets)]
  pub files: BTreeSet<String>,
  #[merge(strategy = merge_btree_maps)]
  pub exports: BTreeMap<String, Exports>,
  #[merge(strategy = overwrite_option)]
  pub man: Option<Man>,
  #[merge(strategy = overwrite_option)]
  pub config: Option<JsonValueBTreeMap>,
  #[merge(strategy = overwrite_option)]
  pub package_manager: Option<String>,
  #[merge(strategy = overwrite_option)]
  pub publish_config: Option<PublishConfig>,
  #[merge(strategy = merge_btree_maps)]
  pub engines: StringBTreeMap,
  #[merge(strategy = merge_sets)]
  pub os: BTreeSet<String>,
  #[merge(strategy = merge_sets)]
  pub cpu: BTreeSet<String>,
  #[merge(strategy = overwrite_option)]
  pub main: Option<String>,
  #[merge(strategy = overwrite_option)]
  pub browser: Option<String>,
  #[merge(strategy = overwrite_option)]
  pub workspaces: Option<Vec<String>>,
  #[serde(flatten)]
  #[merge(strategy = merge_btree_maps)]
  pub metadata: JsonValueBTreeMap,
}

#[derive(Debug, Clone, Copy)]
enum DepKind {
  Dependency,
  DevDependency,
  OptionalDependency,
  PeerDependency,
  BundleDependency,
}

impl PackageJson {
  #[allow(clippy::filter_map_bool_then)]
  /// Turns 'latest' into the actual latest version for a package, pinned to the selected version range.
  pub async fn get_latest_version_range(
    &mut self,
    range_kind: VersionRange,
  ) -> Result<(), GenError> {
    #[allow(clippy::type_complexity)]
    let mut handles: Vec<tokio::task::JoinHandle<Result<(DepKind, String, String), GenError>>> =
      Vec::new();

    let mut names_to_update: Vec<(DepKind, String)> = Vec::new();

    macro_rules! get_latest {
      ($list:ident, $kind:ident) => {
        for (name, version) in &self.$list {
          if version == "latest" {
            names_to_update.push((DepKind::$kind, name.clone()));
          }
        }
      };
    }

    get_latest!(dependencies, Dependency);
    get_latest!(dev_dependencies, DevDependency);
    get_latest!(optional_dependencies, OptionalDependency);
    get_latest!(bundle_dependencies, BundleDependency);
    get_latest!(peer_dependencies, PeerDependency);

    for (kind, name) in names_to_update {
      let handle = tokio::spawn(async move {
        let actual_latest =
          get_latest_version(&name)
            .await
            .map_err(|e| GenError::LatestVersionError {
              package: name.clone(),
              source: e,
            })?;

        Ok((kind, name, actual_latest))
      });

      handles.push(handle);
    }

    let results = future::join_all(handles).await;

    for result in results {
      match result {
        Ok(Ok((kind, name, actual_latest))) => {
          let new_version_range = range_kind.create(actual_latest);

          match kind {
            DepKind::Dependency => self.dependencies.insert(name, new_version_range),
            DepKind::DevDependency => self.dev_dependencies.insert(name, new_version_range),
            DepKind::OptionalDependency => {
              self.optional_dependencies.insert(name, new_version_range)
            }
            DepKind::PeerDependency => self.peer_dependencies.insert(name, new_version_range),
            DepKind::BundleDependency => self.bundle_dependencies.insert(name, new_version_range),
          }
        }
        Ok(Err(task_error)) => return Err(task_error),
        Err(join_error) => {
          return Err(GenError::Custom(format!(
            "An async task failed unexpectedly: {}",
            join_error
          )))
        }
      };
    }

    Ok(())
  }

  fn merge_configs_recursive(
    &mut self,
    store: &IndexMap<String, PackageJson>,
    processed_ids: &mut Vec<String>,
  ) -> Result<(), GenError> {
    for id in self.extends.clone() {
      let was_absent = !processed_ids.contains(&id);
      processed_ids.push(id.clone());

      if !was_absent {
        return Err(GenError::CircularDependency(format!(
          "Found circular dependency for package_json '{}'. The full processed chain is: {}",
          id,
          processed_ids.join(" -> ")
        )));
      }

      let mut target = store
        .get(id.as_str())
        .ok_or(GenError::PresetNotFound {
          kind: Preset::PackageJson,
          name: id.to_string(),
        })?
        .clone();

      target.merge_configs_recursive(store, processed_ids)?;

      self.merge(target);
    }

    Ok(())
  }

  pub fn merge_configs(
    mut self,
    initial_id: &str,
    store: &IndexMap<String, PackageJson>,
  ) -> Result<Self, GenError> {
    let mut processed_ids: Vec<String> = Default::default();

    processed_ids.push(initial_id.to_string());

    self.merge_configs_recursive(store, &mut processed_ids)?;

    Ok(self)
  }
}
