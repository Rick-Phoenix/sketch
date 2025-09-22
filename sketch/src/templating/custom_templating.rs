use std::{
  env::{self},
  path::Path,
};

use indexmap::IndexMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tera::{Context, Tera};

use crate::{
  cli::parsers::parse_key_value_pairs,
  config::Config,
  fs::{create_all_dirs, get_cwd, get_parent_dir, open_file_if_overwriting},
  GenError,
};

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
/// Relative output paths will resolve from the [`Config::out_dir`].
/// The context specified here will override the global context (but not the variables set via cli).
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, JsonSchema)]
pub struct TemplateOutput {
  pub template: TemplateData,
  pub output: String,
  #[serde(default)]
  pub context: IndexMap<String, Value>,
}

impl TemplateOutput {
  pub(crate) fn from_cli(s: &str) -> Result<Self, String> {
    let pairs = parse_key_value_pairs("TemplateOutput", s)?;

    let mut output: Option<String> = None;
    let mut template: Option<TemplateData> = None;

    for (key, val) in pairs {
      match key {
        "output" => {
          if !val.is_empty() {
            output = Some(val.to_string())
          }
        }
        "id" => {
          if !val.is_empty() {
            template = Some(TemplateData::Id(val.to_string()))
          }
        }
        _ => return Err(format!("Invalid key for TemplateOutput: {}", key)),
      };
    }

    let output = output.ok_or_else(|| "Missing template output from command")?;
    let template = template.ok_or_else(|| "Missing template id from command")?;

    Ok(TemplateOutput {
      template,
      output,
      context: Default::default(),
    })
  }
}

impl Config {
  pub(crate) fn initialize_tera(&self) -> Result<Tera, GenError> {
    let mut tera = if let Some(templates_dir) = &self.templates_dir {
      Tera::new(&format!("{}/**/*", templates_dir.display()))
        .map_err(|e| GenError::Custom(format!("Failed to load the templates directory: {}", e)))?
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
  pub fn generate_templates<T: AsRef<Path>>(
    self,
    output_root: T,
    templates: Vec<TemplateOutput>,
  ) -> Result<(), GenError> {
    let overwrite = !self.no_overwrite;
    let mut tera = self.initialize_tera()?;

    let mut global_context = Context::from_serialize(self.vars)
      .map_err(|e| GenError::TemplateContextParsing { source: e })?;

    global_context.extend(get_default_context());

    let output_root = output_root.as_ref();

    for template in templates {
      let mut local_context = global_context.clone();

      if !template.context.is_empty() {
        let added_context = Context::from_serialize(template.context)
          .map_err(|e| GenError::TemplateContextParsing { source: e })?;

        local_context.extend(added_context);
      }

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

        create_all_dirs(get_parent_dir(&output_path))?;

        let mut output_file = open_file_if_overwriting(overwrite, &output_path)?;

        tera
          .render_to(template_name, &local_context, &mut output_file)
          .map_err(|e| GenError::TemplateRendering {
            template: template_name.to_string(),
            source: e,
          })?
      }
    }

    Ok(())
  }
}

pub(crate) fn get_default_context() -> Context {
  let mut context = Context::default();

  context.insert("sketch_cwd", &get_cwd());

  macro_rules! add_env_to_context {
    ($name:ident, $env_name:ident) => {
      paste::paste! {
        if let Ok($name) = env::var(stringify!($env_name)) {
          context.insert(concat!("sketch_", stringify!($name)), &$name);
        }
      }
    };

    ($name:ident) => {
      paste::paste! {
        if let Ok($name) = env::var(stringify!([< $name:upper >])) {
          context.insert(concat!("sketch_", stringify!($name)), &$name);
        }
      }
    };
  }

  add_env_to_context!(os);
  add_env_to_context!(user);
  add_env_to_context!(hostname);
  add_env_to_context!(arch, HOSTTYPE);
  add_env_to_context!(xdg_config, XDG_CONFIG_HOME);
  add_env_to_context!(xdg_data, XDG_DATA_HOME);
  add_env_to_context!(xdg_cache, XDG_CACHE_HOME);
  add_env_to_context!(xdg_state, XDG_STATE_HOME);

  context.insert("sketch_tmp_dir", &env::temp_dir());
  context.insert("sketch_home", &env::home_dir());

  context
}

#[cfg(feature = "uuid")]
fn tera_uuid(
  _: &std::collections::HashMap<String, tera::Value>,
) -> Result<tera::Value, tera::Error> {
  Ok(uuid::Uuid::new_v4().to_string().into())
}
