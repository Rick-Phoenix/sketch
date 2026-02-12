use crate::*;

pub trait ExtensiblePreset: Merge + Sized + Clone {
	fn kind() -> PresetKind;

	fn get_extended_ids(&self) -> &IndexSet<String>;

	fn merge_presets(
		self,
		current_id: &str,
		store: &IndexMap<String, Self>,
	) -> Result<Self, AppError> {
		self.merge_presets_recursive(current_id, store, &mut IndexSet::new())
	}

	fn merge_presets_recursive(
		self,
		current_id: &str,
		store: &IndexMap<String, Self>,
		processed_ids: &mut IndexSet<String>,
	) -> Result<Self, AppError> {
		let presets_to_extend = self.get_extended_ids();

		if presets_to_extend.is_empty() {
			return Ok(self);
		}

		check_for_circular_dependencies(current_id, processed_ids, Self::kind())?;

		let mut base_preset: Option<Self> = None;

		for id in presets_to_extend {
			let extend_target = store
				.get(id)
				.ok_or(AppError::PresetNotFound {
					kind: Self::kind(),
					name: id.clone(),
				})?
				.clone();

			let complete_target =
				extend_target.merge_presets_recursive(id, store, processed_ids)?;

			if let Some(base) = base_preset.as_mut() {
				base.merge(complete_target);
			} else {
				base_preset = Some(complete_target)
			}
		}

		// Can never be None due to the early exit
		let mut aggregated = base_preset.unwrap();

		// We merge `self` last because this is an extension more than a merge,
		// the last element is the most relevant
		aggregated.merge(self);

		Ok(aggregated)
	}
}

fn check_for_circular_dependencies(
	id: &str,
	processed_ids: &mut IndexSet<String>,
	preset_kind: PresetKind,
) -> Result<(), AppError> {
	let was_absent = processed_ids.insert(id.to_string());

	if !was_absent {
		let chain: Vec<&str> = processed_ids.iter().map(|s| s.as_str()).collect();

		return Err(AppError::CircularDependency(format!(
			"Found circular {preset_kind:?} dependency for preset with id '{id}'. The full processed chain is: {}",
			chain.join(" -> ")
		)));
	}

	Ok(())
}
