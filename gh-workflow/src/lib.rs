use indexmap::{IndexMap, IndexSet};
use merge_it::*;
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

type JsonValueBTreeMap = BTreeMap<String, Value>;
type StringBTreeMap = BTreeMap<String, String>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
pub enum StringOrNum {
	String(String),
	Num(i64),
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
pub enum StringOrSortedList {
	String(String),
	List(BTreeSet<String>),
}

impl Merge for StringOrSortedList {
	fn merge(&mut self, right: Self) {
		match self {
			Self::String(left_string) => {
				if let Self::List(mut right_list) = right {
					right_list.insert(left_string.clone());
					*self = Self::List(right_list);
				} else {
					*self = right;
				}
			}
			Self::List(left_list) => match right {
				Self::String(right_string) => {
					left_list.insert(right_string);
				}
				Self::List(right_list) => {
					for item in right_list {
						left_list.insert(item);
					}
				}
			},
		}
	}
}

#[cfg(feature = "presets")]
pub use presets::*;

#[cfg(feature = "presets")]
mod presets {
	use super::*;

	/// Configuration for a Github workflow `step` or a preset id that points to one.
	#[derive(Clone, Deserialize, Debug, PartialEq, Eq, Serialize)]
	#[cfg_attr(feature = "schemars", derive(JsonSchema))]
	#[serde(untagged)]
	pub enum GHStepData {
		/// A preset ID
		#[serde(skip_serializing)]
		Preset(String),
		Config(Box<Step>),
	}

	impl GHStepData {
		pub fn as_config(self) -> Option<Step> {
			if let Self::Config(data) = self {
				Some(*data)
			} else {
				None
			}
		}
	}

	/// A workflow run is made up of one or more jobs. Jobs run in parallel by default. To run jobs sequentially, you can define dependencies on other jobs using the jobs.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/workflow-syntax-for-github-actions#jobs
	///
	/// You can use a job preset (by referring to it by its ID) or define a new one.
	#[derive(Clone, Deserialize, Debug, PartialEq, Eq, Serialize)]
	#[cfg_attr(feature = "schemars", derive(JsonSchema))]
	#[serde(untagged)]
	pub enum JobPresetRef {
		/// The preset ID for this job
		Preset(String),

		/// The definition for this job
		Data(Box<JobPreset>),
	}

	/// A preset for a gihub workflow job.
	#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Merge, Default)]
	#[cfg_attr(feature = "schemars", derive(JsonSchema))]
	pub struct JobPreset {
		/// The list of extended presets.
		#[serde(skip_serializing, default)]
		pub extends_presets: IndexSet<String>,

		#[serde(flatten)]
		pub job: Job,
	}
}

/// A github workflow.
///
/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax
#[derive(Clone, Deserialize, Debug, PartialEq, Eq, Serialize, Merge)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Workflow {
	/// The name of your workflow. GitHub displays the names of your workflows on your repository's actions page. If you omit this field, GitHub sets the name to the workflow's filename.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/workflow-syntax-for-github-actions#name
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub name: Option<String>,

	/// The name for workflow runs generated from the workflow. GitHub displays the workflow run name in the list of workflow runs on your repository's 'Actions' tab.
	///
	/// See more: https://docs.github.com/en/actions/using-workflows/workflow-syntax-for-github-actions#run-name
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub run_name: Option<String>,

	/// The name of the GitHub event that triggers the workflow. You can provide a single event string, array of events, array of event types, or an event configuration map that schedules a workflow or restricts the execution of a workflow to specific files, tags, or branch changes.
	///
	/// For a list of available events, see https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	#[merge(with = merge_options)]
	pub on: Option<Event>,

	/// You can modify the default permissions granted to the GITHUB_TOKEN, adding or removing access as required, so that you only allow the minimum required access.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflow-syntax-for-github-actions#permissions
	#[serde(default, skip_serializing_if = "Option::is_none")]
	#[merge(with = merge_options)]
	pub permissions: Option<Permissions>,

	/// A map of variables that are available to the steps of all jobs in the workflow. You can also set variables that are only available to the steps of a single job or to a single step.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#env
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	pub env: BTreeMap<String, StringNumOrBool>,

	/// Use `defaults` to create a map of default settings that will apply to all jobs in the workflow. You can also set default settings that are only available to a job.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#defaults
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub defaults: Option<Defaults>,

	/// Concurrency ensures that only a single job or workflow using the same concurrency group will run at a time. A concurrency group can be any string or expression. The expression can use any context except for the secrets context.
	///
	/// You can also specify concurrency at the workflow level.
	///
	/// When a concurrent job or workflow is queued, if another job or workflow using the same concurrency group in the repository is in progress, the queued job or workflow will be pending. Any previously pending job or workflow in the concurrency group will be canceled. To also cancel any currently running job or workflow in the same concurrency group, specify cancel-in-progress: true.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflow-syntax-for-github-actions#concurrency
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub concurrency: Option<Concurrency>,

	/// A workflow run is made up of one or more jobs. Jobs run in parallel by default. To run jobs sequentially, you can define dependencies on other jobs using the jobs.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/workflow-syntax-for-github-actions#jobs
	#[serde(default)]
	#[cfg(feature = "presets")]
	pub jobs: IndexMap<String, JobPresetRef>,
	#[cfg(not(feature = "presets"))]
	pub jobs: IndexMap<String, Job>,
}

#[allow(clippy::large_enum_variant)]
/// A workflow run is made up of one or more jobs. Jobs run in parallel by default. To run jobs sequentially, you can define dependencies on other jobs using the jobs.
///
/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/workflow-syntax-for-github-actions#jobs
#[derive(Clone, Deserialize, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
pub enum Job {
	/// The definition for a new job.
	Normal(Box<NormalJob>),

	/// A reusable job, imported from a file or repo.
	Reusable(ReusableWorkflowCallJob),
}

impl Default for Job {
	fn default() -> Self {
		Self::Normal(NormalJob::default().into())
	}
}

impl Merge for Job {
	fn merge(&mut self, other: Self) {
		match self {
			Self::Normal(left_job) => match other {
				Self::Normal(right_job) => {
					left_job.merge(*right_job);
				}
				Self::Reusable(right_job) => {
					left_job.name.merge(right_job.name);
					merge_options(&mut left_job.needs, right_job.needs);
					left_job.if_.merge(right_job.if_);
					left_job.concurrency.merge(right_job.concurrency);
					merge_options(&mut left_job.permissions, right_job.permissions);
				}
			},
			Self::Reusable(left_job) => match other {
				Self::Reusable(right_job) => {
					left_job.merge(right_job);
				}
				Self::Normal(right_job) => {
					left_job.name.merge(right_job.name);
					merge_options(&mut left_job.needs, right_job.needs);
					left_job.if_.merge(right_job.if_);
					left_job.concurrency.merge(right_job.concurrency);
					merge_options(&mut left_job.permissions, right_job.permissions);
				}
			},
		}
	}
}

