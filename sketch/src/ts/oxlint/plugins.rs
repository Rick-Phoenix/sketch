use std::collections::{BTreeMap, BTreeSet};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The plugins for oxlint. See more: https://oxc.rs/docs/guide/usage/linter/plugins.html
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Plugins {
	Eslint,
	Import,
	Jest,
	Jsdoc,
	#[serde(rename = "jsx-a11y")]
	JsxA11y,
	NextJs,
	Node,
	Oxc,
	Promise,
	React,
	#[serde(rename = "react-perf")]
	ReactPerf,
	Regex,
	Typescript,
	Unicorn,
	Vitest,
	Vue,
}

impl Plugins {
	pub const fn as_str(&self) -> &str {
		match self {
			Self::Eslint => "eslint",
			Self::Import => "import",
			Self::Jest => "jest",
			Self::Jsdoc => "jsdoc",
			Self::JsxA11y => "jsx-a11y",
			Self::NextJs => "nextjs",
			Self::Node => "node",
			Self::Oxc => "oxc",
			Self::Promise => "promise",
			Self::React => "react",
			Self::ReactPerf => "react-perf",
			Self::Regex => "regex",
			Self::Typescript => "typescript",
			Self::Unicorn => "unicorn",
			Self::Vitest => "vitest",
			Self::Vue => "vue",
		}
	}
}

impl Plugin {
	pub const fn as_str(&self) -> &str {
		match self {
			Self::Known(variant) => variant.as_str(),
			Self::Custom(name) => name.as_str(),
		}
	}
}

impl PartialOrd for Plugin {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for Plugin {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.as_str().cmp(other.as_str())
	}
}

/// Ways of referring to a plugin.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(untagged)]
pub enum Plugin {
	Known(Plugins),
	Custom(String),
}

/// Specifies allows custom tags for Jsdoc annotations.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(untagged, rename_all = "camelCase")]
pub enum TagNamePreference {
	String(String),
	Data {
		message: String,
		replacement: String,
	},
	Message {
		message: String,
	},
	Bool(bool),
}

/// Settings for the Jsdoc plugin. See more: https://oxc.rs/docs/guide/usage/linter/config-file-reference.html#settings-jsdoc
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct JsDocPluginSettings {
	/// Only for `require-(yields|returns|description|example|param|throws)` rules.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub augments_extends_replaces_docs: Option<bool>,

	/// Only for `require-param-type` and `require-param-description` rule.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub exempt_destructured_roots_from_chekcs: Option<bool>,

	/// For all rules but NOT apply to `empty-tags` rule.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub ignore_internal: Option<bool>,

	/// For all rules but NOT apply to `check-access` and `empty-tags` rule.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub ignore_private: Option<bool>,

	/// Only for `require-(yields|returns|description|example|param|throws)` rules.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub ignore_replaces_docs: Option<bool>,

	/// Only for `require-(yields|returns|description|example|param|throws)` rules.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub implements_replaces_docs: Option<bool>,

	/// Only for `require-(yields|returns|description|example|param|throws)` rules.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub override_replaces_docs: Option<bool>,

	/// Specifies allows custom tags for Jsdoc annotations.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub tag_name_preference: Option<BTreeMap<String, TagNamePreference>>,
}

/// Settings for the jsx-a11y plugin. See more: https://github.com/jsx-eslint/eslint-plugin-jsx-a11y#configurations
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct JsxA11yPluginSettings {
	/// Map of attribute names to their DOM equivalents.This is useful for non-React frameworks that use different attribute names.
	///
	/// Example:
	/// ```json
	/// {
	///   "settings\":
	///   {
	///     "jsx-a11y":
	///     {
	///       "attributes": {
	///         "for": [
	///           "htmlFor",
	///           "for"
	///         ]
	///       }
	///     }
	///   }
	/// }
	/// ```
	#[serde(skip_serializing_if = "Option::is_none")]
	pub attributes: Option<BTreeMap<String, BTreeSet<String>>>,

	/// To have your custom components be checked as DOM elements, you can\nprovide a mapping of your component names to the DOM element name.
	///
	/// Example:
	/// ```json
	/// {
	///   "settings": {
	///     "jsx-a11y": {
	///       "components": {
	///         "Link": "a",
	///         "IconButton": "button"
	///       }
	///     }
	///   }
	/// }
	/// ```
	#[serde(skip_serializing_if = "Option::is_none")]
	pub components: Option<BTreeMap<String, String>>,

	/// An optional setting that define the prop your code uses to create polymorphic components.
	/// This setting will be used to determine the element type in rules that
	/// require semantic context.
	///
	/// For example, if you set the `polymorphicPropName` to `as`, then this element:
	///
	/// ```jsx
	/// <Box as="h3">Hello</Box>
	/// ```
	///
	/// Will be treated as an `h3`. If not set, this component will be treated
	/// as a `Box`.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub polymorphic_prop_name: Option<String>,
}

/// Settings for the nextjs plugin. See more: https://oxc.rs/docs/guide/usage/linter/config-file-reference.html#settings-next
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct NextPluginSettings {
	/// The root directory of the Next.js project.
	///
	/// This is particularly useful when you have a monorepo and your Next.js
	/// project is in a subfolder.
	///
	/// Example:
	///
	/// ```json
	/// {
	///   "settings": {
	///     "next": {
	///       "rootDir": "apps/dashboard/"
	///     }
	///   }
	/// }
	/// ```
	#[serde(skip_serializing_if = "Option::is_none")]
	pub root_dir: Option<OneOrManyStrings>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(untagged)]
pub enum OneOrManyStrings {
	One(String),
	Many(BTreeSet<String>),
}

/// Settings for the react plugin. See more: https://oxc.rs/docs/guide/usage/linter/config-file-reference.html#settings-react
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct ReactPluginSettings {
	/// Components used as alternatives to `<form>` for forms, such as `<Formik>`.
	///
	/// Example:
	///
	/// ```jsonc
	/// {
	///   "settings": {
	///     "react": {
	///       "formComponents": [
	///         "CustomForm",
	///         // OtherForm is considered a form component and has an endpoint attribute
	///         { "name": "OtherForm", "formAttribute": "endpoint" },
	///         // allows specifying multiple properties if necessary
	///         { "name": "Form", "formAttribute": ["registerEndpoint", "loginEndpoint"] }
	///       ]
	///     }
	///   }
	/// }
	/// ```
	#[serde(skip_serializing_if = "Option::is_none")]
	pub form_components: Option<Vec<CustomComponent>>,

	/// Components used as alternatives to `<a>` for linking, such as `<Link>`.
	///
	/// Example:
	///
	/// ```jsonc
	/// {
	///   "settings": {
	///     "react": {
	///       "linkComponents": [
	///         "HyperLink",
	///         // Use `linkAttribute` for components that use a different prop name
	///         // than `href`.
	///         { "name": "MyLink", "linkAttribute": "to" },
	///         // allows specifying multiple properties if necessary
	///         { "name": "Link", "linkAttribute": ["to", "href"] }
	///       ]
	///     }
	///   }
	/// }
	/// ```
	#[serde(skip_serializing_if = "Option::is_none")]
	pub link_components: Option<Vec<CustomComponent>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(untagged)]
pub enum CustomComponent {
	NameOnly(String),
	ObjectWithOneAttr {
		name: String,
		#[serde(alias = "formAttribute", alias = "linkAttribute")]
		attribute: String,
	},
	ObjectWithManyAttrs {
		name: String,
		#[serde(alias = "formAttribute", alias = "linkAttribute")]
		attributes: BTreeSet<String>,
	},
}
