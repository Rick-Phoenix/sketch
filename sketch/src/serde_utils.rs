use crate::*;

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
pub enum StringOrList {
	String(String),
	List(Vec<String>),
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
pub enum ListOrMap {
	List(BTreeSet<String>),
	Map(BTreeMap<String, String>),
}

impl ListOrMap {
	pub fn contains(&self, key: &str) -> bool {
		match self {
			Self::List(list) => list.contains(key),
			Self::Map(map) => map.contains_key(key),
		}
	}

	pub fn get(&self, key: &str) -> Option<&String> {
		match self {
			Self::List(list) => list.get(key),
			Self::Map(map) => map.get(key),
		}
	}
}

pub(crate) fn deserialize_map(path: &Path) -> Result<IndexMap<String, Value>, GenError> {
	let ext = path.extension().ok_or(generic_error!(
		"Could not identify the type of the file `{path:?}` for deserialization"
	))?;

	let map: IndexMap<String, Value> = match ext.to_string_lossy().as_ref() {
		"json" => deserialize_json(path)?,
		"toml" => deserialize_toml(path)?,
		"yaml" => deserialize_yaml(path)?,
		_ => {
			return Err(generic_error!(
				"Could not deserialize file `{path:?}` due to an unsupported extension. Allowed extensions are: yaml, toml, json"
			));
		}
	};

	Ok(map)
}

pub(crate) fn merge_list_or_map(left: &mut Option<ListOrMap>, right: Option<ListOrMap>) {
	if let Some(right) = right {
		if let Some(left_data) = left {
			if let ListOrMap::List(left_list) = left_data
				&& let ListOrMap::List(right_list) = right
			{
				left_list.extend(right_list);
			} else if let ListOrMap::Map(left_list) = left_data
				&& let ListOrMap::Map(right_list) = right
			{
				left_list.extend(right_list);
			} else {
				*left = Some(right);
			}
		} else {
			*left = Some(right);
		}
	}
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
pub enum StringOrSortedList {
	String(String),
	List(BTreeSet<String>),
}

impl Merge for StringOrSortedList {
	fn merge(&mut self, right: Self) {
		match self {
			Self::String(left_string) => {
				if let Self::List(mut right_list) = right {
					right_list.insert(left_string.clone());
					*self = Self::List(right_list);
				} else {
					*self = right;
				}
			}
			Self::List(left_list) => match right {
				Self::String(right_string) => {
					left_list.insert(right_string);
				}
				Self::List(right_list) => {
					for item in right_list {
						left_list.insert(item);
					}
				}
			},
		}
	}
}

pub(crate) fn merge_optional_string_or_sorted_list(
	left: &mut Option<StringOrSortedList>,
	right: Option<StringOrSortedList>,
) {
	if let Some(right) = right {
		if let Some(left_data) = left {
			if let StringOrSortedList::List(left_list) = left_data
				&& let StringOrSortedList::List(right_list) = right
			{
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
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