/// A workflow run is made up of one or more jobs, which run in parallel by default.
///
/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobs
#[derive(Clone, Deserialize, Debug, PartialEq, Eq, Serialize, Merge, Default)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct NormalJob {
	/// The type of machine to run the job on. The machine can be either a GitHub-hosted runner, or a self-hosted runner. Can be a single item, a list, or a group configuration.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_idruns-on
	#[serde(rename = "runs-on", default, skip_serializing_if = "Option::is_none")]
	#[merge(with = merge_options)]
	pub runs_on: Option<RunsOn>,

	/// The name of the job displayed on GitHub.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/workflow-syntax-for-github-actions#jobsjob_idname
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub name: Option<String>,

	/// Identifies any jobs that must complete successfully before this job will run. It can be a string or array of strings. If a job fails, all jobs that need it are skipped unless the jobs use a conditional statement that causes the job to continue.
	///
	/// Can be a string or a list of strings.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/workflow-syntax-for-github-actions#jobsjob_idneeds
	#[serde(default, skip_serializing_if = "Option::is_none")]
	#[merge(with = merge_options)]
	pub needs: Option<StringOrSortedList>,

	/// You can use the if conditional to prevent a job from running unless a condition is met. You can use any supported context and expression to create a conditional.
	///
	/// Expressions in an if conditional do not require the ${{ }} syntax. For more information, see https://help.github.com/en/articles/contexts-and-expression-syntax-for-github-actions.
	///
	/// See more: https://help.github.com/en/actions/automating-your-workflow-with-github-actions/workflow-syntax-for-github-actions#jobsjob_idif
	#[serde(default, rename = "if", skip_serializing_if = "Option::is_none")]
	pub if_: Option<String>,

	/// A map of outputs for a job. Job outputs are available to all downstream jobs that depend on this job.
	///
	/// See more: https://help.github.com/en/actions/reference/workflow-syntax-for-github-actions#jobsjob_idoutputs
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	pub outputs: StringBTreeMap,

	/// A map of default settings that will apply to all steps in the job.
	///
	/// See more: https://help.github.com/en/actions/reference/workflow-syntax-for-github-actions#jobsjob_iddefaults
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub defaults: Option<Defaults>,

	/// For a specific job, you can use jobs.<job_id>.permissions to modify the default permissions granted to the GITHUB_TOKEN, adding or removing access as required, so that you only allow the minimum required access.
	///
	/// Permissions can be defined globally, with `write-all` `or read-all`, or by event.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_idpermissions
	#[serde(default, skip_serializing_if = "Option::is_none")]
	#[merge(with = merge_options)]
	pub permissions: Option<Permissions>,

	/// Concurrency ensures that only a single job or workflow using the same concurrency group will run at a time. A concurrency group can be any string or expression.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflow-syntax-for-github-actions#jobsjob_idconcurrency
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub concurrency: Option<Concurrency>,

	/// The environment that the job references. You can provide the environment as only the environment name, or as an environment object with the `name` and `url`.
	///
	/// See more: https://docs.github.com/en/free-pro-team@latest/actions/reference/workflow-syntax-for-github-actions#jobsjob_idenvironment
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub environment: Option<Environment>,

	/// A map of environment variables that are available to all steps in the job.
	///
	/// See more: https://help.github.com/en/actions/automating-your-workflow-with-github-actions/workflow-syntax-for-github-actions#jobsjob_idenv
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	pub env: BTreeMap<String, StringNumOrBool>,

	/// The maximum number of minutes to let a workflow run before GitHub automatically cancels it. Default: 360
	///
	/// See more: https://help.github.com/en/actions/automating-your-workflow-with-github-actions/workflow-syntax-for-github-actions#jobsjob_idtimeout-minutes
	#[serde(
		default,
		rename = "timeout-minutes",
		skip_serializing_if = "Option::is_none"
	)]
	pub timeout_minutes: Option<StringOrNum>,

	/// Prevents a workflow run from failing when a job fails. Set to true to allow a workflow run to pass when this job fails.
	///
	/// See more: https://help.github.com/en/actions/reference/workflow-syntax-for-github-actions#jobsjob_idcontinue-on-error
	#[serde(
		default,
		rename = "continue-on-error",
		skip_serializing_if = "Option::is_none"
	)]
	pub continue_on_error: Option<StringOrBool>,

	/// A container to run any steps in a job that don't already specify a container. If you have steps that use both script and container actions, the container actions will run as sibling containers on the same network with the same volume mounts.
	///
	/// See more: https://help.github.com/en/actions/automating-your-workflow-with-github-actions/workflow-syntax-for-github-actions#jobsjob_idcontainer
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub container: Option<Container>,

	/// Additional containers to host services for a job in a workflow. These are useful for creating databases or cache services like redis. The runner on the virtual machine will automatically create a network and manage the life cycle of the service containers.
	///
	/// See more: https://help.github.com/en/actions/automating-your-workflow-with-github-actions/workflow-syntax-for-github-actions#jobsjob_idservices
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	pub services: BTreeMap<String, Container>,

	/// A strategy creates a build matrix for your jobs. You can define different variations of an environment to run each job in.
	///
	/// See more: https://help.github.com/en/actions/automating-your-workflow-with-github-actions/workflow-syntax-for-github-actions#jobsjob_idstrategy
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub strategy: Option<Strategy>,

	/// A job contains a sequence of tasks called steps. Steps can run commands, run setup tasks, or run an action in your repository, a public repository, or an action published in a Docker registry.
	///
	/// See more: https://help.github.com/en/actions/automating-your-workflow-with-github-actions/workflow-syntax-for-github-actions#jobsjob_idsteps
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	#[cfg(feature = "presets")]
	pub steps: Vec<GHStepData>,
	#[cfg(not(feature = "presets"))]
	pub steps: Vec<Step>,
}

/// A reusable job, imported from a file or repo.
#[derive(Clone, Deserialize, Debug, PartialEq, Eq, Serialize, Merge, Default)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct ReusableWorkflowCallJob {
	/// The location and version of a reusable workflow file to run as a job, of the form './{path/to}/{localfile}.yml' or '{owner}/{repo}/{path}/{filename}@{ref}'. {ref} can be a SHA, a release tag, or a branch name. Using the commit SHA is the safest for stability and security.
	///
	/// See more: https://docs.github.com/en/actions/learn-github-actions/workflow-syntax-for-github-actions#jobsjob_iduses
	#[merge(skip)]
	pub uses: String,

	/// The name of the job displayed on GitHub.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/workflow-syntax-for-github-actions#jobsjob_idname
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub name: Option<String>,

	/// Identifies any jobs that must complete successfully before this job will run. It can be a string or array of strings. If a job fails, all jobs that need it are skipped unless the jobs use a conditional statement that causes the job to continue.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/workflow-syntax-for-github-actions#jobsjob_idneeds
	#[serde(default, skip_serializing_if = "Option::is_none")]
	#[merge(with = merge_options)]
	pub needs: Option<StringOrSortedList>,

	/// You can use the if conditional to prevent a job from running unless a condition is met. You can use any supported context and expression to create a conditional.
	///
	/// Expressions in an if conditional do not require the ${{ }} syntax. For more information, see https://help.github.com/en/articles/contexts-and-expression-syntax-for-github-actions.
	///
	/// See more: https://help.github.com/en/actions/automating-your-workflow-with-github-actions/workflow-syntax-for-github-actions#jobsjob_idif
	#[serde(default, rename = "if", skip_serializing_if = "Option::is_none")]
	pub if_: Option<String>,

	/// Concurrency ensures that only a single job or workflow using the same concurrency group will run at a time. A concurrency group can be any string or expression. The expression can use any context except for the secrets context.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflow-syntax-for-github-actions#jobsjob_idconcurrency
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub concurrency: Option<Concurrency>,

	/// For a specific job, you can use jobs.<job_id>.permissions to modify the default permissions granted to the GITHUB_TOKEN, adding or removing access as required, so that you only allow the minimum required access.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_idpermissions
	#[serde(default, skip_serializing_if = "Option::is_none")]
	#[merge(with = merge_options)]
	pub permissions: Option<Permissions>,

	/// A map of inputs that are passed to the called workflow. Any inputs that you pass must match the input specifications defined in the called workflow. Unlike 'jobs.<job_id>.steps[*].with', the inputs you pass with 'jobs.<job_id>.with' are not be available as environment variables in the called workflow. Instead, you can reference the inputs by using the inputs context.
	///
	/// See more: https://docs.github.com/en/actions/learn-github-actions/workflow-syntax-for-github-actions#jobsjob_idwith
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	pub env: BTreeMap<String, StringNumOrBool>,

	/// When a job is used to call a reusable workflow, you can use 'secrets' to provide a map of secrets that are passed to the called workflow. Any secrets that you pass must match the names defined in the called workflow.
	///
	/// See more: https://docs.github.com/en/actions/learn-github-actions/workflow-syntax-for-github-actions#jobsjob_idsecrets
	#[serde(default, skip_serializing_if = "Option::is_none")]
	#[merge(with = merge_options)]
	pub secrets: Option<JobSecret>,

	/// A build matrix is a set of different configurations of the virtual environment.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_idstrategymatrix
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub strategy: Option<Strategy>,
}

/// When a job is used to call a reusable workflow, you can use 'secrets' to provide a map of secrets that are passed to the called workflow.
///
/// Any secrets that you pass must match the names defined in the called workflow.
///
/// See more: https://docs.github.com/en/actions/learn-github-actions/workflow-syntax-for-github-actions#jobsjob_idsecrets
#[derive(Clone, Deserialize, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum JobSecret {
	/// Use the `inherit` keyword to pass all the calling workflow's secrets to the called workflow. This includes all secrets the calling workflow has access to, namely organization, repository, and environment secrets. The `inherit` keyword can be used to pass secrets across repositories within the same organization, or across organizations within the same enterprise.
	#[serde(rename = "inherit")]
	Inherit,

	/// A map of secrets that are passed to the called workflow.
	#[serde(untagged)]
	Object(BTreeMap<String, StringNumOrBool>),
}

