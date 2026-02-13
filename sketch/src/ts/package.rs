use super::{vitest::*, *};
use crate::exec::*;

/// The kind of ts package. Only relevant when using defaults.
#[derive(Debug, Deserialize, Serialize, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum PackageKind {
	#[default]
	Library,
	App,
}

/// The configuration struct that is used to generate new packages.
#[derive(Clone, Debug, Deserialize, Serialize, Parser, Merge, PartialEq, Eq, Default)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct TsPackagePreset {
	/// The name of the new package. It defaults to the name of its directory.
	#[arg(short, long)]
	pub name: Option<String>,

	/// A list of [`TsConfigDirective`]s for this package. They can be preset ids or literal configurations.
	#[arg(long, value_parser = TsConfigDirective::from_cli)]
	#[arg(
		help = "One or many tsconfig presets (with their output path) to use for this package (uses defaults if not provided)",
		value_name = "id=ID,output=PATH"
	)]
	pub ts_config: Vec<TsConfigDirective>,

	/// The [`PackageJsonData`] to use for this package. It can be a preset id or a literal definition (or nothing, to use defaults).
	#[arg(long, value_parser = PackageJsonPresetRef::from_cli)]
	#[arg(
		help = "The package.json preset ID to use (uses defaults if not provided)",
		value_name = "ID"
	)]
	pub package_json: Option<PackageJsonPresetRef>,

	/// A license file to generate for the new package.
	#[arg(long)]
	pub license: Option<License>,

	/// One or many templates to generate along with this package. Relative output paths will resolve from the root of the package.
	#[arg(long = "template", short = 't', value_name = "PRESET_ID")]
	pub with_templates: Vec<TemplatingPresetReference>,

	/// One or many rendered commands to execute before the repo's creation
	#[arg(
		long = "hook-pre",
		help = "One or many IDs of templates to render and execute as commands before the package's creation",
		value_name = "ID"
	)]
	pub hooks_pre: Vec<Hook>,

	/// One or many rendered commands to execute after the repo's creation
	#[arg(
		long = "hook-post",
		help = "One or many IDs of templates to render and execute as commands after the package's creation",
		value_name = "ID"
	)]
	pub hooks_post: Vec<Hook>,

	/// The configuration for this package's vitest setup. It can be set to `true` (to use defaults), to a preset id, or to a literal configuration.
	#[arg(skip)]
	pub vitest: Option<VitestPresetRef>,

	/// The configuration for this package's oxlint setup. It can be set to `true` (to use defaults), to a preset id, or to a literal configuration.
	#[arg(long, value_name = "ID")]
	pub oxlint: Option<OxlintConfigSetting>,
}

/// The kinds of Ts package data. Either an id pointing to a stored preset, or a custom configuration.
pub enum TsPackagePresetRef {
	Preset(String),
	Config(TsPackagePreset),
}

pub enum PackageType {
	MonorepoRoot { pnpm: Option<PnpmWorkspace> },
	Normal,
}

impl PackageType {
	/// Returns `true` if the package kind is [`MonorepoRoot`].
	///
	/// [`MonorepoRoot`]: PackageType::MonorepoRoot
	#[must_use]
	pub(crate) const fn is_monorepo_root(&self) -> bool {
		matches!(self, Self::MonorepoRoot { .. })
	}
}

pub struct TsPackageSetup<'a> {
	pub data: TsPackagePresetRef,
	pub pkg_root: &'a Path,
	pub tsconfig_files_to_update: Vec<PathBuf>,
	pub cli_vars: &'a IndexMap<String, Value>,
	pub package_type: PackageType,
	pub install: bool,
}

