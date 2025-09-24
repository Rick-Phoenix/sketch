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
  tera_filters::{
    basename, camel, capture, capture_many, is_dir, is_file, matches_semver, parent_dir, pascal,
    semver, snake, upper_snake,
  },
  tera_functions::tera_uuid,
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

    tera.register_filter("basename", basename);
    tera.register_filter("parent_dir", parent_dir);
    tera.register_filter("capture", capture);
    tera.register_filter("capture_many", capture_many);
    tera.register_filter("is_file", is_file);
    tera.register_filter("is_dir", is_dir);
    tera.register_filter("semver", semver);
    tera.register_filter("matches_semver", matches_semver);
    tera.register_filter("camel", camel);
    tera.register_filter("snake", snake);
    tera.register_filter("upper_snake", upper_snake);
    tera.register_filter("pascal", pascal);

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

fn get_env(vars: &[&str]) -> Option<String> {
  for var in vars {
    if let Ok(value) = env::var(var) {
      return Some(value);
    }
  }
  None
}

fn get_env_with_fallback(context: &mut Context, name: &str, vars: &[&str]) {
  context.insert(
    &format!("sketch_{}", name),
    &get_env(vars).unwrap_or_else(|| "unknown".to_string()),
  );
}

/// Test if the program is running under WSL
#[cfg(target_os = "linux")]
pub fn is_wsl() -> bool {
  if let Ok(b) = std::fs::read("/proc/sys/kernel/osrelease") {
    if let Ok(s) = std::str::from_utf8(&b) {
      let a = s.to_ascii_lowercase();
      return a.contains("microsoft") || a.contains("wsl");
    }
  }
  false
}

/// Test if the program is running under WSL
#[cfg(not(target_os = "linux"))]
pub fn is_wsl() -> bool {
  false
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

  get_env_with_fallback(&mut context, "os", &["CARGO_CFG_TARGET_OS", "OS"]);
  get_env_with_fallback(&mut context, "os_family", &["CARGO_CFG_TARGET_FAMILY"]);
  get_env_with_fallback(&mut context, "arch", &["CARGO_CFG_TARGET_ARCH", "HOSTTYPE"]);

  context.insert("sketch_is_windows", &cfg!(windows));
  context.insert("sketch_is_unix", &cfg!(unix));
  context.insert("sketch_is_wsl", &is_wsl());

  add_env_to_context!(user);
  add_env_to_context!(hostname);
  add_env_to_context!(xdg_config, XDG_CONFIG_HOME);
  add_env_to_context!(xdg_data, XDG_DATA_HOME);
  add_env_to_context!(xdg_cache, XDG_CACHE_HOME);
  add_env_to_context!(xdg_state, XDG_STATE_HOME);

  context.insert("sketch_tmp_dir", &env::temp_dir());
  context.insert("sketch_home", &env::home_dir());

  context
}
