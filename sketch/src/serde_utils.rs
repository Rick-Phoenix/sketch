use crate::*;

pub(crate) fn deserialize_map(path: &Path) -> Result<IndexMap<String, Value>, GenError> {
	let ext = path.extension().with_context(|| {
		format!(
			"Could not identify the type of the file `{}` for deserialization",
			path.display()
		)
	})?;

	let map: IndexMap<String, Value> = match ext.to_string_lossy().as_ref() {
		"json" => deserialize_json(path)?,
		"toml" => deserialize_toml(path)?,
		"yaml" => deserialize_yaml(path)?,
		_ => {
			return Err(anyhow!(
				"Could not deserialize file `{}` due to an unsupported extension. Allowed extensions are: yaml, toml, json", path.display()
			).into());
		}
	};

	Ok(map)
}
