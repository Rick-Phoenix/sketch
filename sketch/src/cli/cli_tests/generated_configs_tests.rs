use std::path::{Path, PathBuf};

use clap::Parser;
use pretty_assertions::assert_eq;

use super::reset_testing_dir;
use crate::{
  cli::{cli_tests::get_clean_example_cmd, execute_cli, Cli},
  docker::compose::{service::ServiceVolume, ComposeFile},
  fs::{deserialize_toml, deserialize_yaml},
  git_workflow::{
    ActionRunner, Event, Job, JobReference, RunsOn, Shell, StringNumOrBool, StringOrBool, Workflow,
  },
  rust::Manifest,
  serde_utils::StringOrNum,
  Config,
};

#[tokio::test]
async fn generated_configs() -> Result<(), Box<dyn std::error::Error>> {
  let output_dir = PathBuf::from("tests/output/generated_configs");
  let commands_dir = output_dir.join("commands");

  reset_testing_dir(&output_dir);
  reset_testing_dir(&commands_dir);

  let default_config = Cli::try_parse_from([
    "sketch",
    "--ignore-config",
    "new",
    &output_dir.join("default_config.yaml").to_string_lossy(),
  ])?;

  execute_cli(default_config).await?;

  let default_config_output: Config = deserialize_yaml(&output_dir.join("default_config.yaml"))?;

  assert_eq!(default_config_output, Config::default());

  let config_file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../examples/presets.yaml");
  let output_file = output_dir.join("compose.yaml");

  let compose_file_cmd = [
    "sketch",
    "-c",
    &config_file.to_string_lossy(),
    "docker-compose",
    "extended",
    &output_file.to_string_lossy(),
  ];

  let compose_file = Cli::try_parse_from(compose_file_cmd)?;

  get_clean_example_cmd(&compose_file_cmd, &[1, 2, 5], &commands_dir.join("compose"))?;

  execute_cli(compose_file).await?;

  let output: ComposeFile = deserialize_yaml(&output_file)?;

  let mut services = output.services;

  let service = services.remove("my_service").unwrap().as_config()?;

  assert!(service.networks.as_ref().unwrap().contains("my_network"));
  assert!(service
    .volumes
    .as_ref()
    .unwrap()
    .contains(&ServiceVolume::Simple("my_volume:/target".to_string())));

  let db_service = services.remove("db").unwrap().as_config()?;

  assert!(db_service.image.unwrap() == "postgres");
  assert!(db_service.networks.as_ref().unwrap().contains("my_network"));

  assert_eq!(
    db_service.environment.as_ref().unwrap().get("TZ").unwrap(),
    "Europe/Berlin"
  );

  let networks = output.networks.unwrap();
  let my_network = networks.get("my_network").unwrap();

  assert!(my_network.external.unwrap());

  let volumes = output.volumes.unwrap();
  let my_volume = volumes.get("my_volume").unwrap();

  assert!(my_volume.external.unwrap());

  let my_other_volume = volumes.get("my_other_volume").unwrap();

  assert!(my_other_volume.external.unwrap());

  let output_file = output_dir.join("Cargo.toml");

  let cargo_cmd = [
    "sketch",
    "-c",
    &config_file.to_string_lossy(),
    "cargo-toml",
    "extended",
    &output_file.to_string_lossy(),
  ];

  let cargo_toml_gen = Cli::try_parse_from(cargo_cmd)?;

  execute_cli(cargo_toml_gen).await?;

  let output: Manifest = deserialize_toml(&output_file)?;

  assert!(output.dependencies.contains_key("serde"));
  assert!(output.dependencies.contains_key("regex"));
  assert!(output.dependencies.contains_key("tokio"));

  let output_file = output_dir.join("workflow.yaml");

  let gh_workflow_cmd = [
    "sketch",
    "-c",
    &config_file.to_string_lossy(),
    "gh-workflow",
    "extended",
    &output_file.to_string_lossy(),
  ];

  let gh_workflow = Cli::try_parse_from(gh_workflow_cmd)?;

  execute_cli(gh_workflow).await?;

  get_clean_example_cmd(&gh_workflow_cmd, &[1, 2, 5], &commands_dir.join("workflow"))?;

  verify_generated_workflow(&output_file)?;

  Ok(())
}

pub(crate) fn verify_generated_workflow(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
  let output: Workflow = deserialize_yaml(path)?;

  let on = if let Event::Object(on) = output.on.unwrap() {
    on
  } else {
    unreachable!();
  };

  assert!(on.push.unwrap().branches.unwrap().contains("main"));

  let env = output.env;

  assert_eq!(
    env.get("my_env").unwrap(),
    &StringNumOrBool::String("somevalue".to_string())
  );

  assert_eq!(
    env.get("another_env").unwrap(),
    &StringNumOrBool::String("anothervalue".to_string())
  );

  assert_eq!(output.defaults.unwrap().run.shell.unwrap(), Shell::Bash);

  let mut jobs = output.jobs;

  assert_eq!(jobs.len(), 2);

  let say_hello_job = unwrap_variant!(
    Job,
    Normal,
    unwrap_variant!(JobReference, Data, jobs.shift_remove("say_hello").unwrap()).job
  );

  let say_goodbye_job = unwrap_variant!(
    Job,
    Normal,
    unwrap_variant!(
      JobReference,
      Data,
      jobs.shift_remove("say_goodbye").unwrap()
    )
    .job
  );

  for (i, job) in [say_hello_job, say_goodbye_job].into_iter().enumerate() {
    assert_eq!(
      job.runs_on.unwrap(),
      RunsOn::Single(ActionRunner::UbuntuLatest)
    );

    let env = job.env;

    assert_eq!(
      env.get("my_env").unwrap(),
      &StringNumOrBool::String("somevalue".to_string())
    );

    assert_eq!(
      env.get("another_env").unwrap(),
      &StringNumOrBool::String("anothervalue".to_string())
    );

    if i == 0 {
      assert_eq!(
        env.get("another_other_value").unwrap(),
        &StringNumOrBool::String("yetanothervalue".to_string())
      );
    }

    assert_eq!(job.timeout_minutes.unwrap(), StringOrNum::Num(25));

    let continue_on_error = unwrap_variant!(StringOrBool, Bool, job.continue_on_error.unwrap());
    assert!(!continue_on_error);

    let steps = job.steps;

    assert_eq!(steps.len(), 2);

    assert_eq!(steps[0].name.as_ref().unwrap(), "Initial checkup");

    assert_eq!(steps[0].run.as_ref().unwrap(), "./setup_script.sh");

    if i == 0 {
      assert_eq!(steps[1].name.as_ref().unwrap(), "say_hello");
      assert_eq!(steps[1].run.as_ref().unwrap(), "echo \"hello!\"");
    } else {
      assert_eq!(steps[1].run.as_ref().unwrap(), "echo \"goodbye!\"");
    }
  }

  Ok(())
}
