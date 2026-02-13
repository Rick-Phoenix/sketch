use crate::{cli::parsers::parse_key_value_pairs, *};
pub(crate) use ::gh_workflow::{Job, JobPreset, JobPresetRef, Step, StepPresetRef, Workflow};

impl WorkflowReference {
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

		let reference = Self::Preset {
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
pub enum WorkflowReference {
	/// A reference to a workflow preset
	Preset {
		/// The name of the output file (inside the .github/workflows directory)
		file_name: PathBuf,
		/// The ID of the preset to use
		id: String,
	},

	/// An inlined definition for a new workflow
	Data {
		/// The name of the output file (inside the .github/workflows directory)
		file_name: PathBuf,
		/// The definition for the new workflow
		workflow: GithubWorkflowPreset,
	},
}

/// Configurations and presets relating to Github
#[derive(Clone, Deserialize, Debug, PartialEq, Eq, Serialize, Default, Merge)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct GithubConfig {
	/// A map of presets for Github workflows
	pub workflow_presets: IndexMap<String, GithubWorkflowPreset>,

	/// A map of presets for Github workflow jobs
	pub workflow_job_presets: IndexMap<String, JobPreset>,

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

/// A preset for a gihub workflow.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Merge)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct GithubWorkflowPreset {
	/// The list of extended presets.
	#[serde(default)]
	#[merge(skip)]
	pub extends_presets: IndexSet<String>,

	#[serde(flatten)]
	pub config: Workflow,
}

impl ExtensiblePreset for GithubWorkflowPreset {
	fn kind() -> PresetKind {
		PresetKind::GithubWorkflow
	}

	fn extended_ids(&mut self) -> &mut IndexSet<String> {
		&mut self.extends_presets
	}
}

impl GithubWorkflowPreset {
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
				JobPresetRef::PresetId(id) => {
					let mut data = github_config
						.workflow_job_presets
						.get(id)
						.ok_or_else(|| AppError::PresetNotFound {
							kind: PresetKind::GithubWorkflowJob,
							name: id.clone(),
						})?
						.clone();

					if data.requires_processing() {
						data = process_gh_job_preset(
							data,
							id,
							&github_config.workflow_job_presets,
							&github_config.steps_presets,
						)?;
					}

					*job = JobPresetRef::Data(data.into());
				}
				JobPresetRef::Data(data) => {
					if data.requires_processing() {
						let owned_data = mem::take(data);
						*data = process_gh_job_preset(
							*owned_data,
							"__inlined",
							&github_config.workflow_job_presets,
							&github_config.steps_presets,
						)?
						.into();
					}
				}
			};
		}

		Ok(merged_preset.config)
	}
}

impl ExtensiblePreset for JobPreset {
	fn kind() -> PresetKind {
		PresetKind::GithubWorkflowJob
	}

	fn extended_ids(&mut self) -> &mut IndexSet<String> {
		&mut self.extends_presets
	}
}

pub fn process_gh_job_preset(
	preset: JobPreset,
	id: &str,
	store: &IndexMap<String, JobPreset>,
	steps_store: &IndexMap<String, Step>,
) -> Result<JobPreset, AppError> {
	let mut merged_preset = preset.merge_presets(id, store)?;

	if let Job::Normal(job) = &mut merged_preset.job {
		for step in job.steps.iter_mut() {
			if let StepPresetRef::PresetId(id) = step {
				let data = steps_store
					.get(id)
					.ok_or_else(|| AppError::PresetNotFound {
						kind: PresetKind::GithubWorkflowStep,
						name: id.clone(),
					})?
					.clone();

				*step = StepPresetRef::Config(Box::new(data));
			}
		}
	}

	Ok(merged_preset)
}
