use std::{
  collections::BTreeMap,
  fs::{create_dir_all, File},
  path::PathBuf,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tera::{Context, Tera};

use crate::{config::Config, GenError};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum TemplateData {
  Content { name: String, content: String },
  Path(String),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TemplateOutput {
  pub template: TemplateData,
  pub output: String,
  #[serde(default)]
  pub context: BTreeMap<String, Value>,
}

impl Config {
  pub fn generate_templates(
    &self,
    output_root: &str,
    templates: Vec<TemplateOutput>,
  ) -> Result<(), GenError> {
    let mut tera = if let Some(ref templates_dir) = self.templates_dir {
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

    let mut context = Context::new();

    let global_context = Context::from_serialize(self.global_templates_vars.clone())
      .map_err(|e| GenError::TemplateContextParsing { source: e })?;
    context.extend(global_context);

    for template in templates {
      let mut local_context = context.clone();
      let added_context = Context::from_serialize(template.context)
        .map_err(|e| GenError::TemplateContextParsing { source: e })?;
      local_context.extend(added_context);

      let output_path = PathBuf::from(output_root).join(template.output);
      let mut output_file = File::create(&output_path).map_err(|e| GenError::FileCreation {
        path: output_path.clone(),
        source: e,
      })?;

      create_dir_all(output_path.parent().expect("Invalid file output path")).map_err(|e| {
        GenError::ParentDirCreation {
          path: output_path.clone(),
          source: e,
        }
      })?;

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
        TemplateData::Path(path) => tera
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
