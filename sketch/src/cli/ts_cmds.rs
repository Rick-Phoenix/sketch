use std::{collections::BTreeSet, path::PathBuf};

use askama::Template;
use clap::{Args, Subcommand};
use globset::{Glob, GlobSetBuilder};
use indexmap::IndexMap;
use merge::Merge;
use walkdir::WalkDir;

use crate::{
  cli::cli_elements::TemplateRef,
  custom_templating::{PresetElement, TemplatingPreset, TemplatingPresetReference},
  exec::launch_command,
  fs::{create_parent_dirs, get_cwd, open_file_if_overwriting},
  ts::{
    oxlint::OxlintConfigSetting,
    package::{PackageConfig, PackageData},
    pnpm::PnpmWorkspace,
    ts_monorepo::CreateTsMonorepoSettings,
    vitest::VitestConfigKind,
    PackageManager,
  },
  Config, GenError, *,
};

pub(crate) async fn handle_ts_commands(
  mut config: Config,
  command: TsCommands,
  cli_vars: &IndexMap<String, Value>,
) -> Result<(), GenError> {
  let overwrite = config.can_overwrite();
  let typescript = config.typescript.get_or_insert_default();

  match command {
    TsCommands::Barrel { args } => {
      create_ts_barrel(args, overwrite)?;
    }
    TsCommands::Monorepo {
      package_args: TsPackageArgs {
        oxlint,
        install,
        with_templates,
      },
      root_package_overrides,
      root_package,
      dir,
      pnpm,
    } => {
      let mut root_package = if let Some(id) = root_package {
        typescript
          .package_presets
          .get(&id)
          .ok_or(GenError::PresetNotFound {
            kind: Preset::TsPackage,
            name: id,
          })?
          .clone()
      } else {
        let mut package = PackageConfig::default();
        package.oxlint = Some(OxlintConfigSetting::Bool(true));
        package.name = Some("root".to_string());
        package
      };

      if let Some(overrides) = root_package_overrides {
        root_package.merge(overrides);
      }

      if let Some(id) = oxlint {
        root_package.oxlint = if id == "default" {
          Some(OxlintConfigSetting::Bool(true))
        } else {
          Some(OxlintConfigSetting::Id(id))
        };
      }

      let package_manager = typescript.package_manager.get_or_insert_default().clone();
      let out_dir = dir.unwrap_or_else(|| "ts_root".into());

      let pnpm_config = if let Some(id) = pnpm {
        Some(
          typescript
            .pnpm_presets
            .get(&id)
            .ok_or(GenError::PresetNotFound {
              kind: Preset::PnpmWorkspace,
              name: id.clone(),
            })?
            .clone()
            .process_data(id.as_str(), &typescript.pnpm_presets)?,
        )
      } else if matches!(package_manager, PackageManager::Pnpm) {
        Some(PnpmWorkspace::default())
      } else {
        None
      };

      if let Some(template_refs) = with_templates {
        let templates_list = root_package.with_templates.get_or_insert_default();

        let mut single_templates: Vec<PresetElement> = Vec::new();

        for template in template_refs {
          match template {
            TemplateRef::PresetId(id) => templates_list.push(TemplatingPresetReference::Preset {
              id,
              context: Default::default(),
            }),
            TemplateRef::Template(def) => single_templates.push(PresetElement::Template(def)),
          };
        }

        if !single_templates.is_empty() {
          templates_list.push(TemplatingPresetReference::Definition(TemplatingPreset {
            templates: single_templates,
            ..Default::default()
          }));
        }
      }

      config
        .create_ts_monorepo(CreateTsMonorepoSettings {
          root_package,
          out_dir: &out_dir,
          pnpm_config,
          cli_vars,
        })
        .await?;

      if install {
        launch_command(
          package_manager.to_string().as_str(),
          &["install"],
          &out_dir,
          Some("Could not install dependencies"),
        )?;
      }
    }
    TsCommands::Package {
      preset,
      package_config,
      update_tsconfig,
      dir,
      vitest,
      package_args: TsPackageArgs {
        oxlint,
        install,
        with_templates,
      },
    } => {
      let mut package = if let Some(preset) = preset {
        typescript
          .package_presets
          .get(&preset)
          .ok_or(GenError::PresetNotFound {
            kind: Preset::TsPackage,
            name: preset.clone(),
          })?
          .clone()
      } else {
        PackageConfig::default()
      };

      if let Some(overrides) = package_config {
        package.merge(overrides);
      }

      if let Some(vitest) = vitest {
        package.vitest = Some(VitestConfigKind::Id(vitest))
      }

      if let Some(id) = oxlint {
        package.oxlint = if id == "default" {
          Some(OxlintConfigSetting::Bool(true))
        } else {
          Some(OxlintConfigSetting::Id(id))
        };
      }

      if let Some(template_refs) = with_templates {
        let templates_list = package.with_templates.get_or_insert_default();

        let mut single_templates: Vec<PresetElement> = Vec::new();

        for template in template_refs {
          match template {
            TemplateRef::PresetId(id) => templates_list.push(TemplatingPresetReference::Preset {
              id,
              context: Default::default(),
            }),
            TemplateRef::Template(def) => single_templates.push(PresetElement::Template(def)),
          };
        }

        if !single_templates.is_empty() {
          templates_list.push(TemplatingPresetReference::Definition(TemplatingPreset {
            templates: single_templates,
            ..Default::default()
          }));
        }
      }

      let package_dir =
        dir.unwrap_or_else(|| package.name.as_deref().unwrap_or("new_package").into());

      if install {
        let package_manager = typescript.package_manager.get_or_insert_default().clone();

        launch_command(
          package_manager.to_string().as_str(),
          &["install"],
          &package_dir,
          Some("Could not install dependencies"),
        )?;
      }

      config
        .build_package(
          PackageData::Config(package),
          package_dir,
          update_tsconfig,
          cli_vars,
        )
        .await?;
    }
  }

  Ok(())
}

