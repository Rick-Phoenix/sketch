use super::*;

impl Config {
	/// A helper to generate custom templates.
	pub fn generate_templates(
		&self,
		output_root: impl AsRef<Path>,
		preset_refs: Vec<TemplatingPresetReference>,
		cli_overrides: &IndexMap<String, Value>,
	) -> Result<(), GenError> {
		let output_root = output_root.as_ref();
		let overwrite = self.can_overwrite();

		let mut tera = self.initialize_tera()?;
		let mut global_context = create_context(&self.vars)?;
		global_context.extend(get_default_context());

		let global_context = Arc::new(global_context);

		for preset_ref in preset_refs {
			let (templates, preset_context) = match preset_ref {
				TemplatingPresetReference::Preset {
					preset_id: id,
					context,
				} => (
					self.templating_presets
						.get(&id)
						.ok_or(GenError::PresetNotFound {
							kind: PresetKind::Templates,
							name: id.clone(),
						})?
						.clone()
						.merge_presets(id.as_str(), &self.templating_presets)?
						.templates,
					context,
				),
				TemplatingPresetReference::Definition(preset) => {
					let preset_full =
						preset.merge_presets("__inlined", &self.templating_presets)?;

					(preset_full.templates, preset_full.context)
				}
			};

			let mut local_context = get_local_context(
				ContextRef::Original(global_context.clone()),
				&preset_context,
			);

			for element in templates {
				match element {
					TemplateKind::Remote(RemotePreset { repo, exclude }) => {
						local_context = get_local_context(local_context, cli_overrides);

						let tmp_dir = env::temp_dir().join("sketch/repo");

						if tmp_dir.exists() {
							remove_dir_all(&tmp_dir).with_context(|| {
								format!("Could not empty the directory `{}`", tmp_dir.display())
							})?;
						}

						let clone_result = Command::new("git")
							.arg("clone")
							.arg("--depth=1")
							.arg(&repo)
							.arg(&tmp_dir)
							.output()
							.with_context(|| format!("Could not clone git repo `{repo}`"))?;

						if !clone_result.status.success() {
							let stderr = String::from_utf8_lossy(&clone_result.stderr);
							return Err(
								anyhow!("Could not clone git repo `{repo}`: {stderr}",).into()
							);
						}

						remove_dir_all(tmp_dir.join(".git")).with_context(|| {
							format!("Could not empty the directory `{}`", tmp_dir.display())
						})?;

						let new_tera = Tera::new(&format!("{}/**/*", tmp_dir.display()))
							.with_context(|| {
								format!(
									"Failed to load the templates from remote template `{repo}`"
								)
							})?;

						tera.extend(&new_tera).with_context(|| {
							format!("Failed to load the templates from remote template `{repo}`")
						})?;

						render_structured_preset(
							overwrite,
							&tera,
							local_context.as_ref(),
							output_root,
							&tmp_dir,
							&tmp_dir,
							&exclude,
						)?;
					}
					TemplateKind::Single(template) => {
						local_context = get_local_context(local_context, cli_overrides);

						render_template_with_output(
							overwrite,
							&mut tera,
							local_context.as_ref(),
							output_root,
							template.output,
							&template.template,
						)?;
					}

					TemplateKind::Structured(StructuredPreset { dir, exclude }) => {
						local_context = get_local_context(local_context, cli_overrides);

						render_structured_preset(
							overwrite,
							&tera,
							local_context.as_ref(),
							output_root,
							&dir,
							self.templates_dir
								.as_ref()
								.context("templates_dir not set")?,
							&exclude,
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
	template: &TemplateRef,
) -> Result<(), GenError> {
	let template_name = template.name();

	if let TemplateRef::Inline { name, content } = &template {
		tera.add_raw_template(name, content)
			.map_err(|e| GenError::TemplateParsing {
				template: name.clone(),
				source: e,
			})?;
	}

	match output {
		TemplateOutputKind::Stdout => {
			let output =
				tera.render(template_name, context)
					.map_err(|e| GenError::TemplateRendering {
						template: template_name.to_string(),
						source: e,
					})?;

			println!("{output}");
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
	exclude: &[String],
) -> Result<(), GenError> {
	let templates_dir = get_abs_path(templates_dir)?;
	let root_dir = templates_dir.join(dir);
	if !root_dir.is_dir() {
		return Err(anyhow!(format!(
			"`{}` is not a valid directory inside `{}`",
			dir.display(),
			templates_dir.display()
		))
		.into());
	}

	let exclude_glob = if exclude.is_empty() {
		None
	} else {
		let mut glob_builder = GlobSetBuilder::new();

		for pattern in exclude {
			glob_builder.add(
				Glob::new(pattern)
					.with_context(|| format!("Could not parse glob pattern `{pattern}`"))?,
			);
		}

		Some(
			glob_builder
				.build()
				.context("Could not build globset")?,
		)
	};

	let _: () = for entry in WalkDir::new(&root_dir)
		.into_iter()
		.filter_map(|e| e.ok())
	{
		let template_path_from_templates_dir = entry
			.path()
			.strip_prefix(&templates_dir)
			.context("`dir` must be a directory inside `templates_dir`")?;
		let mut output_path_from_root_dir = entry
			.path()
			.strip_prefix(&root_dir)
			.context("`dir` must be a directory inside `templates_dir`")?
			.to_path_buf();

		if output_path_from_root_dir
			.to_string_lossy()
			.is_empty()
		{
			continue;
		}

		if let Some(ref globset) = exclude_glob
			&& globset.is_match(template_path_from_templates_dir)
		{
			continue;
		}

		let file_type = entry.file_type();

		if file_type.is_dir() {
			create_all_dirs(entry.path())?;
			continue;
		} else if file_type.is_file() {
			if output_path_from_root_dir
				.extension()
				.is_some_and(|e| e == "j2" || e == "jinja" || e == "jinja2")
			{
				output_path_from_root_dir = output_path_from_root_dir.with_extension("");
			}

			render_template(
				tera,
				&template_path_from_templates_dir.to_string_lossy(),
				&output_root.join(output_path_from_root_dir),
				context,
				overwrite,
			)?;
		}
	};
	Ok(())
}

fn render_template(
	tera: &Tera,
	template_name: &str,
	output_path: &Path,
	context: &Context,
	overwrite: bool,
) -> Result<(), GenError> {
	create_all_dirs(get_parent_dir(output_path)?)?;

	let mut output_file = open_file_if_overwriting(overwrite, output_path)?;

	tera.render_to(template_name, context, &mut output_file)
		.map_err(|e| GenError::TemplateRendering {
			template: template_name.to_string(),
			source: e,
		})
}

pub(crate) fn create_context(context: &IndexMap<String, Value>) -> Result<Context, GenError> {
	Context::from_serialize(context).map_err(|e| GenError::TemplateContextParsing { source: e })
}

pub(crate) fn get_local_context(
	initial_context: ContextRef,
	overrides: &IndexMap<String, Value>,
) -> ContextRef {
	if overrides.is_empty() {
		initial_context
	} else {
		let mut context = match initial_context {
			ContextRef::Original(context) => (*context).clone(),
			ContextRef::New(context) => context,
		};

		for (key, val) in overrides {
			context.insert(key, val);
		}

		ContextRef::New(context)
	}
}

#[derive(Debug, Clone)]
pub(crate) enum ContextRef {
	Original(Arc<Context>),
	New(Context),
}

impl AsRef<Context> for ContextRef {
	fn as_ref(&self) -> &Context {
		match self {
			Self::Original(ctx) => ctx.as_ref(),
			Self::New(ctx) => ctx,
		}
	}
}
