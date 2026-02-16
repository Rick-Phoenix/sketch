use crate::{cli::parsers::parse_key_value_pairs, *};
pub(crate) use gh_workflow_config::{
	GhJobPreset, GhJobPresetRef, Job, Step, StepPresetRef, Workflow,
};

impl GhWorkflowPresetRef {
	pub(crate) fn from_cli(s: &str) -> Result<Self, String> {
		let mut file: Option<PathBuf> = None;
		let mut id: Option<String> = None;

		let pairs = parse_key_value_pairs("WorkflowReference", s)?;

		for (key, val) in pairs {
			match key {
				"file" => {
					file = if val.is_empty() {
						None
					} else {
						Some(val.into())
					}
				}
				"id" => {
					id = if val.is_empty() {
						None
					} else {
						Some(val.to_string())
					}
				}
				_ => return Err(format!("Invalid key for WorkflowReference: {key}")),
			};
		}

		let error_message = "Invalid input for a github workflow reference";

		let reference = Self::PresetId {
			file_name: file.ok_or_else(|| error_message.to_string())?,
			id: id.ok_or_else(|| error_message.to_string())?,
		};

		Ok(reference)
	}
}

/// The definition for a new Github workflow, or a reference to a preset.
#[derive(Clone, Deserialize, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
pub enum GhWorkflowPresetRef {
	/// A reference to a workflow preset
	PresetId {
		/// The name of the output file (inside the .github/workflows directory)
		file_name: PathBuf,
		/// The ID of the preset to use
		id: String,
	},

	/// An inlined definition for a new workflow
	Preset {
		/// The name of the output file (inside the .github/workflows directory)
		file_name: PathBuf,
		/// The definition for the new workflow
		workflow: GhWorkflowPreset,
	},
}

/// Configurations and presets relating to Github
#[derive(Clone, Deserialize, Debug, PartialEq, Eq, Serialize, Default, Merge)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct GithubConfig {
	/// A map of presets for Github workflows
	pub workflow_presets: IndexMap<String, GhWorkflowPreset>,

	/// A map of presets for Github workflow jobs
	pub workflow_job_presets: IndexMap<String, GhJobPreset>,

	/// A map of presets for steps used in a Github workflow job.
	pub steps_presets: IndexMap<String, Step>,
}

impl GithubConfig {
	pub fn get_workflow(&self, id: &str) -> AppResult<Workflow> {
		self.workflow_presets
			.get(id)
			.ok_or_else(|| AppError::PresetNotFound {
				kind: PresetKind::GithubWorkflow,
				name: id.to_string(),
			})?
			.clone()
			.process_data(id, self)
	}
}

/// A preset for a github workflow.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Merge)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct GhWorkflowPreset {
	/// The list of extended presets.
	#[serde(default)]
	#[merge(skip)]
	pub extends_presets: IndexSet<String>,

	#[serde(flatten)]
	pub config: Workflow,
}

impl ExtensiblePreset for GhWorkflowPreset {
	fn kind() -> PresetKind {
		PresetKind::GithubWorkflow
	}

	fn extended_ids(&mut self) -> &mut IndexSet<String> {
		&mut self.extends_presets
	}
}

impl GhWorkflowPreset {
	pub fn process_data(
		self,
		id: &str,
		github_config: &GithubConfig,
	) -> Result<Workflow, AppError> {
		if self.extends_presets.is_empty()
			&& !self
				.config
				.jobs
				.values()
				.any(|j| j.requires_processing())
		{
			return Ok(self.config);
		}

		let mut merged_preset = self.merge_presets(id, &github_config.workflow_presets)?;

		for job in merged_preset.config.jobs.values_mut() {
			match job {
				GhJobPresetRef::PresetId(id) => {
					let mut data = github_config
						.workflow_job_presets
						.get(id)
						.ok_or_else(|| AppError::PresetNotFound {
							kind: PresetKind::GithubWorkflowJob,
							name: id.clone(),
						})?
						.clone();

					if data.requires_processing() {
						data = github_config.process_gh_job_preset(id, data)?;
					}

					*job = GhJobPresetRef::Preset(data.into());
				}
				GhJobPresetRef::Preset(data) => {
					if data.requires_processing() {
						let mut owned_data = mem::take(data);

						*owned_data =
							github_config.process_gh_job_preset("__inlined", *owned_data)?;

						*data = owned_data;
					}
				}
			};
		}

		Ok(merged_preset.config)
	}
}

impl ExtensiblePreset for GhJobPreset {
	fn kind() -> PresetKind {
		PresetKind::GithubWorkflowJob
	}

	fn extended_ids(&mut self) -> &mut IndexSet<String> {
		&mut self.extends_presets
	}
}

impl GithubConfig {
	pub fn process_gh_job_preset(
		&self,
		id: &str,
		preset: GhJobPreset,
	) -> Result<GhJobPreset, AppError> {
		let mut merged_preset = preset.merge_presets(id, &self.workflow_job_presets)?;

		if let Job::Normal(job) = &mut merged_preset.job {
			for step in job.steps.iter_mut() {
				if let StepPresetRef::PresetId(id) = step {
					let data = self
						.steps_presets
						.get(id)
						.ok_or_else(|| AppError::PresetNotFound {
							kind: PresetKind::GithubWorkflowStep,
							name: id.clone(),
						})?
						.clone();

					*step = StepPresetRef::Preset(Box::new(data));
				}
			}
		}

		Ok(merged_preset)
	}
}