#[derive(Subcommand, Debug, Clone)]
pub enum TsCommands {
  /// Generates a new typescript monorepo
  Monorepo {
    /// The root directory for the new monorepo. [default: `ts_root`].
    dir: Option<PathBuf>,

    /// The `pnpm-workspace.yaml` preset to use for the new monorepo. If it's unset and `pnpm` is the chosen package manager, the default preset will be used.
    #[arg(short, long, value_name = "PRESET_ID")]
    pnpm: Option<String>,

    /// The id of the package preset to use for the root package. If unset, the default preset is used, along with the values set via cli flags.
    #[arg(short, long, value_name = "PRESET_ID")]
    root_package: Option<String>,

    #[command(flatten)]
    root_package_overrides: Option<PackageConfig>,

    #[command(flatten)]
    package_args: TsPackageArgs,
  },

  /// Generates a new typescript package
  Package {
    /// The root directory for the new package. Defaults to the package name.
    dir: Option<PathBuf>,

    /// The package preset to use. If unset, the default preset is used, along with the values set via cli flags
    #[arg(short, long, value_name = "ID")]
    preset: Option<String>,

    /// An optional list of tsconfig files where the new tsconfig file will be added as a reference.
    #[arg(short, long)]
    update_tsconfig: Option<Vec<PathBuf>>,

    #[command(flatten)]
    package_args: TsPackageArgs,

    /// The vitest preset to use. It can be set to `default` to use the default preset.
    #[arg(long, value_name = "ID")]
    vitest: Option<String>,

    #[command(flatten)]
    package_config: Option<PackageConfig>,
  },

  /// Creates a barrel file
  Barrel {
    #[command(flatten)]
    args: TsBarrelArgs,
  },
}

#[derive(Debug, Clone, Args)]
pub struct TsPackageArgs {
  /// The oxlint preset to use. It can be set to `default` to use the default preset.
  #[arg(long, value_name = "ID")]
  oxlint: Option<String>,

  /// Installs the dependencies with the chosen package manager
  #[arg(short, long)]
  install: bool,

  /// One or many templates or templating presets to generate in the new package's root
  #[arg(
    short,
    long = "with-template",
    value_name = "PRESET_ID|id=TEMPLATE_ID,output=PATH"
  )]
  with_templates: Option<Vec<TemplateRef>>,
}

#[derive(Debug, Clone, Args)]
pub struct TsBarrelArgs {
  /// The directory where to search recursively for the files and generate the barrel file [default: `.`]
  pub dir: Option<PathBuf>,

  /// The output path for the barrel file. It defaults to `{dir}/index.ts`
  #[arg(short, long)]
  pub output: Option<PathBuf>,

  /// The file extensions that should be kept in export statements.
  #[arg(long = "keep-ext", value_name = "EXT")]
  pub keep_extensions: Vec<String>,

  /// Exports `.ts` files as `.js`. It assumes that `js` is among the file extensions to keep.
  #[arg(long)]
  pub js_ext: bool,

  /// One or more glob patterns to exclude from the imported modules.
  #[arg(long)]
  pub exclude: Option<Vec<String>>,
}

#[derive(Debug, Template)]
#[template(path = "ts/barrel.ts.j2")]
struct BarrelFile {
  pub files: BTreeSet<PathBuf>,
}

const JS_EXTENSIONS: &[&str] = &["vue", "svelte", "jsx", "tsx", "ts", "js"];

fn create_ts_barrel(args: TsBarrelArgs, overwrite: bool) -> Result<(), GenError> {
  let TsBarrelArgs {
    dir,
    keep_extensions,
    exclude,
    js_ext,
    output,
  } = args;

  let dir = dir.unwrap_or_else(|| get_cwd());

  if !dir.is_dir() {
    return Err(generic_error!("`{:?}` is not a directory", dir));
  }

  let mut glob_builder = GlobSetBuilder::new();

  glob_builder.add(Glob::new("index.ts").unwrap());

  if let Some(ref patterns) = exclude {
    for pattern in patterns {
      glob_builder.add(
        Glob::new(pattern)
          .map_err(|e| generic_error!("Could not parse glob pattern `{}`: {}", pattern, e))?,
      );
    }
  }

  let globset = glob_builder
    .build()
    .map_err(|e| generic_error!("Could not build globset: {}", e))?;

  let mut paths: BTreeSet<PathBuf> = BTreeSet::new();

  for entry in WalkDir::new(&dir)
    .into_iter()
    .filter_map(|e| e.ok())
    .filter(|e| e.file_type().is_file())
  {
    let mut path = entry.path().strip_prefix(&dir).unwrap().to_path_buf();

    let extension = if let Some(ext) = path.extension() {
      ext.to_string_lossy().to_string()
    } else {
      continue;
    };

    if !JS_EXTENSIONS.contains(&extension.as_str()) || globset.is_match(&path) {
      continue;
    }

    if js_ext && (extension == "js" || extension == "ts") {
      path = path.with_extension("js");
    } else if !keep_extensions.contains(&extension) {
      path = path.with_extension("");
    }

    paths.insert(path);
  }

  let out_file = output.unwrap_or_else(|| dir.join("index.ts"));

  create_parent_dirs(&out_file)?;

  let mut file = open_file_if_overwriting(overwrite, &out_file)?;

  let barrel = BarrelFile { files: paths };

  barrel
    .write_into(&mut file)
    .map_err(|e| GenError::WriteError {
      path: out_file,
      source: e,
    })?;

  Ok(())
}