impl Merge for JobSecret {
	fn merge(&mut self, other: Self) {
		if let Self::Object(left_map) = self
			&& let Self::Object(right_map) = other
		{
			for (key, val) in right_map {
				left_map.insert(key, val);
			}
		} else {
			*self = other;
		}
	}
}

/// A container to run any steps in a job that don't already specify a container.
///
/// If you have steps that use both script and container actions, the container actions will run as sibling containers on the same network with the same volume mounts.
///
/// See more: https://help.github.com/en/actions/automating-your-workflow-with-github-actions/workflow-syntax-for-github-actions#jobsjob_idcontainer
#[derive(Clone, Deserialize, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Container {
	/// The Docker image to use as the container to run the action. The value can be the Docker Hub image name or a registry name.
	///
	/// See more: https://help.github.com/en/actions/automating-your-workflow-with-github-actions/workflow-syntax-for-github-actions#jobsjob_idcontainerimage
	pub image: String,

	/// If the image's container registry requires authentication to pull the image, you can use credentials to set a map of the username and password. The credentials are the same values that you would provide to the `docker login` command.
	///
	/// See more: https://docs.github.com/en/free-pro-team@latest/actions/reference/workflow-syntax-for-github-actions#jobsjob_idcontainercredentials
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub credentials: Option<Credentials>,

	/// Sets an array of environment variables in the container.
	///
	/// See more: https://help.github.com/en/actions/automating-your-workflow-with-github-actions/workflow-syntax-for-github-actions#jobsjob_idcontainerenv
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	pub env: BTreeMap<String, StringNumOrBool>,

	/// Sets an array of ports to expose on the container.
	///
	/// See more: https://help.github.com/en/actions/automating-your-workflow-with-github-actions/workflow-syntax-for-github-actions#jobsjob_idcontainerports
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub ports: Option<BTreeSet<StringOrNum>>,

	/// Sets an array of volumes for the container to use. You can use volumes to share data between services or other steps in a job. You can specify named Docker volumes, anonymous Docker volumes, or bind mounts on the host.
	///
	/// See more: https://help.github.com/en/actions/automating-your-workflow-with-github-actions/workflow-syntax-for-github-actions#jobsjob_idcontainervolumes
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub volumes: Option<BTreeSet<String>>,

	/// Additional Docker container resource options. For a list of options, see https://docs.docker.com/engine/reference/commandline/create/#options.
	///
	/// See more: https://help.github.com/en/actions/automating-your-workflow-with-github-actions/workflow-syntax-for-github-actions#jobsjob_idcontaineroptions
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub options: Option<String>,
}

/// If the image's container registry requires authentication to pull the image, you can use credentials to set a map of the username and password.
///
/// The credentials are the same values that you would provide to the `docker login` command.
///
/// See more: https://docs.github.com/en/free-pro-team@latest/actions/reference/workflow-syntax-for-github-actions#jobsjob_idcontainercredentials
#[derive(Clone, Deserialize, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Credentials {
	pub username: String,
	pub password: String,
}

/// A strategy creates a build matrix for your jobs. You can define different variations of an environment to run each job in.
///
/// See more: https://help.github.com/en/actions/automating-your-workflow-with-github-actions/workflow-syntax-for-github-actions#jobsjob_idstrategy
#[derive(Clone, Deserialize, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Strategy {
	/// When set to true, GitHub cancels all in-progress jobs if any matrix job fails. Default: true
	///
	/// See more: https://help.github.com/en/actions/automating-your-workflow-with-github-actions/workflow-syntax-for-github-actions#jobsjob_idstrategyfail-fast
	#[serde(default, rename = "fail-fast", skip_serializing_if = "Option::is_none")]
	pub fail_fast: Option<StringOrBool>,

	/// The maximum number of jobs that can run simultaneously when using a matrix job strategy. By default, GitHub will maximize the number of jobs run in parallel depending on the available runners on GitHub-hosted virtual machines.
	///
	/// See more: https://help.github.com/en/actions/automating-your-workflow-with-github-actions/workflow-syntax-for-github-actions#jobsjob_idstrategymax-parallel
	#[serde(
		default,
		rename = "max-parallel",
		skip_serializing_if = "Option::is_none"
	)]
	pub max_parallel: Option<StringOrNum>,

	/// A build matrix is a set of different configurations of the virtual environment.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_idstrategymatrix
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	pub matrix: JsonValueBTreeMap,
}

/// A job contains a sequence of tasks called `steps`. Steps can run commands, run setup tasks, or run an action in your repository, a public repository, or an action published in a Docker registry.
///
/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_idsteps
#[derive(Clone, Deserialize, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Step {
	/// A name for your step to display on GitHub.
	///
	/// See more: https://help.github.com/en/actions/automating-your-workflow-with-github-actions/workflow-syntax-for-github-actions#jobsjob_idstepsname
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub name: Option<String>,

	/// A unique identifier for the step. You can use the id to reference the step in contexts. For more information, see https://help.github.com/en/articles/contexts-and-expression-syntax-for-github-actions.
	///
	/// See more: https://help.github.com/en/actions/automating-your-workflow-with-github-actions/workflow-syntax-for-github-actions#jobsjob_idstepsid
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub id: Option<String>,

	/// You can use the if conditional to prevent a step from running unless a condition is met. You can use any supported context and expression to create a conditional.
	///
	/// Expressions in an if conditional do not require the ${{ }} syntax. For more information, see https://help.github.com/en/articles/contexts-and-expression-syntax-for-github-actions.
	///
	/// See more: https://help.github.com/en/actions/automating-your-workflow-with-github-actions/workflow-syntax-for-github-actions#jobsjob_idstepsif
	#[serde(default, rename = "if", skip_serializing_if = "Option::is_none")]
	pub if_: Option<String>,

	/// Selects an action to run as part of a step in your job. An action is a reusable unit of code. You can use an action defined in the same repository as the workflow, a public repository, or in a published Docker container image.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_idstepsuses
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub uses: Option<String>,

	/// Prevents a job from failing when a step fails. Set to true to allow a job to pass when this step fails.
	///
	/// See more: https://help.github.com/en/actions/automating-your-workflow-with-github-actions/workflow-syntax-for-github-actions#jobsjob_idstepscontinue-on-error
	#[serde(
		default,
		rename = "continue-on-error",
		skip_serializing_if = "Option::is_none"
	)]
	pub continue_on_error: Option<StringOrBool>,

	/// The maximum number of minutes to run the step before killing the process.
	///
	/// See more: https://help.github.com/en/actions/automating-your-workflow-with-github-actions/workflow-syntax-for-github-actions#jobsjob_idstepstimeout-minutes
	#[serde(
		default,
		rename = "timeout-minutes",
		skip_serializing_if = "Option::is_none"
	)]
	pub timeout_minutes: Option<StringOrNum>,

	/// A map of the input parameters defined by the action. Each input parameter is a key/value pair. Input parameters are set as environment variables. The variable is prefixed with INPUT_ and converted to upper case.
	///
	/// See more: https://help.github.com/en/actions/automating-your-workflow-with-github-actions/workflow-syntax-for-github-actions#jobsjob_idstepswith
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub with: Option<BTreeMap<String, StringNumOrBool>>,

	/// Sets environment variables for steps to use in the virtual environment. You can also set environment variables for the entire workflow or a job.
	///
	/// See more: https://help.github.com/en/actions/automating-your-workflow-with-github-actions/workflow-syntax-for-github-actions#jobsjob_idstepsenv
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	pub env: BTreeMap<String, StringNumOrBool>,

	/// Use `shell` to define the shell for a step.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_iddefaultsrunshell
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub shell: Option<Shell>,

	/// Using the `working-directory` keyword, you can specify the working directory of where to run the command.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_idstepsworking-directory
	#[serde(
		default,
		rename = "working-directory",
		skip_serializing_if = "Option::is_none"
	)]
	pub working_directory: Option<String>,

	/// Runs command-line programs that do not exceed 21,000 characters using the operating system's shell. If you do not provide a `name`, the step name will default to the text specified in the `run` command.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_idstepsrun
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub run: Option<String>,
}

/// The environment that the job references. You can provide the environment as only the environment name, or as an environment object with the `name` and `url`.
///
/// See more: https://docs.github.com/en/free-pro-team@latest/actions/reference/workflow-syntax-for-github-actions#jobsjob_idenvironment
#[derive(Clone, Deserialize, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
pub enum Environment {
	/// The environment name.
	String(String),

