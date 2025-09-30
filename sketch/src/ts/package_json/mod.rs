pub mod package_json_elements;

use std::collections::{BTreeMap, BTreeSet};

use futures::future;
use indexmap::{IndexMap, IndexSet};
use merge::Merge;
pub use package_json_elements::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
  merging_strategies::*,
  ts::pnpm::PnpmWorkspace,
  versions::{get_latest_npm_version, VersionRange},
  GenError, JsonValueBTreeMap, Preset, StringBTreeMap,
};

impl Extensible for PackageJsonPreset {
  fn get_extended(&self) -> &IndexSet<String> {
    &self.extends_presets
  }
}

fn get_person_data(id: &str, store: &IndexMap<String, PersonData>) -> Option<PersonData> {
  store.get(id).cloned()
}

impl PackageJsonPreset {
  pub fn process_data(
    self,
    current_id: &str,
    store: &IndexMap<String, PackageJsonPreset>,
    people: &IndexMap<String, PersonData>,
  ) -> Result<PackageJson, GenError> {
    let merged_preset = if self.extends_presets.is_empty() {
      self
    } else {
      let mut processed_ids: IndexSet<String> = IndexSet::new();
      merge_presets(
        Preset::PackageJson,
        current_id,
        self,
        store,
        &mut processed_ids,
      )?
    };

    let mut package_json = merged_preset.config;

    package_json.contributors = package_json
      .contributors
      .into_iter()
      .map(|person| {
        if let Person::Id(ref id) = person && let Some(data) = get_person_data(id, &people) {
          Person::Data(data)
        } else {
          person
        }
      })
      .collect();

    package_json.maintainers = package_json
      .maintainers
      .into_iter()
      .map(|person| {
        if let Person::Id(ref id) = person && let Some(data) = get_person_data(id, &people) {
          Person::Data(data)
        } else {
          person
        }
      })
      .collect();

    if let Some(author) = package_json.author.as_mut() {
      if let Person::Id(id) = author && let Some(data) = get_person_data(id.as_str(), &people) {
        *author = Person::Data(data);
      }
    };

    Ok(package_json)
  }
}

/// A [`PackageJson`] preset.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, JsonSchema, Merge, Default)]
#[serde(default)]
pub struct PackageJsonPreset {
  /// The list of extended presets.
  #[merge(strategy = merge_index_sets)]
  pub extends_presets: IndexSet<String>,
  #[serde(flatten)]
  #[merge(strategy = merge_nested)]
  pub config: PackageJson,
}

/// Ways of indicating [`PackageJson`] data. It can be an id, pointing to a preset, or a literal configuration.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum PackageJsonData {
  Id(String),
  Config(PackageJsonPreset),
}

impl Default for PackageJsonData {
  fn default() -> Self {
    Self::Config(PackageJsonPreset::default().into())
  }
}

impl PackageJsonData {
  pub(crate) fn from_cli(s: &str) -> Result<Self, String> {
    Ok(Self::Id(s.trim().to_string()))
  }
}

/// A struct representing the contents of a `package.json` file.
#[derive(Debug, Deserialize, Serialize, Merge, Clone, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[merge(strategy = overwrite_if_some)]
#[serde(default)]
pub struct PackageJson {
  /// The name of the package.
  pub name: Option<String>,

  /// If set to true, then npm will refuse to publish it.
  pub private: Option<bool>,

  /// Version must be parsable by node-semver, which is bundled with npm as a dependency.
  pub version: Option<String>,

  /// When set to `module`, the type field allows a package to specify all .js files within are ES modules. If the `type` field is omitted or set to `commonjs`, all .js files are treated as CommonJS.
  #[serde(rename = "type")]
  #[merge(skip)]
  pub type_: JsPackageType,

  /// Allows packages within a directory to depend on one another using direct linking of local files. Additionally, dependencies within a workspace are hoisted to the workspace root when possible to reduce duplication. Note: It's also a good idea to set `private` to true when using this feature.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub workspaces: Option<BTreeSet<String>>,

  /// A map of shell scripts to launch from the root of the package.
  #[merge(strategy = merge_btree_maps)]
  pub scripts: StringBTreeMap,

  /// The author of this package.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub author: Option<Person>,

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
  #[merge(strategy = merge_btree_sets)]
  pub keywords: BTreeSet<String>,

  /// The url to the project homepage.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub homepage: Option<String>,

  /// The single path for this package's binary, or a map of several binaries.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = overwrite_if_some)]
  pub bin: Option<Bin>,

  /// The 'files' field is an array of files to include in your project. If you name a folder in the array, then it will also include the files inside that folder.
  #[serde(skip_serializing_if = "BTreeSet::is_empty")]
  #[merge(strategy = merge_btree_sets)]
  pub files: BTreeSet<String>,

