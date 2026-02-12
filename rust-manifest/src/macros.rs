macro_rules! prop_name {
	($name:ident) => {
		&stringify!($name).replace("_", "-")
	};
}

macro_rules! add_if_false {
	($target:ident, $table:ident => $($names:ident),*) => {
		$(
			if $target.$names.is_some_and(|v| !v) {
				$table.insert(prop_name!($names), false.into());
			}
		)*
	};
}

macro_rules! add_string {
	($target:ident, $table:ident => $($names:ident),*) => {
		$(
			if let Some(str) = &$target.$names {
				$table.insert(prop_name!($names), str.into());
			}
		)*
	};
}

macro_rules! add_bool {
	($target:ident, $table:ident => $($names:ident),*) => {
		$(
			if $target.$names {
				$table.insert(prop_name!($names), true.into());
			}
		)*
	};
}

macro_rules! add_optional_bool {
	($target:ident, $table:ident => $($names:ident),*) => {
		$(
			if let Some(bool) = $target.$names {
				$table.insert(prop_name!($names), bool.into());
			}
		)*
	};
}

macro_rules! add_value {
	($target:ident, $table:ident => $($names:ident),*) => {
		$(
			if let Some(val) = &$target.$names {
				$table.insert(prop_name!($names), val.as_toml_value().into());
			}
		)*
	};
}

macro_rules! add_table {
	($target:ident, $table:ident => $($names:ident),*) => {
		$(
			if !$target.$names.is_empty() {
				let mut table = Table::from_iter(
					$target.$names.iter().map(
						|(k, v)| (toml_edit::Key::from(k), Item::from(v.as_toml_value()))
					)
				);

				table.set_implicit(true);
				$table.insert(prop_name!($names), table.into());
			}
		)*
	};
}

macro_rules! add_string_list {
	($target:ident, $table:ident => $($names:ident),*) => {
		$(
			if !$target.$names.is_empty() {
				let mut array = Array::from_iter(&$target.$names);

				format_array(&mut array);

				$table.insert(prop_name!($names), array.into());
			}
		)*
	};
}