	/// An environment object with the `name` and `url`.
	Object {
		/// The name of the environment configured in the repo.
		///
		/// See more: https://docs.github.com/en/free-pro-team@latest/actions/reference/workflow-syntax-for-github-actions#example-using-a-single-environment-name
		name: String,

		/// A deployment URL.
		///
		/// See more: https://docs.github.com/en/free-pro-team@latest/actions/reference/workflow-syntax-for-github-actions#example-using-environment-name-and-url
		#[serde(default, skip_serializing_if = "Option::is_none")]
		url: Option<String>,
	},
}

/// The type of machine to run the job on. The machine can be either a GitHub-hosted runner, or a self-hosted runner. Can be a single item, a list, or a group configuration.
///
/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_idruns-on
#[derive(Clone, Deserialize, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
pub enum RunsOn {
	Single(ActionRunner),
	List(BTreeSet<ActionRunner>),

	/// You can use `runs-on` to target runner groups, so that the job will execute on any runner that is a member of that group. For more granular control, you can also combine runner groups with labels.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#choosing-runners-in-a-group
	Group {
		/// You can use `runs-on` to target runner groups, so that the job will execute on any runner that is a member of that group. For more granular control, you can also combine runner groups with labels.
		///
		/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#choosing-runners-in-a-group
		group: String,

		/// When you combine groups and labels, the runner must meet both requirements to be eligible to run the job.
		///
		/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#example-combining-groups-and-labels
		#[serde(default, skip_serializing_if = "Option::is_none")]
		labels: Option<StringOrSortedList>,
	},
}

/// The type of machine to run the job on. The machine can be either a GitHub-hosted runner, or a self-hosted runner.
///
/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_idruns-on
#[allow(non_camel_case_types)]
#[derive(Clone, Deserialize, Debug, PartialEq, Serialize, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum ActionRunner {
	/// A runner with the latest LTS version of Ubuntu.
	///
	/// See more: https://github.com/actions/runner-images/blob/main/images/ubuntu
	UbuntuLatest,

	/// See more: https://github.com/actions/runner-images/blob/main/images/ubuntu/Ubuntu2404-Readme.md
	#[serde(rename = "ubuntu-24.04")]
	Ubuntu24_04,

	/// See more: https://github.com/actions/partner-runner-images/blob/main/images/arm-ubuntu-24-image.md
	#[serde(rename = "ubuntu-24.04-arm")]
	Ubuntu24_04_Arm,

	/// See more: https://github.com/actions/runner-images/blob/main/images/ubuntu/Ubuntu2204-Readme.md
	#[serde(rename = "ubuntu-22.04")]
	Ubuntu22_04,

	/// See more: https://github.com/actions/partner-runner-images/blob/main/images/arm-ubuntu-22-image.md
	#[serde(rename = "ubuntu-22.04-arm")]
	Ubuntu22_04_Arm,

	/// A runner with the latest version of Windows.
	///
	/// See more: https://github.com/actions/runner-images/blob/main/images/windows
	WindowsLatest,

	/// See more: https://github.com/actions/runner-images/blob/main/images/windows/Windows2025-Readme.md
	Windows2025,

	/// See more: https://github.com/actions/runner-images/blob/main/images/windows/Windows2022-Readme.md
	Windows2022,

	/// See more: https://github.com/actions/partner-runner-images/blob/main/images/arm-windows-11-image.md
	#[serde(rename = "windows-11-arm")]
	Windows11_Arm,

	/// A runner with the latest version of MacOS.
	///
	/// See more: https://github.com/actions/runner-images/blob/main/images/macos
	#[serde(rename = "macos-latest")]
	MacOsLatest,

	/// See more: https://github.com/actions/runner-images/blob/main/images/macos/macos-26-arm64-Readme.md
	#[serde(rename = "macos-26")]
	MacOs26,

	/// See more: https://github.com/actions/runner-images/blob/main/images/macos/macos-15-arm64-Readme.md
	#[serde(rename = "macos-15")]
	MacOs15,

	/// See more: https://github.com/actions/runner-images/blob/main/images/macos/macos-15-Readme.md
	#[serde(rename = "macos-15-intel")]
	MacOs15_Intel,

	/// See more: https://github.com/actions/runner-images/blob/main/images/macos/macos-14-arm64-Readme.md
	#[serde(rename = "macos-14")]
	MacOs14,

	/// See more: https://github.com/actions/runner-images/blob/main/images/macos/macos-13-Readme.md
	#[serde(rename = "macos-13")]
	MacOs13,

	#[serde(untagged)]
	Other(String),
}

impl Merge for RunsOn {
	fn merge(&mut self, right: Self) {
		match self {
			Self::Single(left_string) => {
				if let Self::List(mut right_list) = right {
					right_list.insert(left_string.clone());
					*self = Self::List(right_list);
				} else {
					*self = right;
				}
			}
			Self::List(left_list) => match right {
				Self::Single(right_string) => {
					left_list.insert(right_string);
				}
				Self::List(right_list) => {
					left_list.extend(right_list);
				}
				_ => {
					*self = right;
				}
			},
			Self::Group {
				labels: left_labels,
				group: left_group,
			} => {
				if let Self::Group {
					labels: right_labels,
					group: right_group,
				} = right
				{
					*left_group = right_group;

					if let Some(right_labels) = right_labels {
						if let Some(left_labels) = left_labels {
							left_labels.merge(right_labels);
						} else {
							*left_labels = Some(right_labels);
						}
					}
				} else {
					*self = right;
				}
			}
		}
	}
}

/// You can modify the default permissions granted to the GITHUB_TOKEN, adding or removing access as required, so that you only allow the minimum required access.
///
/// Permissions can be defined globally, with `write-all` `or read-all`, or by event.
///
/// See more: https://docs.github.com/en/actions/reference/workflow-syntax-for-github-actions#permissions
#[derive(Clone, Deserialize, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
pub enum Permissions {
	Global(PermissionsGlobal),
	Event(PermissionsEvent),
}

impl Merge for Permissions {
	fn merge(&mut self, right: Self) {
		match self {
			Self::Global(_) => {
				*self = right;
			}
			Self::Event(obj_left) => {
				if let Self::Event(obj_right) = right {
					obj_left.merge(obj_right);
				} else {
					*self = right;
				}
			}
		}
	}
}

#[derive(Clone, Deserialize, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum PermissionsGlobal {
	ReadAll,
	WriteAll,
}

#[derive(Clone, Deserialize, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum PermissionsLevel {
	None,
	Read,
	Write,
}

/// For each of the available permissions, you can assign one of the access levels: read (if applicable), write, or none.
///
/// write includes read. If you specify the access for any of these permissions, all of those that are not specified are set to none.
///
/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#permissions
#[derive(Clone, Deserialize, Debug, PartialEq, Eq, Serialize, Merge)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct PermissionsEvent {
	/// Work with GitHub Actions. For example, `actions: write` permits an action to cancel a workflow run.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_idpermissions
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub actions: Option<PermissionsLevel>,

	/// Work with artifact attestations. For example, `attestations: write` permits an action to generate an artifact attestation for a build.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_idpermissions
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub attestations: Option<PermissionsLevel>,

	/// Work with check runs and check suites. For example, `checks: write` permits an action to create a check run.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_idpermissions
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub checks: Option<PermissionsLevel>,

	/// Work with the contents of the repository. For example, `contents: read` permits an action to list the commits, and `contents: write` allows the action to create a release.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_idpermissions
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub contents: Option<PermissionsLevel>,

	/// Work with deployments. For example, `deployments: write` permits an action to create a new deployment.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_idpermissions
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub deployments: Option<PermissionsLevel>,

	/// Work with GitHub Discussions. For example, `discussions: write` permits an action to close or delete a discussion.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_idpermissions
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub discussions: Option<PermissionsLevel>,

	/// Fetch an OpenID Connect (OIDC) token. This requires `id-token: write`.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_idpermissions
	#[serde(default, rename = "id-token", skip_serializing_if = "Option::is_none")]
	pub id_token: Option<PermissionsLevel>,

	/// Work with issues. For example, `issues: write` permits an action to add a comment to an issue.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_idpermissions
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub issues: Option<PermissionsLevel>,

	/// Generate AI inference responses with GitHub Models. For example, `models: read` permits an action to use the GitHub Models inference API.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_idpermissions
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub models: Option<ModelsPermissions>,

	/// Work with GitHub Packages. For example, `packages: write` permits an action to upload and publish packages on GitHub Packages.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_idpermissions
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub packages: Option<PermissionsLevel>,

	/// Work with GitHub Pages. For example, `pages: write` permits an action to request a GitHub Pages build.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_idpermissions
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub pages: Option<PermissionsLevel>,

	/// Work with pull requests. For example, `pull-requests: write` permits an action to add a label to a pull request.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_idpermissions
	#[serde(
		default,
		rename = "pull-requests",
		skip_serializing_if = "Option::is_none"
	)]
	pub pull_requests: Option<PermissionsLevel>,

	#[serde(
		default,
		rename = "repository-projects",
		skip_serializing_if = "Option::is_none"
	)]
	pub repository_projects: Option<PermissionsLevel>,

	/// Work with GitHub code scanning alerts. For example, `security-events: read` permits an action to list the code scanning alerts for the repository, and `security-events: write` allows an action to update the status of a code scanning alert.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_idpermissions
	#[serde(
		default,
		rename = "security-events",
		skip_serializing_if = "Option::is_none"
	)]
	pub security_events: Option<PermissionsLevel>,

	/// Work with commit statuses. For example, `statuses:read` permits an action to list the commit statuses for a given reference.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_idpermissions
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub statuses: Option<PermissionsLevel>,
}

