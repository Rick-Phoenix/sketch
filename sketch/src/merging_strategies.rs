use crate::*;

pub trait ExtensiblePreset: Merge + Sized + Clone {
	fn kind() -> PresetKind;

	fn extended_ids(&mut self) -> &mut IndexSet<String>;

	fn merge_presets(
		self,
		current_id: &str,
		store: &IndexMap<String, Self>,
	) -> Result<Self, AppError> {
		self.merge_presets_recursive(current_id, store, &mut IndexSet::new())
	}

	fn merge_presets_recursive(
		mut self,
		current_id: &str,
		store: &IndexMap<String, Self>,
		processed_ids: &mut IndexSet<String>,
	) -> Result<Self, AppError> {
		let presets_to_extend = self.extended_ids().clone();

		if presets_to_extend.is_empty() {
			return Ok(self);
		}

		check_for_circular_dependencies(current_id, processed_ids, Self::kind())?;

		for id in &presets_to_extend {
			let mut extend_target = store
				.get(id)
				.ok_or_else(|| AppError::PresetNotFound {
					kind: Self::kind(),
					name: id.clone(),
				})?
				.clone()
				.merge_presets_recursive(id, store, processed_ids)?;

			extend_target.merge(self);

			self = extend_target;
		}

		*self.extended_ids() = presets_to_extend;

		Ok(self)
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
