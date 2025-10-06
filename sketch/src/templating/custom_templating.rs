use std::{
  env,
  fs::remove_dir_all,
  path::{Path, PathBuf},
  process::Command,
};

use globset::{Glob, GlobSetBuilder};
use indexmap::{IndexMap, IndexSet};
use merge::Merge;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tera::{Context, Tera};
use walkdir::WalkDir;

use crate::{
  config::Config,
  fs::{create_all_dirs, get_abs_path, get_parent_dir, open_file_if_overwriting},
  merge_index_maps, merge_index_sets, merge_presets, merge_vecs,
  tera_setup::get_default_context,
  Extensible, GenError, Preset,
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

impl TemplatingPresetReference {
  pub fn get_context(&self) -> &IndexMap<String, Value> {
    match self {
      TemplatingPresetReference::Preset { context, .. } => context,
      TemplatingPresetReference::Definition(templating_preset) => &templating_preset.context,
    }
  }
}

/// A templating preset. It stores information about one or many templates, such as their source, output paths and contextual variables.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, JsonSchema, Default, Merge)]
#[serde(default)]
pub struct TemplatingPreset {
  /// The list of extended preset IDs.
  #[merge(strategy = merge_index_sets)]
  pub extends_presets: IndexSet<String>,
  /// The list of templates for this preset. Each element can be an individual template or a path to a directory inside `templates_dir` to render all the templates inside of it.
  #[merge(strategy = merge_vecs)]
  pub templates: Vec<PresetElement>,

  /// Additional context for the templates in this preset. It overrides previously set values, but not values set via the cli.
  #[merge(strategy = merge_index_maps)]
  pub context: IndexMap<String, Value>,
}

impl Extensible for TemplatingPreset {
  fn get_extended(&self) -> &IndexSet<String> {
    &self.extends_presets
  }
}

impl TemplatingPreset {
  pub fn process_data(
    self,
    id: &str,
    store: &IndexMap<String, TemplatingPreset>,
  ) -> Result<TemplatingPreset, GenError> {
    if self.extends_presets.is_empty() {
      return Ok(self);
    }

    let mut processed_ids: IndexSet<String> = IndexSet::new();

    let merged_preset = merge_presets(Preset::Templates, id, self, store, &mut processed_ids)?;

    Ok(merged_preset)
  }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum PresetElement {
  /// The data for a single template.
  Template(TemplateOutput),

  /// A path to a directory inside `templates_dir`, where all templates will be recursively extracted and rendered in the output directory, following the same file tree structure.
  Structured(StructuredPreset),

  /// A preset defined in a git repository.
  Remote(RemotePreset),
}

/// A preset defined in a git repository.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, JsonSchema)]
pub struct RemotePreset {
  /// The link of the repo where the preset is defined
  repo: String,
  /// A list of glob patterns for the templates to exclude
  exclude: Option<Vec<String>>,
}

/// A structured preset. It points to a directory within `templates_dir`, and optionally adds additional context. All of the templates inside the specified directory will be recursively rendered in the destination directory, with the same exact directory structure and names. If a template file ends with a `jinja` extension such as `.j2`, that gets stripped automatically.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, JsonSchema)]
pub struct StructuredPreset {
  /// A relative path to a directory starting from `templates_dir`
  dir: PathBuf,
  /// A list of glob patterns for the templates to exclude
  exclude: Option<Vec<String>>,
}

/// The types of configuration values for a template's data.
/// It can either be an id (which points to the key used to store a literal template in the config, or to a file path starting from the root of the templates directory specified in the config.)
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum TemplateData {
  /// A literal definition for a template.
  Content {
    /// The id of the newly created template. Mostly useful for organizational and debugging purposes.
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
  /// Render the output to stdout
  #[serde(skip)]
  Stdout,
  /// Render the output to a file
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
    &self,
    output_root: T,
    preset_refs: Vec<TemplatingPresetReference>,
    cli_overrides: &IndexMap<String, Value>,
  ) -> Result<(), GenError> {
    let overwrite = self.can_overwrite();
    let mut tera = self.initialize_tera()?;

    let mut global_context = Context::from_serialize(&self.vars)
      .map_err(|e| GenError::TemplateContextParsing { source: e })?;

    global_context.extend(get_default_context());

    let output_root = output_root.as_ref();

    for preset_ref in preset_refs {
      let mut local_context = global_context.clone();

      extend_context(&mut local_context, preset_ref.get_context())?;

      let preset = match preset_ref {
        TemplatingPresetReference::Preset { id, .. } => self
          .templating_presets
          .get(&id)
          .ok_or(GenError::PresetNotFound {
            kind: Preset::Templates,
            name: id.clone(),
          })?
          .clone()
          .process_data(id.as_str(), &self.templating_presets)?,
        TemplatingPresetReference::Definition(preset) => preset,
      };

      for element in preset.templates {
        match element {
          PresetElement::Remote(RemotePreset { repo, exclude }) => {
            apply_cli_overrides(&mut local_context, cli_overrides)?;

            let tmp_dir = env::temp_dir().join("sketch/repo");

            if tmp_dir.exists() {
              remove_dir_all(&tmp_dir).map_err(|e| {
                generic_error!("Could not empty the directory `{tmp_dir:?}`: {}", e)
              })?;
            }

            let clone_result = Command::new("git")
              .arg("clone")
              .arg(&repo)
              .arg(&tmp_dir)
              .output()
              .map_err(|e| generic_error!("Could not clone git repo `{}`: {}", repo, e))?;

            if !clone_result.status.success() {
              let stderr = String::from_utf8_lossy(&clone_result.stderr);
              return Err(generic_error!(
                "Could not clone git repo `{}`: {}",
                repo,
                stderr
              ));
            }

            remove_dir_all(tmp_dir.join(".git"))
              .map_err(|e| generic_error!("Could not empty the directory `{tmp_dir:?}`: {}", e))?;

            let new_tera = Tera::new(&format!("{}/**/*", tmp_dir.display())).map_err(|e| {
              GenError::Custom(format!(
                "Failed to load the templates from remote template `{}`: {}",
                repo, e
              ))
            })?;

            tera.extend(&new_tera).map_err(|e| {
              GenError::Custom(format!(
                "Failed to load the templates from remote template `{}`: {}",
                repo, e
              ))
            })?;

            render_structured_preset(
              overwrite,
              &tera,
              &local_context,
              output_root,
              &tmp_dir,
              &tmp_dir,
              exclude,
            )?;
          }
          PresetElement::Template(template) => {
            extend_context(&mut local_context, &template.context)?;
            apply_cli_overrides(&mut local_context, cli_overrides)?;

            render_template_with_output(
              overwrite,
              &mut tera,
              &local_context,
              output_root,
              template.output,
              template.template,
            )?;
          }

          PresetElement::Structured(StructuredPreset { dir, exclude }) => {
            apply_cli_overrides(&mut local_context, cli_overrides)?;

            render_structured_preset(
              overwrite,
              &tera,
              &local_context,
              output_root,
              &dir,
              &self
                .templates_dir
                .as_ref()
                .ok_or(GenError::Custom(format!("templates_dir not set")))?,
              exclude,
            )?;
          }
        };
      }
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
  dir: &Path,
  templates_dir: &Path,
  exclude: Option<Vec<String>>,
) -> Result<(), GenError> {
  let templates_dir = get_abs_path(templates_dir)?;
  let root_dir = templates_dir.join(&dir);
  if !root_dir.is_dir() {
    return Err(GenError::Custom(format!(
      "`{}` is not a valid directory inside `{}`",
      dir.display(),
      templates_dir.display()
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
    for entry in WalkDir::new(&root_dir).into_iter().filter_map(|e| e.ok()) {
      let input_path = entry
        .path()
        .strip_prefix(&templates_dir)
        .map_err(|_| generic_error!("`dir` must be a directory inside `templates_dir`"))?;
      let mut output_path = entry
        .path()
        .strip_prefix(&root_dir)
        .map_err(|_| generic_error!("`dir` must be a directory inside `templates_dir`"))?
        .to_path_buf();

      if output_path.to_string_lossy().is_empty() {
        continue;
      }

      if let Some(ref globset) = globset {
        if globset.is_match(&input_path) {
          continue;
        }
      }

      let file_type = entry.file_type();

      if file_type.is_dir() {
        create_all_dirs(&output_path)?;
        continue;
      } else if file_type.is_file() {
        if output_path
          .extension()
          .is_some_and(|e| e == "j2" || e == "jinja" || e == "jinja2")
        {
          output_path = output_path.with_extension("");
        }

        render_template(
          tera,
          &input_path.to_string_lossy(),
          &output_root.join(output_path),
          context,
          overwrite,
        )?;
      }
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

fn create_context(context: &IndexMap<String, Value>) -> Result<Context, GenError> {
  Context::from_serialize(context).map_err(|e| GenError::TemplateContextParsing { source: e })
}

fn extend_context(
  global_context: &mut Context,
  new_context: &IndexMap<String, Value>,
) -> Result<(), GenError> {
  if !new_context.is_empty() {
    let new_context = create_context(new_context)?;
    global_context.extend(new_context);
  }

  Ok(())
}

fn apply_cli_overrides(
  context: &mut Context,
  overrides: &IndexMap<String, Value>,
) -> Result<(), GenError> {
  for (key, val) in overrides {
    context.insert(key, val);
  }

  Ok(())
}
