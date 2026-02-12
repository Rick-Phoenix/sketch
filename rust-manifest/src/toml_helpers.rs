use super::*;

pub(crate) trait AsTomlValue {
	fn as_toml_value(&self) -> Item;
}

impl<T: Into<Item> + Clone> AsTomlValue for T {
	fn as_toml_value(&self) -> Item {
		self.clone().into()
	}
}

pub(crate) fn json_to_toml(json: &Value) -> Option<Item> {
	match json {
		Value::Null => None,

		Value::Bool(b) => Some(Item::Value(TomlValue::from(*b))),

		Value::Number(n) => {
			if let Some(i) = n.as_i64() {
				Some(Item::Value(TomlValue::from(i)))
			} else if let Some(f) = n.as_f64() {
				Some(Item::Value(TomlValue::from(f)))
			} else {
				Some(Item::Value(TomlValue::from(n.to_string())))
			}
		}

		Value::String(s) => Some(Item::Value(TomlValue::from(s))),

		Value::Array(vec) => {
			if vec.is_empty() {
				return Some(Item::Value(TomlValue::Array(Array::new())));
			}

			let all_objects = vec.iter().all(|v| v.is_object());

			if all_objects {
				// CASE A: [[bin]] style (Array of Tables)
				let mut array_of_tables = ArrayOfTables::new();

				for val in vec {
					// We know it's an object, so we force conversion to a standard Table
					if let Some(table) = json_to_standard_table(val) {
						array_of_tables.push(table);
					}
				}
				Some(Item::ArrayOfTables(array_of_tables))
			} else {
				// CASE B: features = ["a", "b"] style (Inline Array)
				let mut arr = Array::new();
				for val in vec {
					if let Some(item) = json_to_toml(val) {
						match item {
							Item::Value(v) => arr.push(v),
							Item::Table(t) => {
								// Inline arrays can't hold standard tables, convert to inline
								let mut inline = t.into_inline_table();
								InlineTable::fmt(&mut inline);
								arr.push(TomlValue::InlineTable(inline));
							}
							_ => {} // formatting error or invalid structure
						}
					}
				}

				format_array(&mut arr);
				Some(Item::Value(TomlValue::Array(arr)))
			}
		}

		Value::Object(_) => json_to_item_table(json),
	}
}

/// Used specifically for populating ArrayOfTables
pub(crate) fn json_to_standard_table(json: &Value) -> Option<Table> {
	if let Value::Object(map) = json {
		let mut table = Table::new();
		table.set_implicit(true);
		for (k, v) in map {
			if let Some(item) = json_to_toml(v) {
				table.insert(k, item);
			}
		}
		Some(table)
	} else {
		None
	}
}

/// Helper to decide between InlineTable vs Standard Table (for single objects)
pub(crate) fn json_to_item_table(json: &Value) -> Option<Item> {
	if let Value::Object(map) = json {
		// 1. Dependency Heuristic
		let is_dependency =
			map.contains_key("version") || map.contains_key("git") || map.contains_key("path");

		// 2. Complexity Heuristic
		let has_nested_objects = map.values().any(|v| v.is_object());
		let is_small = map.len() <= 3;

		if is_dependency || (is_small && !has_nested_objects) {
			// Inline Table: { version = "1.0" }
			let mut inline = InlineTable::new();
			for (k, v) in map {
				// We need values, not Items, for InlineTable
				if let Some(Item::Value(val)) = json_to_toml(v) {
					inline.insert(k, val);
				}
			}
			InlineTable::fmt(&mut inline);
			Some(Item::Value(TomlValue::InlineTable(inline)))
		} else {
			// Standard Table: [section]
			json_to_standard_table(json).map(Item::Table)
		}
	} else {
		None
	}
}

pub(crate) fn toml_string_list<'a>(strings: impl IntoIterator<Item = &'a String>) -> Item {
	let mut arr = Array::from_iter(strings);

	format_array(&mut arr);

	arr.into()
}

pub(crate) fn format_array(arr: &mut Array) {
	const MAX_INLINE_ITEMS: usize = 4;
	const MAX_INLINE_CHARS: usize = 50;

	let count = arr.len();

	let total_chars: usize = arr
		.iter()
		.map(|item| item.to_string().len())
		.sum();

	let has_tables = arr.iter().any(|item| item.is_inline_table());

	let should_expand = count > MAX_INLINE_ITEMS || total_chars > MAX_INLINE_CHARS || has_tables;

	if should_expand {
		for item in arr.iter_mut() {
			item.decor_mut().set_prefix("\n\t");
		}

		arr.set_trailing_comma(true);

		arr.set_trailing("\n");
	} else {
		arr.fmt();
	}
}