impl Config {
	/// Generates a new typescript package.
	pub async fn create_ts_package(mut self, setup: TsPackageSetup<'_>) -> Result<(), AppError> {
		let TsPackageSetup {
			data,
			pkg_root,
			tsconfig_files_to_update,
			cli_vars,
			mut package_type,
			install,
		} = setup;

		let overwrite = self.can_overwrite();

		let typescript = mem::take(&mut self.typescript).unwrap_or_default();

		let package_manager = typescript.package_manager.unwrap_or_default();
		let version_ranges = typescript.version_range.unwrap_or_default();

		let package_config = match data {
			TsPackagePresetRef::Config(conf) => conf,
			TsPackagePresetRef::Preset(id) => typescript
				.package_presets
				.get(&id)
				.ok_or_else(|| AppError::PresetNotFound {
					kind: PresetKind::TsPackage,
					name: id.clone(),
				})?
				.clone(),
		};

		let package_name = if let Some(name) = package_config.name.as_ref() {
			name.clone()
		} else {
			pkg_root
				.file_name()
				.expect("Failed to detect dirname")
				.to_string_lossy()
				.to_string()
		};

		create_all_dirs(pkg_root)?;

		// We must get the absolute path after creation
		let pkg_root = get_abs_path(pkg_root)?;

		if !package_config.hooks_pre.is_empty() {
			self.execute_command(
				self.shell.as_deref(),
				&pkg_root,
				package_config.hooks_pre,
				cli_vars,
				false,
			)?;
		}

		let mut package_json_data = match package_config.package_json.unwrap_or_default() {
			PackageJsonPresetRef::Id(id) => typescript.get_package_json(&id)?,
			PackageJsonPresetRef::Config(preset) => preset.process_data(
				"__inlined",
				&typescript.package_json_presets,
				&typescript.people,
			)?,
		};

		if package_json_data.package_manager.is_none() {
			package_json_data.package_manager = Some(package_manager.to_string());
		}

		package_json_data.name = Some(package_name.clone());

		if !typescript.no_default_deps.unwrap_or_default() {
			let mut default_deps = vec!["typescript", "oxlint"];

			if let Some(ref vitest) = package_config.vitest
				&& vitest.is_enabled()
			{
				default_deps.push("vitest")
			}

			for dep in default_deps {
				if !package_json_data
					.dev_dependencies
					.contains_key(dep)
				{
					let version = if typescript.catalog.unwrap_or_default() {
						"catalog:".to_string()
					} else {
						"latest".to_string()
					};

					package_json_data
						.dev_dependencies
						.insert(dep.to_string(), version);
				}
			}
		}

		#[cfg(feature = "npm-version")]
		{
			let convert_latest = !typescript
				.no_convert_latest_to_range
				.unwrap_or_default();

			process_package_json_dependencies(
				&mut package_json_data,
				package_manager,
				convert_latest,
				version_ranges,
			)
			.await?;
		}

		if let Some(workspaces) = &package_json_data.workspaces {
			for path in workspaces {
				create_dirs_from_stripped_glob(&pkg_root.join(path))?;
			}
		}

		serialize_json(
			&package_json_data,
			&pkg_root.join("package.json"),
			overwrite,
		)?;

		if let PackageType::MonorepoRoot { pnpm } = &mut package_type
			&& let Some(pnpm_data) = pnpm
		{
			for dir in &pnpm_data.packages {
				create_dirs_from_stripped_glob(&pkg_root.join(dir))?;
			}

			#[cfg(feature = "npm-version")]
			pnpm::add_dependencies_to_catalog(pnpm_data, version_ranges, &package_json_data)
				.await?;

			serialize_yaml(&pnpm_data, &pkg_root.join("pnpm-workspace.yaml"), overwrite)?;
		}

		#[cfg(feature = "npm-version")]
		if !package_type.is_monorepo_root()
			&& typescript.catalog.unwrap_or_default()
			&& matches!(package_manager, PackageManager::Pnpm)
		{
			let pnpm_workspace_path =
				find_file_up(&pkg_root, "pnpm-workspace.yaml").with_context(|| {
					format!(
						"Could not find a `pnpm-workspace.yaml` file while searching upwards from `{}`",
						pkg_root.display()
					)
				})?;

			let mut pnpm_workspace: PnpmWorkspace = deserialize_yaml(&pnpm_workspace_path)?;

			pnpm::add_dependencies_to_catalog(
				&mut pnpm_workspace,
				version_ranges,
				&package_json_data,
			)
			.await?;

			serialize_yaml(&pnpm_workspace, &pnpm_workspace_path, overwrite)?;
		}

		let mut tsconfig_files: Vec<(String, TsConfig)> = Default::default();

		if !package_config.ts_config.is_empty() {
			for directive in package_config.ts_config {
				let tsconfig_data = match directive.config.unwrap_or_default() {
					TsConfigKind::Id(id) => typescript.get_tsconfig_preset(&id)?.config,
					TsConfigKind::Config(ts_config) => {
						ts_config
							.merge_presets("__inlined", &typescript.ts_config_presets)?
							.config
					}
				};

				tsconfig_files.push((
					directive
						.output
						.unwrap_or_else(|| "tsconfig.json".to_string()),
					tsconfig_data,
				));
			}
		} else if package_type.is_monorepo_root() {
			let root_tsconfig_name = "tsconfig.options.json".to_string();

			let root_tsconfig_plain = TsConfig {
				extends: Some(root_tsconfig_name.clone()),
				files: btreeset![],
				references: btreeset![],
				..Default::default()
			};

			tsconfig_files.push(("tsconfig.json".to_string(), root_tsconfig_plain));

			let tsconfig_options = get_default_root_tsconfig();

			tsconfig_files.push((root_tsconfig_name, tsconfig_options));
		} else {
			tsconfig_files.push(("tsconfig.json".to_string(), get_default_package_tsconfig()));
		}

		for (file, tsconfig) in tsconfig_files {
			serialize_json(&tsconfig, &pkg_root.join(file), overwrite)?;
		}

		for path in tsconfig_files_to_update {
			let mut tsconfig: TsConfig = deserialize_json(&path)?;

			let path_to_new_tsconfig = get_relative_path(&path, &pkg_root.join("tsconfig.json"))?;

			tsconfig
				.references
				.insert(ts_config::TsConfigReference {
					path: path_to_new_tsconfig.to_string_lossy().to_string(),
				});

			serialize_json(&tsconfig, &path, true)?;
		}

		if !package_type.is_monorepo_root() {
			create_all_dirs(&pkg_root.join("src"))?;
		}

		if let Some(vitest_config) = package_config.vitest
			&& vitest_config.is_enabled()
		{
			let mut vitest = match vitest_config {
				VitestPresetRef::Bool(_) => VitestPreset::default(),
				VitestPresetRef::Id(id) => typescript.get_vitest_preset(&id)?,
				VitestPresetRef::Config(preset) => preset,
			};

			let tests_dir = pkg_root.join(&vitest.tests_dir);
			let tests_setup_dir = tests_dir.join(&vitest.setup_dir);

			create_all_dirs(&tests_dir)?;
			create_all_dirs(&tests_setup_dir)?;

			let file_parent_dir = vitest
				.out_dir
				.as_deref()
				.unwrap_or(tests_dir.as_path());

			let file_path = file_parent_dir.join("vitest.config.ts");

			vitest.setup_dir = get_relative_path(file_parent_dir, &tests_setup_dir)?
				.to_string_lossy()
				.to_string();

			let mut context = tera::Context::new();

			context.insert("config", &vitest);

			let src_rel_path = get_relative_path(file_parent_dir, &pkg_root.join("src"))?;

			context.insert("src_rel_path", &src_rel_path);

			let template = read_to_string(templates_dir().join("ts/vitest.config.ts.j2"))
				.context("Failed to read vitest template")?;

			let output = tera::Tera::one_off(&template, &context, false)
				.context("Failed to render vitest template")?;

			write_file(&file_path, &output, self.can_overwrite())?;

			let test_setup_file = read_to_string(templates_dir().join("ts/tests_setup.ts.j2"))
				.context("Failed to read tests setup template")?;

			write_file(
				&tests_setup_dir.join("tests_setup.ts"),
				&test_setup_file,
				true,
			)?;
		}

		if let Some(oxlint_config) = package_config.oxlint
			&& oxlint_config.is_enabled()
		{
			let oxlint_config = match oxlint_config {
				OxlintConfigSetting::Bool(_) => OxlintConfig::default(),
				OxlintConfigSetting::Id(id) => typescript.get_oxlint_preset(&id)?.config,
				OxlintConfigSetting::Config(oxlint_preset) => {
					oxlint_preset
						.merge_presets(
							&format!("__inlined_definition_{package_name}"),
							&typescript.oxlint_presets,
						)?
						.config
				}
			};

			serialize_json(&oxlint_config, &pkg_root.join(".oxlintrc.json"), overwrite)?;
		}

		if let Some(license) = package_config.license {
			write_file(&pkg_root.join("LICENSE"), license.get_content(), overwrite)?;
		}

		if !package_config.with_templates.is_empty() {
			self.generate_templates(&pkg_root, package_config.with_templates, cli_vars)?;
		}

		if !package_config.hooks_post.is_empty() {
			self.execute_command(
				self.shell.as_deref(),
				&pkg_root,
				package_config.hooks_post,
				cli_vars,
				false,
			)?;
		}

		if install {
			launch_command(
				package_manager.to_string().as_str(),
				&["install"],
				&pkg_root,
				Some("Could not install dependencies"),
			)?;
		}

		Ok(())
	}
}