  /// The `exports` field is used to restrict external access to non-exported module files, also enables a module to import itself using `name`.
  #[serde(skip_serializing_if = "BTreeMap::is_empty")]
  #[merge(strategy = merge_btree_maps)]
  pub exports: BTreeMap<String, Exports>,

  /// Defines which package manager is expected to be used when working on the current project. This field is currently experimental and needs to be opted-in; see https://nodejs.org/api/corepack.html
  #[serde(alias = "package_manager")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub package_manager: Option<String>,

  /// Configuration settings for pnpm.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub pnpm: Option<Box<PnpmWorkspace>>,

  /// Overrides is used to support selective version overrides using npm, which lets you define custom package versions or ranges inside your dependencies. For yarn, use resolutions instead. See: https://docs.npmjs.com/cli/v9/configuring-npm/package-json#overrides
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_btree_maps)]
  pub overrides: Option<JsonValueBTreeMap>,

  /// Dependencies are specified with a simple hash of package name to version range. The version range is a string which has one or more space-separated descriptors. Dependencies can also be identified with a tarball or git URL.
  #[merge(strategy = merge_btree_maps)]
  pub dependencies: StringBTreeMap,

  /// Specifies dependencies that are required for the development and testing of the project. These dependencies are not needed in the production environment.
  // Necessary to have both camelCase and snake_case
  #[serde(alias = "dev_dependencies")]
  #[merge(strategy = merge_btree_maps)]
  pub dev_dependencies: StringBTreeMap,

  /// Specifies dependencies that are required by the package but are expected to be provided by the consumer of the package.
  #[serde(alias = "peer_dependencies")]
  #[serde(skip_serializing_if = "BTreeMap::is_empty")]
  #[merge(strategy = merge_btree_maps)]
  pub peer_dependencies: StringBTreeMap,

  /// When a user installs your package, warnings are emitted if packages specified in "peerDependencies" are not already installed. The "peerDependenciesMeta" field serves to provide more information on how your peer dependencies are utilized. Most commonly, it allows peer dependencies to be marked as optional. Metadata for this field is specified with a simple hash of the package name to a metadata object.
  #[serde(alias = "peer_dependencies_meta")]
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_btree_maps)]
  pub peer_dependencies_meta: Option<BTreeMap<String, PeerDependencyMeta>>,

  /// Specifies dependencies that are optional for your project. These dependencies are attempted to be installed during the npm install process, but if they fail to install, the installation process will not fail.
  #[serde(alias = "optional_dependencies")]
  #[serde(skip_serializing_if = "BTreeMap::is_empty")]
  #[merge(strategy = merge_btree_maps)]
  pub optional_dependencies: StringBTreeMap,

  /// Array of package names that will be bundled when publishing the package.
  #[serde(alias = "bundle_dependencies")]
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_btree_sets)]
  pub bundle_dependencies: Option<BTreeSet<String>>,

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
  #[merge(strategy = merge_btree_sets)]
  pub contributors: BTreeSet<Person>,

  /// A list of people who maintains this package.
  #[serde(skip_serializing_if = "BTreeSet::is_empty")]
  #[merge(strategy = merge_btree_sets)]
  pub maintainers: BTreeSet<Person>,

  /// Specify either a single file or an array of filenames to put in place for the man program to find.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub man: Option<Man>,

  /// An object that can be used to set configuration parameters used in package scripts that persist across upgrades.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[merge(strategy = merge_optional_btree_maps)]
  pub config: Option<JsonValueBTreeMap>,

  /// A set of config values that will be used at publish-time. It's especially handy if you want to set the tag, registry or access, so that you can ensure that a given package is not tagged with "latest", published to the global public registry or that a scoped module is private by default.
  #[serde(alias = "publish_config")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub publish_config: Option<PublishConfig>,

  /// Defines which tools and versions are expected to be used.
  #[serde(skip_serializing_if = "BTreeMap::is_empty")]
  #[merge(strategy = merge_btree_maps)]
  pub engines: StringBTreeMap,

  /// Specify which operating systems your module will run on.
  #[serde(skip_serializing_if = "BTreeSet::is_empty")]
  #[merge(strategy = merge_btree_sets)]
  pub os: BTreeSet<String>,

  /// Specify that your code only runs on certain cpu architectures.
  #[serde(skip_serializing_if = "BTreeSet::is_empty")]
  #[merge(strategy = merge_btree_sets)]
  pub cpu: BTreeSet<String>,

  #[serde(skip_serializing_if = "BTreeMap::is_empty")]
  #[serde(flatten)]
  #[merge(strategy = merge_btree_maps)]
  pub metadata: JsonValueBTreeMap,
}

