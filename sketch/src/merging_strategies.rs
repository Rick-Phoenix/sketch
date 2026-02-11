use crate::*;

pub trait Extensible {
	fn get_extended(&self) -> &IndexSet<String>;
}

fn process_preset_id(
	id: &str,
	processed_ids: &mut IndexSet<String>,
	preset_kind: Preset,
) -> Result<(), GenError> {
	let was_absent = processed_ids.insert(id.to_string());

	if !was_absent {
		let chain: Vec<&str> = processed_ids.iter().map(|s| s.as_str()).collect();

		return Err(GenError::CircularDependency(format!(
			"Found circular {:?} dependency for '{}'. The full processed chain is: {}",
			preset_kind,
			id,
			chain.join(" -> ")
		)));
	}

	Ok(())
}

pub(crate) fn merge_presets<T: Merge + Extensible + Clone>(
	preset_kind: Preset,
	current_id: &str,
	preset: T,
	store: &IndexMap<String, T>,
	processed_ids: &mut IndexSet<String>,
) -> Result<T, GenError> {
	process_preset_id(current_id, processed_ids, preset_kind)?;

	let presets_to_extend = preset.get_extended();

	if presets_to_extend.is_empty() {
		return Ok(preset);
	}

	let mut base: Option<T> = None;

	for id in presets_to_extend {
		let extend_target = store
			.get(id)
			.ok_or(GenError::PresetNotFound {
				kind: preset_kind,
				name: id.clone(),
			})?
			.clone();

		let complete_target = merge_presets(preset_kind, id, extend_target, store, processed_ids)?;

		if let Some(aggregated) = base.as_mut() {
			aggregated.merge(complete_target);
		} else {
			base = Some(complete_target)
		}
	}

	// Can never be None due to the early exit
	let mut aggregated = base.unwrap();

	aggregated.merge(preset);

	Ok(aggregated)
}
