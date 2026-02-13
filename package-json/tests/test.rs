use std::{
	fs::{File, create_dir_all},
	path::PathBuf,
};

use maplit::{btreemap, btreeset};
use pretty_assertions::assert_eq;
use serde_json::Value;

use package_json::*;

fn convert_btreemap_to_json<T>(map: std::collections::BTreeMap<String, T>) -> Value
where
	T: Into<Value>,
{
	map.into_iter().collect()
}

#[test]
fn package_json_gen() -> Result<(), Box<dyn std::error::Error>> {
	let test_package_json = PackageJson {
		catalog: Default::default(),
		catalogs: Default::default(),
		#[cfg(feature = "pnpm")]
		pnpm: None,
		peer_dependencies_meta: btreemap! {
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
		},
		private: true,
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
		overrides: btreemap! {
			"key".to_string() => convert_btreemap_to_json(btreemap! {
				"override".to_string() => "setting".to_string()
			})
		},
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
		author: Some(Person::Data(Person {
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
		bundle_dependencies: btreeset! { "typescript".to_string(), "vite".to_string() },
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
		config: btreemap! {
			"myConfig".to_string() => convert_btreemap_to_json(btreemap! {
				"prop1".to_string() => vec! [ "hello there".to_string(), "general kenobi".to_string() ],
				"prop2".to_string() => vec! [ "hello there".to_string(), "general kenobi".to_string() ],
			}),
			"myConfig2".to_string() => convert_btreemap_to_json(btreemap! {
				"prop1".to_string() => vec! [ "hello there".to_string(), "general kenobi".to_string() ],
				"prop2".to_string() => vec! [ "hello there".to_string(), "general kenobi".to_string() ],
			}),
		},
		contributors: btreeset! {
			Person::Data(Person {
				name: "legolas".to_string(),
				url: Some("legolas.com".to_string()),
				email: Some("legolas@middleearth.com".to_string()),
			}),
			Person::Data(Person {
				name: "aragorn".to_string(),
				url: Some("aragorn.com".to_string()),
				email: Some("aragorn@middleearth.com".to_string()),
			})
		},
		maintainers: btreeset! {
			Person::Data(Person {
				name: "legolas".to_string(),
				url: Some("legolas.com".to_string()),
				email: Some("legolas@middleearth.com".to_string()),
			}),
			Person::Data(Person {
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

	let output_path = PathBuf::from(concat!(
		env!("CARGO_MANIFEST_DIR"),
		"/tests/output/package.json"
	));

	create_dir_all(output_path.parent().unwrap()).unwrap();

	serde_json::to_writer_pretty(File::create(&output_path).unwrap(), &test_package_json)?;

	let result: PackageJson = serde_json::from_reader(File::open(&output_path)?)?;

	assert_eq!(test_package_json, result);

	Ok(())
}
