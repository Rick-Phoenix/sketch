use crate::*;

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
