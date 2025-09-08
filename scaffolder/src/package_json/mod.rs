use std::collections::{BTreeMap, BTreeSet};

use askama::Template;
use futures::future;
use indexmap::IndexMap;
use merge::Merge;
pub use package_json_elements::*;
use serde::{Deserialize, Serialize};

use crate::{
  merging_strategies::*, rendering::render_json_val, versions::get_latest_version, GenError,
  JsonValueBTreeMap, Preset, StringBTreeMap, VersionRange,
};
pub mod package_json_elements;

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum PackageJsonKind {
  Id(String),
  Config(Box<PackageJson>),
}

/// A struct representing the contents of a package.json file.
#[derive(Debug, Deserialize, Serialize, Template, Merge, Clone, PartialEq, Eq)]
#[template(path = "package.json.j2")]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct PackageJson {
  #[merge(skip)]
  pub name: String,
  #[merge(strategy = merge::bool::overwrite_true)]
  pub private: bool,
  #[merge(skip)]
  pub type_: JsModuleType,
  #[merge(skip)]
  pub version: String,
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
  #[merge(skip)]
  pub workspaces: BTreeSet<String>,
  #[merge(skip)]
  pub directories: Option<Directories>,
  #[serde(flatten)]
  #[merge(strategy = merge_btree_maps)]
  pub metadata: JsonValueBTreeMap,
}

impl Default for PackageJson {
  fn default() -> Self {
    Self {
      name: "my-awesome-package".to_string(),
      private: true,
      type_: JsModuleType::Module,
      version: "0.1.0".to_string(),
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
      directories: Default::default(),
    }
  }
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

#[cfg(test)]
mod test {
  use std::{fs::File, path::PathBuf};

  use askama::Template;
  use maplit::{btreemap, btreeset};

  use super::{
    Bugs, Directories, Exports, JsModuleType, Man, PackageJson, Person, PersonData, PublishConfig,
    PublishConfigAccess, Repository,
  };
  use crate::{convert_btreemap_to_json, GenError};

  #[test]
  fn package_json_gen() -> Result<(), Box<dyn std::error::Error>> {
    let test_package_json = PackageJson {
      private: true,
      type_: JsModuleType::Module,
      version: "0.1.0".to_string(),
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
      workspaces: btreeset!["packages".to_string(), "apps".to_string()],
      name: "my_package".to_string(),
      dev_dependencies: btreemap! { "typescript".to_string() => "7.0.0".to_string(), "vite".to_string() => "8.0.0".to_string() },
      dependencies: btreemap! { "typescript".to_string() => "7.0.0".to_string(), "vite".to_string() => "8.0.0".to_string() },
      bundle_dependencies: btreemap! { "typescript".to_string() => "7.0.0".to_string(), "vite".to_string() => "8.0.0".to_string() },
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
      extends: Default::default(),
      use_default_deps: true,
    };

    let output_path = PathBuf::from("output/test/package_json/package.json");
    let mut output_file = File::create(&output_path).map_err(|e| GenError::FileCreation {
      path: output_path.clone(),
      source: e,
    })?;

    test_package_json.write_into(&mut output_file)?;

    let result: PackageJson = serde_json::from_reader(File::open(&output_path)?)?;

    assert_eq!(test_package_json, result);

    Ok(())
  }
}
