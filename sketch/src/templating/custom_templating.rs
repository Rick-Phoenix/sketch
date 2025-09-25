use std::{
  borrow::Cow,
  path::{Path, PathBuf},
};

use globset::{Glob, GlobSetBuilder};
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

/// A reference to a templating preset, or a new preset definition.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum TemplatingPresetReference {
  /// A reference to a templating preset, with some optional context
  Preset {
    /// The id of the preset to select.
    id: String,
    /// Additional context for the templates in this preset. It overrides previously set values, but not values set via the cli.
    #[serde(default)]
    context: IndexMap<String, Value>,
  },
  /// The definition for a new templating preset.
  Definition(TemplatingPreset),
}

/// A templating preset. It stores information about one or many templates, such as their source, output paths and contextual variables.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum TemplatingPreset {
  /// Data for a single template.
  Single(TemplateOutput),
  /// A list of individual templates, with extra optional context.
  Collection {
    /// A list of individual templates to include in this preset.
    templates: Vec<TemplateOutput>,
    /// Additional context for the templates in this preset. It overrides previously set values, but not values set via the cli.
    #[serde(default)]
    context: IndexMap<String, Value>,
  },
  /// A structured preset. It points to a directory within `templates_dir`, and optionally adds additional context. All of the templates inside the specified directory will be recursively rendered in the destination directory, with the same exact directory structure and names. If a template file ends with a `jinja` extension such as `.j2`, that gets stripped automatically.
  Structured {
    /// A relative path to a directory starting from `templates_dir`
    dir: PathBuf,
    /// A list of glob patterns for the templates to exclude
    exclude: Option<Vec<String>>,
    /// Additional context for the templates in this preset. It overrides previously set values, but not values set via the cli.
    #[serde(default)]
    context: IndexMap<String, Value>,
  },
}

/// The types of configuration values for a template's data.
/// It can either be an id (which points to the key used to store a literal template in the config, or to a file path starting from the root of the templates directory specified in the config.)
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum TemplateData {
  /// A literal definition for a template.
  Content {
    /// The id of the newly created template.
    name: String,
    /// The content of the new template.
    content: String,
  },
  /// An id pointing to a template defined in a configuration file or inside `templates_dir`.
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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, JsonSchema)]
pub enum TemplateOutputKind {
  #[serde(skip)]
  Stdout,
  #[serde(untagged)]
  Path(PathBuf),
}

/// The data for outputting a new template.
/// The context specified here will override the global context (but not the variables set via cli).
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, JsonSchema)]
pub struct TemplateOutput {
  /// The definition or id for the template to use.
  pub template: TemplateData,
  /// The output path for the generated file.
  pub output: TemplateOutputKind,
  #[serde(default)]
  /// Additional context for the templates in this preset. It overrides previously set values, but not values set via the cli.
  pub context: IndexMap<String, Value>,
}

