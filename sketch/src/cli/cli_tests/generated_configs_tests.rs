use std::{
  fs::read_to_string,
  path::{Path, PathBuf},
};

use clap::Parser;
use indoc::indoc;
use pretty_assertions::assert_eq;

use super::reset_testing_dir;
use crate::{
  cli::{cli_tests::get_clean_example_cmd, execute_cli, Cli},
  docker::compose::{
    service::{Port, ServiceVolume},
    ComposeFile,
  },
  fs::{deserialize_json, deserialize_toml, deserialize_yaml},
  git_workflow::{
    ActionRunner, Event, Job, JobReference, RunsOn, Shell, StringNumOrBool, StringOrBool, Workflow,
  },
  rust::Manifest,
  serde_utils::StringOrNum,
  ts::package_json::PackageJson,
  Config,
};

#[tokio::test]
async fn js_catalog() -> Result<(), Box<dyn std::error::Error>> {
  let output_dir = PathBuf::from("tests/output/bun_package");
  let config_path = PathBuf::from("tests/bun.yaml");

  reset_testing_dir(&output_dir);

  let cmd = Cli::try_parse_from([
    "sketch",
    "--ignore-config",
    "-c",
    &config_path.to_string_lossy(),
    "ts",
    "package",
    "--preset",
    "with_catalog",
    &output_dir.to_string_lossy(),
  ])?;

  execute_cli(cmd).await?;

  let target_package_json: PackageJson = deserialize_json(&output_dir.join("package.json"))?;

  assert!(target_package_json.catalog.contains_key("hono"));
  assert!(target_package_json
    .catalogs
    .get("svelte")
    .unwrap()
    .contains_key("svelte"));

  Ok(())
}

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
    "--ignore-config",
    "-c",
    &config_file.to_string_lossy(),
    "docker-compose",
    "--service",
    "caddy",
    "extended",
    &output_file.to_string_lossy(),
  ];

  let compose_file = Cli::try_parse_from(compose_file_cmd)?;

  get_clean_example_cmd(&compose_file_cmd, &[1, 2, 3], &commands_dir.join("compose"))?;

  execute_cli(compose_file).await?;

  let output: ComposeFile = deserialize_yaml(&output_file)?;

  let mut services = output.services;

  let caddy_service = services.remove("caddy").unwrap().as_config()?;

  assert!(caddy_service
    .networks
    .as_ref()
    .unwrap()
    .contains("my_network"));
  assert!(caddy_service
    .ports
    .as_ref()
    .unwrap()
    .contains(&Port::String("80:80".to_string())));
  assert!(caddy_service
    .ports
    .as_ref()
    .unwrap()
    .contains(&Port::String("443:443".to_string())));

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
    "--ignore-config",
    "-c",
    &config_file.to_string_lossy(),
    "cargo-toml",
    "cli-custom",
    &output_file.to_string_lossy(),
  ];

  let cargo_toml_gen = Cli::try_parse_from(cargo_cmd)?;

  execute_cli(cargo_toml_gen).await?;

  let output: Manifest = deserialize_toml(&output_file)?;

  let serde_features = output
    .dependencies
    .get("serde")
    .unwrap()
    .features()
    .unwrap();

  assert!(serde_features.contains("preserve_order"));

  let indexmap_features = output
    .dependencies
    .get("indexmap")
    .unwrap()
    .features()
    .unwrap();

  assert!(indexmap_features.contains("serde"));

  let clap_features = output.dependencies.get("clap").unwrap().features().unwrap();

  assert!(clap_features.contains("derive"));

  let owo_colors_features = output
    .dependencies
    .get("owo-colors")
    .unwrap()
    .features()
    .unwrap();

  assert!(owo_colors_features.contains("supports-colors"));

  assert!(output.dependencies.contains_key("ratatui"));

  let output_file = output_dir.join("workflow.yaml");

  let gh_workflow_cmd = [
    "sketch",
    "--ignore-config",
    "-c",
    &config_file.to_string_lossy(),
    "gh-workflow",
    "extended",
    &output_file.to_string_lossy(),
  ];

  let gh_workflow = Cli::try_parse_from(gh_workflow_cmd)?;

  execute_cli(gh_workflow).await?;

  get_clean_example_cmd(
    &gh_workflow_cmd,
    &[1, 2, 3, 5],
    &commands_dir.join("workflow"),
  )?;

  verify_generated_workflow(&output_file)?;

  let gitignore_path = output_dir.join(".gitignore");

  let gitignore_cmd = [
    "sketch",
    "--ignore-config",
    "-c",
    &config_file.to_string_lossy(),
    "gitignore",
    "ts",
    &gitignore_path.to_string_lossy(),
  ];

  let gitignore_gen = Cli::try_parse_from(gitignore_cmd)?;

  execute_cli(gitignore_gen).await?;

  let gitignore_output = read_to_string(&gitignore_path)?;

  let gitignore_entries: Vec<&str> = gitignore_output.split('\n').collect();

  for entry in ["*.env", "dist", "*.tsBuildInfo", "node_modules"] {
    assert!(gitignore_entries.contains(&entry));
  }

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

  let jobs = output.jobs;

  assert_eq!(jobs.len(), 2);

  for (name, job) in jobs {
    let mut job = if let JobReference::Data(data) = job && let Job::Normal(content) = data.job {
      content
    } else {
      panic!()
    };

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

    assert_eq!(job.timeout_minutes.unwrap(), StringOrNum::Num(25));

    let continue_on_error = unwrap_variant!(StringOrBool, Bool, job.continue_on_error.unwrap());
    assert!(!continue_on_error);

    if name == "check_main_branch" {
      assert_eq!(
        env.get("another_other_value").unwrap(),
        &StringNumOrBool::String("yetanothervalue".to_string())
      );

      assert_eq!(
        job.outputs.unwrap().get("is_on_main").unwrap(),
        "${{ steps.branch_check.outputs.is_on_main }}"
      );

      let first_step = job.steps.remove(0).as_config().unwrap();

      assert_eq!(
        first_step.name.as_ref().unwrap(),
        "Check if a tag is on the main branch"
      );
      assert_eq!(first_step.id.as_ref().unwrap(), "branch_check");
      assert_eq!(
        first_step.run.unwrap().trim(),
        indoc! {
          r#"
            if git branch -r --contains ${{ github.ref }} | grep -q 'origin/main'; then
            echo "On main branch. Proceeding with the workflow..."
            echo "is_on_main=true" >> "$GITHUB_OUTPUT"
            else
            echo "Not on main branch. Skipping workflow..."
            fi
          "#
        }
        .trim()
      );
    } else {
      assert_eq!(
        job.name.as_ref().unwrap(),
        "Do something while on main branch"
      );

      assert_eq!(
        job.if_.unwrap(),
        "needs.check_branch.outputs.is_on_main == 'true'"
      );

      let first_step = job.steps.remove(0).as_config().unwrap();

      assert_eq!(
        first_step.run.as_ref().unwrap(),
        "echo \"Done something from main branch!\""
      );
    }
  }

  Ok(())
}
