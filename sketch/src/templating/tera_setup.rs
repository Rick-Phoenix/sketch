use std::env::{self};

use tera::{Context, Tera};

use crate::{
  cli::parsers::parse_key_value_pairs,
  config::Config,
  custom_templating::{TemplateData, TemplateOutput, TemplateOutputKind},
  fs::get_cwd,
  tera_filters::{
    absolute, basename, camel, capture, capture_many, glob, is_absolute, is_dir, is_file,
    is_relative, matches_glob, matches_semver, parent_dir, pascal, read_dir, relative, semver,
    snake, strip_prefix, strip_suffix, to_toml, to_yaml, upper_snake,
  },
  tera_functions::tera_uuid,
  GenError,
};

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

    let output = output.ok_or("Missing template output from command")?;
    let template = template.ok_or("Missing template id from command")?;

    Ok(TemplateOutput {
      template,
      output: TemplateOutputKind::Path(output.into()),
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

    tera.register_function("uuid", tera_uuid);

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
    tera.register_filter("is_absolute", is_absolute);
    tera.register_filter("is_relative", is_relative);
    tera.register_filter("relative", relative);
    tera.register_filter("absolute", absolute);
    tera.register_filter("strip_prefix", strip_prefix);
    tera.register_filter("strip_suffix", strip_suffix);
    tera.register_filter("read_dir", read_dir);
    tera.register_filter("glob", glob);
    tera.register_filter("matches_glob", matches_glob);
    tera.register_filter("to_yaml", to_yaml);
    tera.register_filter("to_toml", to_toml);

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
    format!("sketch_{}", name),
    &get_env(vars).unwrap_or_else(|| "unknown".to_string()),
  );
}

/// Test if the program is running under WSL
#[cfg(target_os = "linux")]
pub fn is_wsl() -> bool {
  if let Ok(b) = std::fs::read("/proc/sys/kernel/osrelease")
    && let Ok(s) = std::str::from_utf8(&b) {
      let a = s.to_ascii_lowercase();
      return a.contains("microsoft") || a.contains("wsl");
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
  context.insert("sketch_is_macos", &cfg!(target_os = "macos"));
  context.insert("sketch_is_linux", &cfg!(target_os = "linux"));
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
