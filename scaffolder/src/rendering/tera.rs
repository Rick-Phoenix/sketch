use std::{
  fs::{create_dir_all, File},
  io::ErrorKind,
  path::PathBuf,
};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tera::{Context, Tera};

use crate::{config::Config, GenError};

/// The types of configuration values for a template's data.
/// It can either be an id (which points to the key used to store a literal template in the config, or to a file path starting from the root of the templates directory specified in the config.)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum TemplateData {
  Content { name: String, content: String },
  Id(String),
}

impl TemplateData {
  pub fn name(&self) -> &str {
    match self {
      TemplateData::Content { name, .. } => name,
      TemplateData::Id(name) => name,
    }
  }
}

/// The data for outputting a new template.
/// The output directory will be joined to the root of the package being generated with this template.
/// The context specified here will override the global context.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TemplateOutput {
  pub template: TemplateData,
  pub output: String,
  #[serde(default)]
  pub context: IndexMap<String, Value>,
}

impl Config {
  /// A helper to generate custom templates.
  pub fn generate_templates(
    self,
    output_root: &str,
    templates: Vec<TemplateOutput>,
  ) -> Result<(), GenError> {
    let mut tera = if let Some(templates_dir) = self.templates_dir {
      Tera::new(&format!("{}/**/*", templates_dir))
        .map_err(|e| GenError::TemplateDirLoading { source: e })?
    } else {
      Tera::default()
    };

    for (name, template) in &self.templates {
      tera
        .add_raw_template(name, template)
        .map_err(|e| GenError::TemplateParsing {
          template: name.to_string(),
          source: e,
        })?;
    }

    let global_context = Context::from_serialize(self.global_templates_vars)
      .map_err(|e| GenError::TemplateContextParsing { source: e })?;

    for template in templates {
      let mut local_context = global_context.clone();

      if !template.context.is_empty() {
        let added_context = Context::from_serialize(template.context)
          .map_err(|e| GenError::TemplateContextParsing { source: e })?;

        local_context.extend(added_context);
      }

      let output_path = PathBuf::from(output_root).join(template.output);

      create_dir_all(output_path.parent().ok_or(GenError::Custom(format!(
        "Could not get the parent directory for '{}'",
        output_path.display()
      )))?)
      .map_err(|e| GenError::ParentDirCreation {
        path: output_path.clone(),
        source: e,
      })?;

      let mut output_file = if self.overwrite {
        File::create(&output_path).map_err(|e| GenError::FileCreation {
          path: output_path.clone(),
          source: e,
        })?
      } else {
        File::create_new(&output_path).map_err(|e| match e.kind() {
          ErrorKind::AlreadyExists => GenError::FileExists {
            path: output_path.clone(),
          },
          _ => GenError::WriteError {
            path: output_path.clone(),
            source: e,
          },
        })?
      };

      match template.template {
        TemplateData::Content { name, content } => {
          tera
            .add_raw_template(&name, &content)
            .map_err(|e| GenError::TemplateParsing {
              template: name.to_string(),
              source: e,
            })?;
          tera
            .render_to(&name, &local_context, &mut output_file)
            .map_err(|e| GenError::TemplateRendering {
              template: name.to_string(),
              source: e,
            })?
        }
        TemplateData::Id(path) => tera
          .render_to(&path, &local_context, &mut output_file)
          .map_err(|e| GenError::TemplateRendering {
            template: path.to_string(),
            source: e,
          })?,
      }
    }

    Ok(())
  }
}

#[cfg(test)]
mod test {
  use std::{fs::File, path::PathBuf};

  use serde::{Deserialize, Serialize};

  use crate::{config::Config, GenError};

  #[derive(Debug, Serialize, Deserialize)]
  struct CustomTemplateTest {
    pub my_var: usize,
  }

  #[test]
  fn custom_templates() -> Result<(), GenError> {
    let config = Config::from_file(PathBuf::from(
      "tests/custom_templates/custom_templates.toml",
    ))?;

    let templates = config
      .package_presets
      .get("custom_templates_test")
      .unwrap()
      .generate_templates
      .clone()
      .unwrap();

    config.generate_templates("output/custom_templates", templates.clone())?;

    for template in templates {
      let output_path = PathBuf::from("output/custom_templates").join(template.output);
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
}
