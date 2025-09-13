use std::{
  env::{self, current_dir},
  fs::{create_dir_all, File},
  io::ErrorKind,
  path::PathBuf,
};

use indexmap::IndexMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tera::{Context, Tera};

use crate::{config::Config, GenError};

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
/// The output directory will be joined to the root of the package being generated with this template.
/// The context specified here will override the global context.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, JsonSchema)]
pub struct TemplateOutput {
  pub template: TemplateData,
  pub output: String,
  #[serde(default)]
  pub context: IndexMap<String, Value>,
}

pub(crate) fn get_default_context() -> Context {
  let mut context = Context::default();

  context.insert("__cwd", &current_dir().expect("Could not get the cwd"));

  macro_rules! add_env_to_context {
    ($name:ident, $env_name:ident) => {
      paste::paste! {
        if let Ok($name) = env::var(stringify!($env_name)) {
          context.insert(concat!("__", stringify!($name)), &$name);
        }
      }
    };

    ($name:ident) => {
      paste::paste! {
        if let Ok($name) = env::var(stringify!([< $name:upper >])) {
          context.insert(concat!("__", stringify!($name)), &$name);
        }
      }
    };
  }

  add_env_to_context!(os);
  add_env_to_context!(user);
  add_env_to_context!(home);
  add_env_to_context!(hostname);
  add_env_to_context!(arch, HOSTTYPE);
  add_env_to_context!(xdg_config, XDG_CONFIG_HOME);
  add_env_to_context!(xdg_data, XDG_DATA_HOME);
  add_env_to_context!(xdg_cache, XDG_CACHE_HOME);
  add_env_to_context!(xdg_state, XDG_STATE_HOME);

  context.insert("__tmp_dir", &env::temp_dir());

  context
}

#[cfg(feature = "uuid")]
fn tera_uuid(
  _: &std::collections::HashMap<String, tera::Value>,
) -> Result<tera::Value, tera::Error> {
  Ok(uuid::Uuid::new_v4().to_string().into())
}

impl Config {
  pub fn initialize_tera(&self) -> Result<Tera, GenError> {
    let mut tera = if let Some(templates_dir) = &self.templates_dir {
      Tera::new(&format!("{}/**/*", templates_dir.display()))
        .map_err(|e| GenError::TemplateDirLoading { source: e })?
    } else {
      Tera::default()
    };

    tera.autoescape_on(vec![]);

    #[cfg(feature = "uuid")]
    {
      tera.register_function("uuid", tera_uuid);
    }

    for (name, template) in &self.templates {
      tera
        .add_raw_template(name, template)
        .map_err(|e| GenError::TemplateParsing {
          template: name.to_string(),
          source: e,
        })?;
    }

    Ok(tera)
  }

  /// A helper to generate custom templates.
  pub fn generate_templates<T: Into<PathBuf>>(
    self,
    output_root: T,
    templates: Vec<TemplateOutput>,
  ) -> Result<(), GenError> {
    let mut tera = self.initialize_tera()?;

    let mut global_context = Context::from_serialize(self.global_templates_vars)
      .map_err(|e| GenError::TemplateContextParsing { source: e })?;

    global_context.extend(get_default_context());

    let output_root: PathBuf = output_root.into();

    for template in templates {
      let mut local_context = global_context.clone();

      if !template.context.is_empty() {
        let added_context = Context::from_serialize(template.context)
          .map_err(|e| GenError::TemplateContextParsing { source: e })?;

        local_context.extend(added_context);
      }

      let output_path = output_root.join(template.output);

      create_dir_all(output_path.parent().ok_or(GenError::Custom(format!(
        "Could not get the parent directory for '{}'",
        output_path.display()
      )))?)
      .map_err(|e| GenError::ParentDirCreation {
        path: output_path.clone(),
        source: e,
      })?;

      let mut output_file = if self.no_overwrite {
        File::create_new(&output_path).map_err(|e| match e.kind() {
          ErrorKind::AlreadyExists => GenError::FileExists {
            path: output_path.clone(),
          },
          _ => GenError::WriteError {
            path: output_path.clone(),
            source: e,
          },
        })?
      } else {
        File::create(&output_path).map_err(|e| GenError::FileCreation {
          path: output_path.clone(),
          source: e,
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
