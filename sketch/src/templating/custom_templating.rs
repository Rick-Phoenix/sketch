use std::{
  borrow::Cow,
  path::{Path, PathBuf},
};

use indexmap::IndexMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tera::{Context, Tera};
use walkdir::WalkDir;

use crate::{
  config::Config,
  fs::{create_all_dirs, get_abs_path, get_parent_dir, open_file_if_overwriting},
  tera_setup::get_default_context,
  GenError, Preset,
};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum TemplatingPresetReference {
  Preset {
    id: String,
    #[serde(default)]
    context: IndexMap<String, Value>,
  },
  Definition(TemplatingPreset),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum TemplatingPreset {
  Single(TemplateOutput),
  Collection {
    templates: Vec<TemplateOutput>,
    #[serde(default)]
    context: IndexMap<String, Value>,
  },
  Structured {
    dir: PathBuf,
    #[serde(default)]
    context: IndexMap<String, Value>,
  },
}

/// The types of configuration values for a template's data.
/// It can either be an id (which points to the key used to store a literal template in the config, or to a file path starting from the root of the templates directory specified in the config.)
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, JsonSchema)]
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
/// The context specified here will override the global context (but not the variables set via cli).
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, JsonSchema)]
pub struct TemplateOutput {
  pub template: TemplateData,
  pub output: String,
  #[serde(default)]
  pub context: IndexMap<String, Value>,
}

impl Config {
  /// A helper to generate custom templates.
  pub fn generate_templates<T: AsRef<Path>>(
    self,
    output_root: T,
    templates: Vec<TemplatingPresetReference>,
  ) -> Result<(), GenError> {
    let overwrite = !self.no_overwrite;
    let mut tera = self.initialize_tera()?;

    let mut global_context = Context::from_serialize(self.vars)
      .map_err(|e| GenError::TemplateContextParsing { source: e })?;

    global_context.extend(get_default_context());

    let output_root = output_root.as_ref();

    for template in templates {
      let mut local_context = Cow::Borrowed(&global_context);

      let preset = match template {
        TemplatingPresetReference::Preset { id, context } => {
          local_context = get_local_context(local_context, context)?;

          self
            .templating_presets
            .get(&id)
            .ok_or(GenError::PresetNotFound {
              kind: Preset::Templates,
              name: id,
            })?
            .clone()
        }
        TemplatingPresetReference::Definition(preset) => preset,
      };

      match preset {
        TemplatingPreset::Collection { templates, context } => {
          local_context = get_local_context(local_context, context)?;

          for template in templates {
            render_template_with_output(
              overwrite,
              &mut tera,
              &local_context,
              output_root,
              template,
            )?;
          }
        }
        TemplatingPreset::Structured { dir, context } => {
          local_context = get_local_context(local_context, context)?;

          render_structured_preset(
            overwrite,
            &tera,
            &local_context,
            output_root,
            dir,
            &self
              .templates_dir
              .clone()
              .ok_or(GenError::Custom(format!("templates_dir not set")))?,
          )?;
        }
        TemplatingPreset::Single(template_with_output) => {
          render_template_with_output(
            overwrite,
            &mut tera,
            &local_context,
            output_root,
            template_with_output,
          )?;
        }
      };
    }

    Ok(())
  }
}

fn render_template_with_output(
  overwrite: bool,
  tera: &mut Tera,
  context: &Context,
  output_root: &Path,
  template: TemplateOutput,
) -> Result<(), GenError> {
  let local_context = get_local_context(Cow::Borrowed(context), template.context)?;

  let template_name = template.template.name();

  if let TemplateData::Content { name, content } = &template.template {
    tera
      .add_raw_template(name, content)
      .map_err(|e| GenError::TemplateParsing {
        template: name.to_string(),
        source: e,
      })?;
  }

  if template.output == "__stdout" {
    let output =
      tera
        .render(template_name, &local_context)
        .map_err(|e| GenError::TemplateRendering {
          template: template_name.to_string(),
          source: e,
        })?;

    println!("{}", output);
  } else {
    let output_path = output_root.join(template.output);

    render_template(tera, template_name, &output_path, &local_context, overwrite)?;
  }

  Ok(())
}

fn render_structured_preset(
  overwrite: bool,
  tera: &Tera,
  context: &Context,
  output_root: &Path,
  dir: PathBuf,
  templates_dir: &Path,
) -> Result<(), GenError> {
  let templates_dir = get_abs_path(templates_dir)?;
  let root_dir = templates_dir.join(&dir);
  if !root_dir.is_dir() {
    return Err(GenError::Custom(format!(
      "`{}` is not a valid directory inside `templates_dir`",
      dir.display()
    )));
  }
  Ok(
    for entry in WalkDir::new(&root_dir)
      .into_iter()
      .filter_map(|e| e.ok())
      .filter(|e| e.file_type().is_file())
    {
      let template_path = entry.path().strip_prefix(&templates_dir).unwrap();
      let output_path = entry.path().strip_prefix(&root_dir).unwrap();

      render_template(
        tera,
        &template_path.to_string_lossy(),
        &output_root.join(output_path),
        context,
        overwrite,
      )?;
    },
  )
}

fn render_template(
  tera: &Tera,
  template_name: &str,
  output_path: &Path,
  context: &Context,
  overwrite: bool,
) -> Result<(), GenError> {
  create_all_dirs(get_parent_dir(&output_path))?;

  let mut output_file = open_file_if_overwriting(overwrite, &output_path)?;

  tera
    .render_to(template_name, &context, &mut output_file)
    .map_err(|e| GenError::TemplateRendering {
      template: template_name.to_string(),
      source: e,
    })
}

fn get_local_context<'a>(
  global_context: Cow<'a, Context>,
  new_context: IndexMap<String, Value>,
) -> Result<Cow<'a, Context>, GenError> {
  if new_context.is_empty() {
    Ok(global_context)
  } else {
    let mut local_context = global_context.as_ref().clone();

    let added_context = Context::from_serialize(new_context)
      .map_err(|e| GenError::TemplateContextParsing { source: e })?;

    local_context.extend(added_context);

    Ok(Cow::Owned(local_context))
  }
}