#[derive(Clone, Deserialize, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum ModelsPermissions {
	None,
	Read,
}

/// Concurrency ensures that only a single job or workflow using the same concurrency group will run at a time.
///
/// A concurrency group can be any string or expression. The expression can use any context except for the secrets context.
///
/// You can also specify concurrency at the workflow level.
///
/// When a concurrent job or workflow is queued, if another job or workflow using the same concurrency group in the repository is in progress, the queued job or workflow will be pending. Any previously pending job or workflow in the concurrency group will be canceled. To also cancel any currently running job or workflow in the same concurrency group, specify cancel-in-progress: true.
///
/// See more: https://docs.github.com/en/actions/reference/workflow-syntax-for-github-actions#concurrency
#[derive(Clone, Deserialize, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Concurrency {
	/// When a concurrent job or workflow is queued, if another job or workflow using the same concurrency group in the repository is in progress, the queued job or workflow will be pending. Any previously pending job or workflow in the concurrency group will be canceled.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflow-syntax-for-github-actions#example-using-concurrency-to-cancel-any-in-progress-job-or-run-1
	pub group: String,

	/// To cancel any currently running job or workflow in the same concurrency group, specify cancel-in-progress: true.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflow-syntax-for-github-actions#example-using-concurrency-to-cancel-any-in-progress-job-or-run-1
	#[serde(
		default,
		rename = "cancel-in-progress",
		skip_serializing_if = "Option::is_none"
	)]
	pub cancel_in_progress: Option<StringOrBool>,
}

#[derive(Clone, Deserialize, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
pub enum StringOrBool {
	String(String),
	Bool(bool),
}

/// Use `defaults` to create a map of default settings that will apply to all jobs in the workflow. You can also set default settings that are only available to a job.
///
/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#defaults
#[derive(Clone, Deserialize, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Defaults {
	/// You can use `defaults.run` to provide default `shell` and `working-directory` options for all `run` steps in a workflow. You can also set default settings for run that are only available to a job.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#defaultsrun
	pub run: RunObject,
}

/// You can use `defaults.run` to provide default `shell` and `working-directory` options for all `run` steps in a workflow. You can also set default settings for run that are only available to a job.
///
/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#defaultsrun
#[derive(Clone, Deserialize, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct RunObject {
	/// Use `shell` to `define` the shell for a step.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#defaultsrunshell
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub shell: Option<Shell>,

	/// Use `working-directory` to define the working directory for the shell for a step. This keyword can reference several contexts.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#defaultsrunworking-directory
	#[serde(rename = "working-directory")]
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub working_directory: Option<String>,
}

/// You can override the default shell settings in the runner's operating system using the shell keyword. You can use built-in shell keywords, or you can define a custom set of shell options.
///
/// See more: https://docs.github.com/en/actions/using-workflows/workflow-syntax-for-github-actions#jobsjob_idstepsshell
#[derive(Clone, Deserialize, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum Shell {
	Bash,
	Cmd,
	Powershell,
	Python,
	Pwsh,
	Sh,
	#[serde(untagged)]
	Other(String),
}

/// The name of the GitHub event that triggers the workflow.
///
/// You can provide a single event string, array of events, array of event types, or an event configuration map that schedules a workflow or restricts the execution of a workflow to specific files, tags, or branch changes.
///
/// For a list of available events, see https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows.
#[derive(Clone, Deserialize, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
pub enum Event {
	Single(EventDefinition),
	Multiple(BTreeSet<EventDefinition>),
	Object(Box<EventObject>),
}

impl Merge for Event {
	fn merge(&mut self, other: Self) {
		match self {
			Self::Single(_) => {
				*self = other;
			}

			Self::Multiple(left_list) => {
				if let Self::Multiple(right_list) = other {
					left_list.extend(right_list);
				} else {
					*self = other;
				}
			}
			Self::Object(left_object) => {
				if let Self::Object(right_object) = other {
					left_object.merge(right_object);
				} else {
					*self = other;
				}
			}
		}
	}
}

/// Workflow triggers are events that cause a workflow to run.
///
/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#about-events-that-trigger-workflows
#[derive(Clone, Copy, Deserialize, Debug, PartialEq, Eq, Serialize, PartialOrd, Ord)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum EventDefinition {
	/// Runs your workflow anytime the branch_protection_rule event occurs
	///
	/// See more: https://docs.github.com/en/actions/learn-github-actions/events-that-trigger-workflows#branch_protection_rule
	BranchProtectionRule,

	/// Runs your workflow anytime the check_run event occurs
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#check-run-event-check_run
	CheckRun,

	/// Runs your workflow anytime the check_suite event occurs.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#check-suite-event-check_suite
	CheckSuite,

	/// Runs your workflow anytime someone creates a branch or tag, which triggers the create event.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#create-event-create
	Create,

	/// Runs your workflow anytime someone deletes a branch or tag, which triggers the delete event.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#delete-event-delete
	Delete,

	/// Runs your workflow anytime someone creates a deployment, which triggers the deployment event. Deployments created with a commit SHA may not have a Git ref.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#deployment-event-deployment
	Deployment,

	/// Runs your workflow anytime a third party provides a deployment status, which triggers the deployment_status event. Deployments created with a commit SHA may not have a Git ref.
	///
	/// See more: https://docs.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows
	DeploymentStatus,

	/// Runs your workflow anytime the discussion event occurs. More than one activity type triggers this event.
	///
	/// See more: https://docs.github.com/en/actions/reference/events-that-trigger-workflows#discussion
	Discussion,

	/// Runs your workflow anytime the discussion_comment event occurs. More than one activity type triggers this event.
	///
	/// See more: https://docs.github.com/en/actions/reference/events-that-trigger-workflows#discussion_comment
	DiscussionComment,

	/// Runs your workflow anytime when someone forks a repository, which triggers the fork event.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#fork-event-fork
	Fork,

	/// Runs your workflow when someone creates or updates a Wiki page, which triggers the gollum event.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#gollum-event-gollum
	Gollum,

	/// Runs your workflow anytime the issue_comment event occurs. More than one activity type triggers this event.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#issue-comment-event-issue_comment
	IssueComment,

	/// Runs your workflow anytime the issues event occurs. More than one activity type triggers this event.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#issues-event-issues
	Issues,

	/// Runs your workflow anytime the label event occurs. More than one activity type triggers this event.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#label-event-label
	Label,

	/// Runs your workflow when a pull request is added to a merge queue, which adds the pull request to a merge group. For information about the merge queue, see https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/incorporating-changes-from-a-pull-request/merging-a-pull-request-with-a-merge-queue.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#merge_group
	MergeGroup,

	/// Runs your workflow anytime the milestone event occurs. More than one activity type triggers this event.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#milestone-event-milestone
	Milestone,

	/// Runs your workflow anytime someone pushes to a GitHub Pages-enabled branch, which triggers the page_build event.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#page-build-event-page_build
	PageBuild,

	/// Runs your workflow anytime the project event occurs. More than one activity type triggers this event.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#project-event-project
	Project,

	/// Runs your workflow anytime the project_card event occurs. More than one activity type triggers this event.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#project-card-event-project_card
	ProjectCard,

	/// Runs your workflow anytime the project_column event occurs. More than one activity type triggers this event.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#project-column-event-project_column
	ProjectColumn,

	/// Runs your workflow anytime someone makes a private repository public, which triggers the public event.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#public-event-public
	Public,

	/// Runs your workflow anytime the pull_request event occurs. More than one activity type triggers this event.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#pull-request-event-pull_request
	PullRequest,

	/// Runs your workflow anytime the pull_request_review event occurs. More than one activity type triggers this event.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#pull-request-review-event-pull_request_review
	PullRequestReview,

	/// Runs your workflow anytime a comment on a pull request's unified diff is modified, which triggers the pull_request_review_comment event. More than one activity type triggers this event.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#pull-request-review-comment-event-pull_request_review_comment
	PullRequestReviewComment,

	/// This event is similar to pull_request, except that it runs in the context of the base repository of the pull request, rather than in the merge commit.
	///
	/// See more: https://docs.github.com/en/actions/reference/events-that-trigger-workflows#pull_request_target
	PullRequestTarget,

	/// Runs your workflow when someone pushes to a repository branch, which triggers the push event.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#push-event-push
	Push,

	/// Runs your workflow anytime a package is published or updated.
	///
	/// See more: https://help.github.com/en/actions/reference/events-that-trigger-workflows#registry-package-event-registry_package
	RegistryPackage,

	/// Runs your workflow anytime the release event occurs. More than one activity type triggers this event.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#release-event-release
	Release,

	/// You can use the GitHub API to trigger a webhook event called repository_dispatch when you want to trigger a workflow for activity that happens outside of GitHub.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#external-events-repository_dispatch
	RepositoryDispatch,

	/// Runs your workflow anytime the status of a Git commit changes, which triggers the status event.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#status-event-status
	Status,

	/// Runs your workflow anytime the watch event occurs.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#watch-event-watch
	Watch,

	/// `workflow_call` is used to indicate that a workflow can be called by another workflow.
	///
	/// See more: https://docs.github.com/en/actions/learn-github-actions/events-that-trigger-workflows#workflow_call
	WorkflowCall,

	/// To enable a workflow to be triggered manually, you need to configure the `workflow_dispatch` event. You can manually trigger a workflow run using the GitHub API, GitHub CLI, or the GitHub UI.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#workflow_dispatch
	WorkflowDispatch,

	/// This event occurs when a workflow run is requested or completed, and allows you to execute a workflow based on the finished result of another workflow.
	///
	/// See more: https://docs.github.com/en/actions/reference/events-that-trigger-workflows#workflow_run
	WorkflowRun,
}

