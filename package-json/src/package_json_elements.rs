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

/// You can specify an object containing a URL that provides up-to-date information about ways to help fund development of your package, a string URL, or an array of objects and string URLs.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
pub enum Funding {
	Url(String),
	Data(FundingData),
	List(Vec<Self>),
}

/// Used to inform about ways to help fund development of the package.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct FundingData {
	/// The type of funding or the platform through which funding can be provided, e.g. patreon, opencollective, tidelift or github
	#[serde(rename = "type", skip_serializing_if = "Option::is_none")]
	pub type_: Option<String>,

	pub url: String,
}

/// The single path for this package's binary, or a map of several binaries.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
pub enum Bin {
	Single(String),
	Map(StringBTreeMap),
}

/// An enum representing formats for the `repository` field in a `package.json` file.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
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

/// A struct representing the `bugs` field in a `package.json` file.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Bugs {
	/// The url to your project's issue tracker.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub url: Option<String>,

	/// The email address to which issues should be reported.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub email: Option<String>,
}

/// A struct that represents how an individual's information is represented in a `package.json` file in the author, maintainers and contributors fields.
#[derive(Clone, Debug, Serialize, Deserialize, Default, Ord, PartialEq, PartialOrd, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Person {
	pub name: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub email: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub url: Option<String>,
}

/// A struct that represents a value in the `exports` object inside a `package.json` file.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
pub enum Exports {
	Path(String),
	Data {
		#[serde(default, skip_serializing_if = "Option::is_none")]
		types: Option<String>,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		import: Option<String>,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		require: Option<String>,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		node: Option<String>,
		#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
		#[serde(flatten)]
		other: StringBTreeMap,
		#[serde(default, skip_serializing_if = "Option::is_none")]
		default: Option<String>,
	},
}

/// A struct that represents the value of the `directories` field in a `package.json` file.
#[derive(Clone, Debug, Serialize, Default, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default)]
pub struct Directories {
	/// If you specify a `bin` directory, then all the files in that folder will be used as the `bin` hash.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub bin: Option<String>,

	/// Tell people where the bulk of your library is. Nothing special is done with the lib folder in any way, but it's useful meta info.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub lib: Option<String>,

	/// Put markdown files in here. Eventually, these will be displayed nicely, maybe, someday.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub doc: Option<String>,

	/// Put example scripts in here. Someday, it might be exposed in some clever way.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub example: Option<String>,

	/// A folder that is full of man pages. Sugar to generate a 'man' array by walking the folder.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub man: Option<String>,

	/// The tests directory.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub test: Option<String>,

	#[serde(skip_serializing_if = "BTreeMap::is_empty")]
	#[serde(flatten)]
	pub other: StringBTreeMap,
}

/// A struct that represents the kinds of values for the `man` field of a `package.json` file.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
pub enum Man {
	Path(String),
	List(Vec<String>),
}

/// The values that can be used to define `access` in a [`PublishConfig`]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
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

/// A set of config values that will be used at publish-time. It's especially handy if you want to set the tag, registry or access, so that you can ensure that a given package is not tagged with "latest", published to the global public registry or that a scoped module is private by default.
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default)]
pub struct PublishConfig {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub access: Option<PublishConfigAccess>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub tag: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub registry: Option<String>,
	#[serde(skip_serializing_if = "BTreeMap::is_empty")]
	#[serde(flatten)]
	pub other: StringBTreeMap,
}

/// The type of JS package.
#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum JsPackageType {
	#[serde(rename = "module")]
	#[default]
	Module,
	CommonJs,
}

impl Display for JsPackageType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Module => write!(f, "module"),
			Self::CommonJs => write!(f, "CommonJs"),
		}
	}
}

#[derive(Debug, Clone)]
pub enum JsDepKind {
	Dependency,
	DevDependency,
	OptionalDependency,
	PeerDependency,
	CatalogDependency(Option<String>),
}
