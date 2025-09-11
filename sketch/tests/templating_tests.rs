use std::{fs::File, path::PathBuf};

use serde::{Deserialize, Serialize};
use sketch_it::{config::Config, GenError};

#[derive(Debug, Serialize, Deserialize)]
struct CustomTemplateTest {
  pub my_var: usize,
}

#[test]
fn custom_templates() -> Result<(), GenError> {
  let config = Config::from_file(PathBuf::from(
    "tests/custom_templates/custom_templates.toml",
  ))?;

  let presets = config.typescript.clone().unwrap().package_presets.clone();

  let templates = presets
    .get("custom_templates_test")
    .unwrap()
    .clone()
    .generate_templates
    .unwrap();

  config.generate_templates("tests/output/custom_templates", templates.clone())?;

  for template in templates {
    let output_path = PathBuf::from("tests/output/custom_templates").join(template.output);
    let output_file =
      File::open(&output_path).expect("Could not open file in output/text for testing");

    let output: CustomTemplateTest = serde_yaml_ng::from_reader(&output_file).map_err(|e| {
      GenError::Custom(format!(
        "Could not read the output path {} in the custom_templates test: {}",
        output_path.display(),
        e
      ))
    })?;

    if output_path
      .to_string_lossy()
      .ends_with("with_override.yaml")
    {
      assert_eq!(output.my_var, 20);
    } else {
      assert_eq!(output.my_var, 15);
    }
  }

  Ok(())
}