macro_rules! events {
  ($name:ident, $doc:literal, $link:literal, variants = [$($variants:ident),*]) => {
    paste::paste!{
      #[doc = $doc]
      #[doc = ""]
      #[doc = concat!("See more: ", $link)]
      #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Merge)]
			#[cfg_attr(feature = "schemars", derive(JsonSchema))]
      pub struct $name {
        /// The types of events that should trigger this workflow.
        #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
        pub types: BTreeSet<[< $name Events >]>
      }

      #[doc = "The kind of events that can trigger a " $name:snake " event"]
      #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
			#[cfg_attr(feature = "schemars", derive(JsonSchema))]
      #[serde(rename_all = "snake_case")]
      pub enum [< $name Events >] {
        $($variants),*
      }
    }
  };
}

events!(
	BranchProtectionRule,
	"Runs your workflow anytime the branch_protection_rule event occurs.",
	"https://docs.github.com/en/actions/learn-github-actions/events-that-trigger-workflows#branch_protection_rule",
	variants = [Created, Edited, Deleted]
);

events!(
	CheckRun,
	"Runs your workflow anytime the check_run event occurs.",
	"https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#check-run-event-check_run",
	variants = [Created, Completed, Rerequested, RequestedAction]
);

events!(
	CheckSuite,
	"Runs your workflow anytime the check_suite event occurs.",
	"https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#check-suite-event-check_suite",
	variants = [Completed, Requested, Rerequested]
);

events!(
	Discussion,
	"Runs your workflow anytime the discussion event occurs. More than one activity type triggers this event.",
	"https://docs.github.com/en/actions/reference/events-that-trigger-workflows#discussion",
	variants = [
		Answered,
		CategoryChanged,
		Created,
		Deleted,
		Edited,
		Labeled,
		Locked,
		Pinned,
		Transferred,
		Unanswered,
		Unlabeled,
		Unlocked,
		Unpinned
	]
);

events!(
	DiscussionComment,
	"Runs your workflow anytime the discussion_comment event occurs. More than one activity type triggers this event.",
	"https://docs.github.com/en/actions/reference/events-that-trigger-workflows#discussion_comment",
	variants = [Created, Deleted, Edited]
);

events!(
	IssueComment,
	"Runs your workflow anytime the issue_comment event occurs. More than one activity type triggers this event.",
	"https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#issue-comment-event-issue_comment",
	variants = [Created, Deleted, Edited]
);

events!(
	Issues,
	"Runs your workflow anytime the issues event occurs. More than one activity type triggers this event.",
	"https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#issues-event-issues",
	variants = [
		Assigned,
		Closed,
		Deleted,
		Edited,
		Demilestoned,
		Labeled,
		Locked,
		Milestoned,
		Opened,
		Pinned,
		Transferred,
		Unassigned,
		Unlabeled,
		Unlocked,
		Unpinned
	]
);

events!(
	Label,
	"Runs your workflow anytime the label event occurs. More than one activity type triggers this event.",
	"https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#label-event-label",
	variants = [Created, Deleted, Edited]
);

events!(
	MergeGroup,
	"Runs your workflow when a pull request is added to a merge queue, which adds the pull request to a merge group. For information about the merge queue, see https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/incorporating-changes-from-a-pull-request/merging-a-pull-request-with-a-merge-queue.",
	"https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#merge_group",
	variants = [ChecksRequested]
);

events!(
	Milestone,
	"Runs your workflow anytime the milestone event occurs. More than one activity type triggers this event.",
	"https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#milestone-event-milestone",
	variants = [Closed, Created, Deleted, Edited, Opened]
);

events!(
	Project,
	"Runs your workflow anytime the project event occurs. More than one activity type triggers this event.",
	"https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#project-event-project",
	variants = [Closed, Created, Deleted, Edited, Reopened, Updated]
);

events!(
	ProjectCard,
	"Runs your workflow anytime the project_card event occurs. More than one activity type triggers this event.",
	"https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#project-card-event-project_card",
	variants = [Converted, Created, Deleted, Edited, Moved]
);

events!(
	ProjectColumn,
	"Runs your workflow anytime the project_column event occurs. More than one activity type triggers this event.",
	"https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#project-column-event-project_column",
	variants = [Created, Deleted, Moved, Updated]
);

/// Runs your workflow when activity on a pull request in the workflow's repository occurs.
///
/// For example, if no activity types are specified, the workflow runs when a pull request is opened or reopened or when the head branch of the pull request is updated. For activity related to pull request reviews, pull request review comments, or pull request comments, use the `pull_request_review`, `pull_request_review_comment`, or `issue_comment` events instead.
///
/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#pull_request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Merge)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct PullRequest {
	/// The types of events that should trigger this workflow.
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	pub types: BTreeSet<PullRequestEvents>,

	/// Runs on pull requests that target specific branches.
	///
	/// Cannot be used with `branches-ignore`
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	pub branches: BTreeSet<String>,

	/// Runs on pull requests that target specific branches, except those listed here.
	///
	/// Cannot be used with `branches`
	#[serde(
		default,
		rename = "branches-ignore",
		skip_serializing_if = "BTreeSet::is_empty"
	)]
	pub branches_ignore: BTreeSet<String>,

	/// Cannot be used with `tags-ignore`
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	pub tags: BTreeSet<String>,

	/// Cannot be used with `tags`
	#[serde(
		default,
		rename = "tags-ignore",
		skip_serializing_if = "BTreeSet::is_empty"
	)]
	pub tags_ignore: BTreeSet<String>,

	/// Runs when a pull request changes specific files.
	///
	/// Cannot be used with `paths-ignore`
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	pub paths: BTreeSet<String>,

	/// Runs when a pull request changes specific files, except those listed here.
	///
	/// Cannot be used with `paths`
	#[serde(
		default,
		rename = "paths-ignore",
		skip_serializing_if = "BTreeSet::is_empty"
	)]
	pub paths_ignore: BTreeSet<String>,
}