impl Config {
  /// A helper to generate custom templates.
  pub fn generate_templates<T: AsRef<Path>>(
    self,
    output_root: T,
    templates: Vec<TemplatingPresetReference>,
    cli_overrides: Option<Vec<(String, Value)>>,
  ) -> Result<(), GenError> {
    let overwrite = !self.no_overwrite;
    let mut tera = self.initialize_tera()?;

    let mut global_context = Context::from_serialize(self.vars)
      .map_err(|e| GenError::TemplateContextParsing { source: e })?;

    global_context.extend(get_default_context());

    let output_root = output_root.as_ref();

    for template in templates {
      let mut local_context: IndexMap<String, Value> = IndexMap::new();

      let preset = match template {
        TemplatingPresetReference::Preset { id, context } => {
          local_context.extend(context);

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
          local_context.extend(context);

          let context = get_context(&global_context, local_context, cli_overrides.as_deref())?;

          for template in templates {
            render_template_with_output(
              overwrite,
              &mut tera,
              &context,
              output_root,
              template.output,
              template.template,
            )?;
          }
        }
        TemplatingPreset::Structured {
          dir,
          context,
          exclude,
        } => {
          local_context.extend(context);

          let context = get_context(&global_context, local_context, cli_overrides.as_deref())?;

          render_structured_preset(
            overwrite,
            &tera,
            &context,
            output_root,
            dir,
            &self
              .templates_dir
              .clone()
              .ok_or(GenError::Custom(format!("templates_dir not set")))?,
            exclude,
          )?;
        }
        TemplatingPreset::Single(template) => {
          local_context.extend(template.context);

          let context = get_context(&global_context, local_context, cli_overrides.as_deref())?;

          render_template_with_output(
            overwrite,
            &mut tera,
            &context,
            output_root,
            template.output,
            template.template,
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
  output: TemplateOutputKind,
  template: TemplateData,
) -> Result<(), GenError> {
  let template_name = template.name();

  if let TemplateData::Content { name, content } = &template {
    tera
      .add_raw_template(name, content)
      .map_err(|e| GenError::TemplateParsing {
        template: name.to_string(),
        source: e,
      })?;
  }

  match output {
    TemplateOutputKind::Stdout => {
      let output =
        tera
          .render(template_name, context)
          .map_err(|e| GenError::TemplateRendering {
            template: template_name.to_string(),
            source: e,
          })?;

      println!("{}", output);
    }
    TemplateOutputKind::Path(path) => {
      render_template(
        tera,
        template_name,
        &output_root.join(path),
        context,
        overwrite,
      )?;
    }
  };

  Ok(())
}

fn render_structured_preset(
  overwrite: bool,
  tera: &Tera,
  context: &Context,
  output_root: &Path,
  dir: PathBuf,
  templates_dir: &Path,
  exclude: Option<Vec<String>>,
) -> Result<(), GenError> {
  let templates_dir = get_abs_path(templates_dir)?;
  let root_dir = templates_dir.join(&dir);
  if !root_dir.is_dir() {
    return Err(GenError::Custom(format!(
      "`{}` is not a valid directory inside `templates_dir`",
      dir.display()
    )));
  }

  let globset = if let Some(ref patterns) = exclude {
    let mut glob_builder = GlobSetBuilder::new();

    for pattern in patterns {
      glob_builder.add(
        Glob::new(pattern)
          .map_err(|e| generic_error!("Could not parse glob pattern `{}`: {}", pattern, e))?,
      );
    }

    Some(
      glob_builder
        .build()
        .map_err(|e| generic_error!("Could not build globset: {}", e))?,
    )
  } else {
    None
  };

  Ok(
    for entry in WalkDir::new(&root_dir)
      .into_iter()
      .filter_map(|e| e.ok())
      .filter(|e| e.file_type().is_file())
    {
      let template_path = entry
        .path()
        .strip_prefix(&templates_dir)
        .map_err(|_| generic_error!("`dir` must be a directory inside `templates_dir`"))?;
      let mut output_path = entry
        .path()
        .strip_prefix(&root_dir)
        .map_err(|_| generic_error!("`dir` must be a directory inside `templates_dir`"))?
        .to_path_buf();

      if let Some(ref globset) = globset {
        if globset.is_match(&template_path) {
          continue;
        }
      }

      if output_path
        .extension()
        .is_some_and(|e| e == "j2" || e == "jinja" || e == "jinja2")
      {
        output_path = output_path.with_extension("");
      }

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
  global_context: &'a Context,
  local_context: IndexMap<String, Value>,
) -> Result<Cow<'a, Context>, GenError> {
  if local_context.is_empty() {
    Ok(Cow::Borrowed(global_context))
  } else {
    let mut new_context = global_context.clone();

    new_context.extend(create_context(local_context)?);

    Ok(Cow::Owned(new_context))
  }
}

fn get_context<'a>(
  global_context: &'a Context,
  mut local_context: IndexMap<String, Value>,
  overrides: Option<&'a [(String, Value)]>,
) -> Result<Cow<'a, Context>, GenError> {
  apply_cli_overrides(&mut local_context, overrides)?;
  get_local_context(global_context, local_context)
}

fn create_context(context: IndexMap<String, Value>) -> Result<Context, GenError> {
  Context::from_serialize(context).map_err(|e| GenError::TemplateContextParsing { source: e })
}

fn apply_cli_overrides(
  context: &mut IndexMap<String, Value>,
  overrides: Option<&[(String, Value)]>,
) -> Result<(), GenError> {
  if let Some(overrides) = overrides {
    for (key, val) in overrides {
      context.insert(key.clone(), val.clone());
    }
  }

  Ok(())
}
