use crate::{cli::parsers::parse_key_value_pairs, *};
pub(crate) use ::gh_workflow::{GHStepData, Job, JobPreset, JobPresetRef, Step, Workflow};

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
#[merge(with = IndexMap::extend)]
#[serde(default)]
pub struct GithubConfig {
	/// A map of presets for Github workflows
	pub workflow_presets: IndexMap<String, GithubWorkflowPreset>,

	/// A map of presets for Github workflow jobs
	pub workflow_job_presets: IndexMap<String, JobPreset>,

	/// A map of presets for steps used in a Github workflow job.
	pub steps_presets: IndexMap<String, Step>,
}

/// A preset for a gihub workflow.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Merge)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct GithubWorkflowPreset {
	/// The list of extended presets.
	#[serde(default)]
	pub extends_presets: IndexSet<String>,

	#[serde(flatten)]
	pub config: Workflow,
}

impl Extensible for GithubWorkflowPreset {
	fn get_extended(&self) -> &IndexSet<String> {
		&self.extends_presets
	}
}

impl GithubWorkflowPreset {
	pub fn process_data(
		self,
		id: &str,
		github_config: &GithubConfig,
	) -> Result<Workflow, GenError> {
		if self.extends_presets.is_empty() {
			return Ok(self.config);
		}

		let mut processed_ids: IndexSet<String> = IndexSet::new();

		let merged_preset = merge_presets(
			Preset::GithubWorkflow,
			id,
			self,
			&github_config.workflow_presets,
			&mut processed_ids,
		)?;

		let mut config = merged_preset.config;

		for (_, job) in config.jobs.iter_mut() {
			match job {
				JobPresetRef::Preset(id) => {
					let data = github_config
						.workflow_job_presets
						.get(id)
						.ok_or(GenError::PresetNotFound {
							kind: Preset::GithubWorkflowJob,
							name: id.clone(),
						})?
						.clone();

					*job = JobPresetRef::Data(
						process_gh_job_preset(
							data,
							id,
							&github_config.workflow_job_presets,
							&github_config.steps_presets,
						)?
						.into(),
					);
				}
				JobPresetRef::Data(data) => {
					let data = mem::take(data);
					*job = JobPresetRef::Data(
						process_gh_job_preset(
							*data,
							"__inlined",
							&github_config.workflow_job_presets,
							&github_config.steps_presets,
						)?
						.into(),
					);
				}
			};
		}

		Ok(config)
	}
}

impl Extensible for JobPreset {
	fn get_extended(&self) -> &IndexSet<String> {
		&self.extends_presets
	}
}

pub fn process_gh_job_preset(
	preset: JobPreset,
	id: &str,
	store: &IndexMap<String, JobPreset>,
	steps_store: &IndexMap<String, Step>,
) -> Result<JobPreset, GenError> {
	let mut processed_ids: IndexSet<String> = IndexSet::new();

	let mut merged_preset = merge_presets(
		Preset::GithubWorkflowJob,
		id,
		preset,
		store,
		&mut processed_ids,
	)?;

	if let Job::Normal(job) = &mut merged_preset.job {
		for step in job.steps.iter_mut() {
			if let GHStepData::Preset(id) = step {
				let data = steps_store
					.get(id)
					.ok_or(GenError::PresetNotFound {
						kind: Preset::GithubWorkflowStep,
						name: id.clone(),
					})?
					.clone();

				*step = GHStepData::Config(Box::new(data));
			}
		}
	}

	Ok(merged_preset)
}