/// The kinds of events that can trigger a pull_request event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum PullRequestEvents {
	Assigned,
	AutoMergeDisabled,
	AutoMergeEnabled,
	Closed,
	ConvertedToDraft,
	Demilestoned,
	Dequeued,
	Edited,
	Enqueued,
	Labeled,
	Locked,
	Milestoned,
	Opened,
	ReadyForReview,
	Reopened,
	ReviewRequested,
	ReviewRequestRemoved,
	Synchronize,
	Unassigned,
	Unlabeled,
	Unlocked,
}

events!(
	PullRequestReview,
	"Runs your workflow anytime the pull_request_review event occurs. More than one activity type triggers this event.",
	"https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#pull-request-review-event-pull_request_review",
	variants = [Dismissed, Edited, Submitted]
);

events!(
	PullRequestReviewComment,
	"Runs your workflow anytime a comment on a pull request's unified diff is modified, which triggers the pull_request_review_comment event. More than one activity type triggers this event.",
	"https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#pull-request-review-comment-event-pull_request_review_comment",
	variants = [Created, Deleted, Edited]
);

// PullRequestTarget is the same as PullRequest but with different documentation

/// Runs your workflow when someone pushes to a repository branch, which triggers the push event.
///
/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#push-event-push
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Merge)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Push {
	/// Runs only when specific branches are pushed.
	///
	/// Cannot be used with `branches-ignore`
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	pub branches: BTreeSet<String>,

	/// Runs only when specific branches are pushed, except those listed here.
	///
	/// Cannot be used with `branches`
	#[serde(
		default,
		rename = "branches-ignore",
		skip_serializing_if = "BTreeSet::is_empty"
	)]
	pub branches_ignore: BTreeSet<String>,

	/// Runs only when specific tags are pushed.
	///
	/// Cannot be used with `tags-ignore`
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	pub tags: BTreeSet<String>,

	/// Runs only when specific tags are pushed, except for the ones listed here.
	///
	/// Cannot be used with `tags`
	#[serde(
		default,
		rename = "tags-ignore",
		skip_serializing_if = "BTreeSet::is_empty"
	)]
	pub tags_ignore: BTreeSet<String>,

	/// Runs only when a push changes specific files.
	///
	/// Cannot be used with `paths-ignore`
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	pub paths: BTreeSet<String>,

	/// Runs only when a push changes specific files, except those listed here.
	///
	/// Cannot be used with `paths`
	#[serde(
		default,
		rename = "paths-ignore",
		skip_serializing_if = "BTreeSet::is_empty"
	)]
	pub paths_ignore: BTreeSet<String>,
}

events!(
	RegistryPackage,
	"Runs your workflow anytime a package is published or updated.",
	"https://help.github.com/en/actions/reference/events-that-trigger-workflows#registry-package-event-registry_package",
	variants = [Published, Updated]
);

events!(
	Release,
	"Runs your workflow anytime the release event occurs. More than one activity type triggers this event.",
	"https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#release-event-release",
	variants = [
		Created,
		Deleted,
		Edited,
		Prereleased,
		Published,
		Released,
		Unpublished
	]
);

/// This event occurs when a workflow run is requested or completed, and allows you to execute a workflow based on the finished result of another workflow.
///
/// See more: https://docs.github.com/en/actions/reference/events-that-trigger-workflows#workflow_run
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Merge)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct WorkflowRun {
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	/// The types of events that should trigger this workflow.
	pub types: BTreeSet<WorkflowRunEvents>,

	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	/// The workflows that should trigger this workflow.
	pub workflows: BTreeSet<String>,

	/// Runs only when specific branches are involved.
	///
	/// Cannot be used with `branches-ignore`
	#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
	pub branches: BTreeSet<String>,

	/// Runs only when specific branches are involved, except those listed here.
	///
	/// Cannot be used with `branches`
	#[serde(
		default,
		rename = "branches-ignore",
		skip_serializing_if = "BTreeSet::is_empty"
	)]
	pub branches_ignore: BTreeSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum WorkflowRunEvents {
	Completed,
	InProgress,
	Requested,
}

/// You can schedule a workflow to run at specific UTC times using POSIX cron syntax (https://pubs.opengroup.org/onlinepubs/9699919799/utilities/crontab.html#tag_20_25_07).
///
/// Scheduled workflows run on the latest commit on the default or base branch. The shortest interval you can run scheduled workflows is once every 5 minutes.
///
/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#scheduled-events-schedule
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Schedule {
	/// Use POSIX cron syntax to schedule workflows to run at specific UTC times. Scheduled workflows run on the latest commit on the default branch. The shortest interval you can run scheduled workflows is once every 5 minutes.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#schedule
	pub cron: String,
}

/// Workflow triggers are events that cause a workflow to run.
///
/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#about-events-that-trigger-workflows
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Merge)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[merge(with = merge_options)]
#[serde(deny_unknown_fields)]
pub struct EventObject {
	/// Runs your workflow anytime the branch_protection_rule event occurs
	///
	/// See more: https://docs.github.com/en/actions/learn-github-actions/events-that-trigger-workflows#branch_protection_rule
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub branch_protection_rule: Option<BranchProtectionRule>,

	/// Runs your workflow anytime the check_run event occurs
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#check-run-event-check_run
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub check_run: Option<CheckRun>,

	/// Runs your workflow anytime the check_suite event occurs.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#check-suite-event-check_suite
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub check_suite: Option<CheckSuite>,

	/// Runs your workflow anytime someone creates a branch or tag, which triggers the create event.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#create-event-create
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub create: Option<JsonValueBTreeMap>,

	/// Runs your workflow anytime someone deletes a branch or tag, which triggers the delete event.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#delete-event-delete
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub delete: Option<JsonValueBTreeMap>,

	/// Runs your workflow anytime someone creates a deployment, which triggers the deployment event. Deployments created with a commit SHA may not have a Git ref.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#deployment-event-deployment
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub deployment: Option<JsonValueBTreeMap>,

	/// Runs your workflow anytime a third party provides a deployment status, which triggers the deployment_status event. Deployments created with a commit SHA may not have a Git ref.
	///
	/// See more: https://docs.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub deployment_status: Option<JsonValueBTreeMap>,

	/// Runs your workflow anytime the discussion event occurs. More than one activity type triggers this event.
	///
	/// See more: https://docs.github.com/en/actions/reference/events-that-trigger-workflows#discussion
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub discussion: Option<Discussion>,

	/// Runs your workflow anytime the discussion_comment event occurs. More than one activity type triggers this event.
	///
	/// See more: https://docs.github.com/en/actions/reference/events-that-trigger-workflows#discussion_comment
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub discussion_comment: Option<DiscussionComment>,

	/// Runs your workflow anytime when someone forks a repository, which triggers the fork event.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#fork-event-fork
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub fork: Option<JsonValueBTreeMap>,

	/// Runs your workflow when someone creates or updates a Wiki page, which triggers the gollum event.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#gollum-event-gollum
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub gollum: Option<JsonValueBTreeMap>,

	/// Runs your workflow anytime the issue_comment event occurs. More than one activity type triggers this event.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#issue-comment-event-issue_comment
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub issue_comment: Option<IssueComment>,

	/// Runs your workflow anytime the issues event occurs. More than one activity type triggers this event.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#issues-event-issues
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub issues: Option<Issues>,

	/// Runs your workflow anytime the label event occurs. More than one activity type triggers this event.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#label-event-label
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub label: Option<Label>,

	/// Runs your workflow when a pull request is added to a merge queue, which adds the pull request to a merge group. For information about the merge queue, see https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/incorporating-changes-from-a-pull-request/merging-a-pull-request-with-a-merge-queue.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#merge_group
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub merge_group: Option<MergeGroup>,

	/// Runs your workflow anytime the milestone event occurs. More than one activity type triggers this event.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#milestone-event-milestone
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub milestone: Option<Milestone>,

	/// Runs your workflow anytime someone pushes to a GitHub Pages-enabled branch, which triggers the page_build event.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#page-build-event-page_build
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub page_build: Option<JsonValueBTreeMap>,

	/// Runs your workflow anytime the project event occurs. More than one activity type triggers this event.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#project-event-project
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub project: Option<Project>,

	/// Runs your workflow anytime the project_card event occurs. More than one activity type triggers this event.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#project-card-event-project_card
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub project_card: Option<ProjectCard>,

	/// Runs your workflow anytime the project_column event occurs. More than one activity type triggers this event.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#project-column-event-project_column
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub project_column: Option<ProjectColumn>,

	/// Runs your workflow anytime someone makes a private repository public, which triggers the public event.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#public-event-public
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub public: Option<JsonValueBTreeMap>,