impl Default for PackageJson {
  fn default() -> Self {
    Self {
      name: None,
      private: None,
      pnpm: None,
      overrides: None,
      bin: None,
      funding: None,
      type_: JsPackageType::Module,
      version: None,
      dependencies: Default::default(),
      peer_dependencies_meta: None,
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

impl PackageJson {
  #[allow(clippy::filter_map_bool_then)]
  /// Converts dependencies marked with `latest` into a version range starting from the latest version fetched with the npm API.
  pub async fn convert_latest_to_range(
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
    get_latest!(peer_dependencies, PeerDependency);

    for (kind, name) in names_to_update {
      let handle = tokio::spawn(async move {
        let actual_latest = get_latest_npm_version(&name).await.map_err(|e| {
          GenError::Custom(format!(
            "Could not get the latest version for npm package '{}': {}",
            name, e
          ))
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
          }
        }
        Ok(Err(task_error)) => return Err(task_error),
        Err(join_error) => {
          return Err(GenError::Custom(format!(
            "Failed to fetch an npm package's version due to an async task's failure: {}",
            join_error
          )))
        }
      };
    }

    Ok(())
  }
}

#[cfg(test)]
mod test {

  use std::{
    fs::{create_dir_all, File},
    path::PathBuf,
  };

  use maplit::{btreemap, btreeset};
  use pretty_assertions::assert_eq;
  use serde_json::Value;

  use super::{
    Bugs, Directories, Exports, JsPackageType, Man, PackageJson, PersonData, PublishConfig,
    PublishConfigAccess, Repository,
  };
  use crate::{
    fs::{get_parent_dir, serialize_json},
    ts::package_json::{Bin, Funding, FundingData, PeerDependencyMeta, Person},
  };

  fn convert_btreemap_to_json<T>(map: std::collections::BTreeMap<String, T>) -> Value
  where
    T: Into<Value>,
  {
    map.into_iter().collect()
  }

  #[test]
  fn package_json_gen() -> Result<(), Box<dyn std::error::Error>> {
    let test_package_json = PackageJson {
      pnpm: None,
      peer_dependencies_meta: Some(btreemap! {
        "abc".to_string() => PeerDependencyMeta { optional: Some(true), extras: btreemap! {
          "setting".to_string() => convert_btreemap_to_json(btreemap! {
              "inner".to_string() => "setting".to_string()
            })
          }
        },
        "abc2".to_string() => PeerDependencyMeta { optional: Some(true), extras: btreemap! {
          "setting".to_string() => convert_btreemap_to_json(btreemap! {
            "inner".to_string() => "setting".to_string()
          })
        }
        }
      }),
      private: Some(true),
      type_: JsPackageType::Module,
      bin: Some(Bin::Map(btreemap! {
        "bin1".to_string() => "bin/bin1".to_string(),
        "bin2".to_string() => "bin/bin2".to_string(),
      })),
      funding: Some(Funding::List(vec![
        Funding::Data(FundingData {
          url: "website".to_string(),
          type_: Some("collective".to_string()),
        }),
        Funding::Url("website.com".to_string()),
        Funding::Data(FundingData {
          url: "website".to_string(),
          type_: Some("individual".to_string()),
        }),
      ])),
      overrides: Some(btreemap! {
        "key".to_string() => convert_btreemap_to_json(btreemap! {
          "override".to_string() => "setting".to_string()
        })
      }),
      version: Some("0.1.0".to_string()),
      exports: btreemap! {
        ".".to_string() => Exports::Path("src/index.js".to_string()),
        "main".to_string() => Exports::Data {
          require: Some("src/index.js".to_string()),
          import: Some("src/index.js".to_string()),
          node: Some("src/index.js".to_string()),
          default: Some("src/index.js".to_string()),
          types: Some("src/index.js".to_string()),
          other: btreemap! { "extra".to_string() => "src/extra.js".to_string() }
        }
      },
      main: Some("dist/index.js".to_string()),
      browser: Some("dist/index.js".to_string()),
      author: Some(Person::Data(PersonData {
        url: Some("abc".to_string()),
        name: "abc".to_string(),
        email: Some("abc".to_string()),
      })),
      license: Some("Apache-2.0".to_string()),
      bugs: Some(Bugs {
        url: Some("abc".to_string()),
        email: Some("abc".to_string()),
      }),
      files: btreeset! { "dist".to_string() },
      homepage: Some("abc".to_string()),
      keywords: btreeset! { "something".to_string() },
      package_manager: Some("pnpm".to_string()),
      cpu: btreeset! { "arm64".to_string(), "x86".to_string() },
      os: btreeset! { "darwin".to_string(), "linux".to_string() },
      engines: btreemap! { "node".to_string() => "23.0.0".to_string(), "deno".to_string() => "2.0.0".to_string() },
      workspaces: Some(btreeset!["packages".to_string(), "apps".to_string()]),
      name: Some("my_package".to_string()),
      dev_dependencies: btreemap! { "typescript".to_string() => "7.0.0".to_string(), "vite".to_string() => "8.0.0".to_string() },
      dependencies: btreemap! { "typescript".to_string() => "7.0.0".to_string(), "vite".to_string() => "8.0.0".to_string() },
      bundle_dependencies: Some(btreeset! { "typescript".to_string(), "vite".to_string() }),
      optional_dependencies: btreemap! { "typescript".to_string() => "7.0.0".to_string(), "vite".to_string() => "8.0.0".to_string() },
      peer_dependencies: btreemap! { "typescript".to_string() => "7.0.0".to_string(), "vite".to_string() => "8.0.0".to_string() },
      description: Some("my_test".to_string()),
      scripts: btreemap! { "test".to_string() => "vitest run".to_string(), "dev".to_string() => "vite dev".to_string() },
      repository: Some(Repository::Data {
        type_: Some("abc".to_string()),
        url: Some("abc".to_string()),
        directory: Some("abc".to_string()),
      }),
      metadata: btreemap! {
        "specialConfig".to_string() => convert_btreemap_to_json(btreemap! {
          "prop1".to_string() => vec! [ "hello there".to_string(), "general kenobi".to_string() ],
          "prop2".to_string() => vec! [ "hello there".to_string(), "general kenobi".to_string() ],
        }),
        "specialConfig2".to_string() => convert_btreemap_to_json(btreemap! {
          "prop1".to_string() => vec! [ "hello there".to_string(), "general kenobi".to_string() ],
          "prop2".to_string() => vec! [ "hello there".to_string(), "general kenobi".to_string() ],
        }),
      },
      publish_config: Some(PublishConfig {
        access: Some(PublishConfigAccess::Public),
        tag: Some("abc".to_string()),
        registry: Some("abc".to_string()),
        other: btreemap! { "something".to_string() => "a thing".to_string(), "somethingElse".to_string() => "another thing".to_string() },
      }),
      man: Some(Man::List(vec!["man1".to_string(), "man2".to_string()])),
      config: Some(btreemap! {
        "myConfig".to_string() => convert_btreemap_to_json(btreemap! {
          "prop1".to_string() => vec! [ "hello there".to_string(), "general kenobi".to_string() ],
          "prop2".to_string() => vec! [ "hello there".to_string(), "general kenobi".to_string() ],
        }),
        "myConfig2".to_string() => convert_btreemap_to_json(btreemap! {
          "prop1".to_string() => vec! [ "hello there".to_string(), "general kenobi".to_string() ],
          "prop2".to_string() => vec! [ "hello there".to_string(), "general kenobi".to_string() ],
        }),
      }),
      contributors: btreeset! {
        Person::Data(PersonData {
          name: "legolas".to_string(),
          url: Some("legolas.com".to_string()),
          email: Some("legolas@middleearth.com".to_string()),
        }),
        Person::Data(PersonData {
          name: "aragorn".to_string(),
          url: Some("aragorn.com".to_string()),
          email: Some("aragorn@middleearth.com".to_string()),
        })
      },
      maintainers: btreeset! {
        Person::Data(PersonData {
          name: "legolas".to_string(),
          url: Some("legolas.com".to_string()),
          email: Some("legolas@middleearth.com".to_string()),
        }),
        Person::Data(PersonData {
          name: "aragorn".to_string(),
          url: Some("aragorn.com".to_string()),
          email: Some("aragorn@middleearth.com".to_string()),
        })
      },
      directories: Some(Directories {
        man: Some("abc".to_string()),
        test: Some("abc".to_string()),
        lib: Some("abc".to_string()),
        doc: Some("abc".to_string()),
        example: Some("abc".to_string()),
        bin: Some("abc".to_string()),
        other: btreemap! {
          "hello there".to_string() => "general kenobi".to_string(),
          "hello there".to_string() => "general kenobi".to_string(),
        },
      }),
    };

    let output_path = PathBuf::from("tests/output/package_json_gen/package.json");

    create_dir_all(get_parent_dir(&output_path)).unwrap();

    serialize_json(&test_package_json, &output_path, true)?;

    let result: PackageJson = serde_json::from_reader(File::open(&output_path)?)?;

    assert_eq!(test_package_json, result);

    Ok(())
  }
}