	/// Runs your workflow anytime the pull_request event occurs. More than one activity type triggers this event.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#pull-request-event-pull_request
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub pull_request: Option<PullRequest>,

	/// Runs your workflow anytime the pull_request_review event occurs. More than one activity type triggers this event.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#pull-request-review-event-pull_request_review
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub pull_request_review: Option<PullRequestReview>,

	/// Runs your workflow anytime a comment on a pull request's unified diff is modified, which triggers the pull_request_review_comment event. More than one activity type triggers this event.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#pull-request-review-comment-event-pull_request_review_comment
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub pull_request_review_comment: Option<PullRequestReviewComment>,

	/// This event is similar to pull_request, except that it runs in the context of the base repository of the pull request, rather than in the merge commit.
	///
	/// See more: https://docs.github.com/en/actions/reference/events-that-trigger-workflows#pull_request_target
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub pull_request_target: Option<PullRequest>,

	/// Runs your workflow when someone pushes to a repository branch, which triggers the push event.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#push-event-push
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub push: Option<Push>,

	/// Runs your workflow anytime a package is published or updated.
	///
	/// See more: https://help.github.com/en/actions/reference/events-that-trigger-workflows#registry-package-event-registry_package
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub registry_package: Option<RegistryPackage>,

	/// Runs your workflow anytime the release event occurs. More than one activity type triggers this event.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#release-event-release
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub release: Option<Release>,

	/// You can use the GitHub API to trigger a webhook event called repository_dispatch when you want to trigger a workflow for activity that happens outside of GitHub.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#external-events-repository_dispatch
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub repository_dispatch: Option<JsonValueBTreeMap>,

	/// You can schedule a workflow to run at specific UTC times using POSIX cron syntax (https://pubs.opengroup.org/onlinepubs/9699919799/utilities/crontab.html#tag_20_25_07). Scheduled workflows run on the latest commit on the default or base branch. The shortest interval you can run scheduled workflows is once every 5 minutes.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#scheduled-events-schedule
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	#[merge(with = Vec::extend)]
	pub schedule: Vec<Schedule>,

	/// Runs your workflow anytime the status of a Git commit changes, which triggers the status event.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#status-event-status
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub status: Option<JsonValueBTreeMap>,

	/// Runs your workflow anytime the watch event occurs.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/events-that-trigger-workflows#watch-event-watch
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub watch: Option<JsonValueBTreeMap>,

	/// `workflow_call` is used to indicate that a workflow can be called by another workflow.
	///
	/// See more: https://docs.github.com/en/actions/learn-github-actions/events-that-trigger-workflows#workflow_call
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub workflow_call: Option<WorkflowCall>,

	/// To enable a workflow to be triggered manually, you need to configure the `workflow_dispatch` event. You can manually trigger a workflow run using the GitHub API, GitHub CLI, or the GitHub UI.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#workflow_dispatch
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub workflow_dispatch: Option<WorkflowDispatch>,

	/// This event occurs when a workflow run is requested or completed, and allows you to execute a workflow based on the finished result of another workflow.
	///
	/// See more: https://docs.github.com/en/actions/reference/events-that-trigger-workflows#workflow_run
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub workflow_run: Option<WorkflowRun>,
}

/// When using the workflow_call keyword, you can optionally specify inputs that are passed to the called workflow from the caller workflow.
///
/// See more: https://docs.github.com/en/actions/reference/workflow-syntax-for-github-actions#onworkflow_callinputs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct WorkflowCallInput {
	/// Required if input is defined for the on.workflow_call keyword. The value of this parameter is a string specifying the data type of the input. This must be one of: boolean, number, or string.
	///
	/// See more: https://docs.github.com/en/actions/learn-github-actions/workflow-syntax-for-github-actions#onworkflow_callinput_idtype
	#[serde(rename = "type")]
	pub type_: StringNumOrBool,

	/// A boolean to indicate whether the action requires the input parameter. Set to true when the parameter is required.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/metadata-syntax-for-github-actions#inputsinput_idrequired
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub required: Option<bool>,

	/// A string description of the input parameter.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/metadata-syntax-for-github-actions#inputsinput_iddescription
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,

	/// The default value is used when an input parameter isn't specified in a workflow file.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/metadata-syntax-for-github-actions#inputsinput_iddefault
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub default: Option<StringNumOrBool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
pub enum StringNumOrBool {
	String(String),
	Num(i64),
	Bool(bool),
}

/// When using the `workflow_call` keyword, you can optionally specify inputs that are passed to the called workflow from the caller workflow.
///
/// See more: https://docs.github.com/en/actions/reference/workflow-syntax-for-github-actions#onworkflow_calloutputs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct WorkflowCallOutput {
	/// A string description of the output parameter.
	///
	/// See more: https://docs.github.com/en/actions/sharing-automations/creating-actions/metadata-syntax-for-github-actions#outputsoutput_iddescription
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,

	/// The value that the output parameter will be mapped to. You can set this to a string or an expression with context. For example, you can use the steps context to set the value of an output to the output value of a step.
	///
	/// See more: https://docs.github.com/en/actions/sharing-automations/creating-actions/metadata-syntax-for-github-actions#outputsoutput_idvalue
	pub value: String,
}

/// Within the called workflow, you can use the secrets context to refer to a secret.
///
/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#onworkflow_callsecrets
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Secret {
	/// A boolean specifying whether the secret must be supplied.
	///
	/// See more: https://docs.github.com/en/actions/learn-github-actions/workflow-syntax-for-github-actions#onworkflow_callsecretssecret_idrequired
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub required: Option<bool>,

	/// A string description of the secret parameter.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,
}

/// Use `on.workflow_call` to define the inputs and outputs for a reusable workflow. You can also map the secrets that are available to the called workflow.
///
/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#onworkflow_call
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Merge)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct WorkflowCall {
	/// When using the `workflow_call` keyword, you can optionally specify inputs that are passed to the called workflow from the caller workflow.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#onworkflow_callinputs
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	pub inputs: BTreeMap<String, WorkflowCallInput>,

	/// A map of outputs for a called workflow. Called workflow outputs are available to all downstream jobs in the caller workflow. Each output has an identifier, an optional `description`, and a `value`. The `value` must be set to the value of an output from a job within the called workflow.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#onworkflow_calloutputs
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	pub outputs: BTreeMap<String, WorkflowCallOutput>,

	/// A map of the secrets that can be used in the called workflow.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#onworkflow_callsecrets
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	pub secrets: BTreeMap<String, Secret>,
}

/// The triggered workflow receives the inputs in the `inputs` context.
///
/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#onworkflow_dispatchinputs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct WorkflowDispatchInput {
	/// A string representing the type of the input.
	///
	/// See more: https://docs.github.com/en/actions/using-workflows/workflow-syntax-for-github-actions#onworkflow_dispatchinputsinput_idtype
	#[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
	pub type_: Option<InputType>,

	/// A string representing the default value. The default value is used when an input parameter isn't specified in a workflow file.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/metadata-syntax-for-github-actions#inputsinput_iddefault
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub required: Option<bool>,

	/// A string description of the input parameter.
	///
	/// See more: https://help.github.com/en/github/automating-your-workflow-with-github-actions/metadata-syntax-for-github-actions#inputsinput_iddescription
	pub description: String,

	/// A string shown to users using the deprecated input.
	#[serde(
		default,
		rename = "deprecationMessage",
		skip_serializing_if = "Option::is_none"
	)]
	pub deprecation_message: Option<String>,

	/// The options of the dropdown list, if the type is a choice.
	///
	/// See more: https://github.blog/changelog/2021-11-10-github-actions-input-types-for-manual-workflows
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub options: Option<BTreeSet<String>>,
}

/// A string representing the type of the input.
///
/// See more: https://docs.github.com/en/actions/using-workflows/workflow-syntax-for-github-actions#onworkflow_dispatchinputsinput_idtype
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum InputType {
	Boolean,
	Choice,
	Environment,
	Number,
	String,
}

/// When using the `workflow_dispatch` event, you can optionally specify inputs that are passed to the workflow.
///
/// This trigger only receives events when the workflow file is on the default branch.
///
/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#onworkflow_dispatch
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Merge)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct WorkflowDispatch {
	/// The triggered workflow receives the inputs in the `inputs` context.
	///
	/// See more: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#onworkflow_dispatchinputs
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	pub inputs: BTreeMap<String, WorkflowDispatchInput>,
}
