use super::*;

pub(crate) struct RenderCtx<'a> {
	pub tera: &'a mut Tera,
	pub overwrite: bool,
	pub context: &'a Context,
	pub output_root: &'a Path,
}

impl Config {
	/// A helper to generate custom templates.
	pub fn generate_templates(
		&self,
		output_root: impl AsRef<Path>,
		preset_refs: Vec<TemplatingPresetReference>,
		cli_overrides: &IndexMap<String, Value>,
	) -> Result<(), AppError> {
		let output_root = output_root.as_ref();
		let overwrite = self.can_overwrite();

		let mut tera = self.initialize_tera()?;
		let mut global_context = create_context(&self.vars)?;
		global_context.extend(get_default_context());

		let mut template_context = TemplateContext::new(&global_context, cli_overrides);

		for preset_ref in preset_refs {
			let preset = match preset_ref {
				TemplatingPresetReference::Preset {
					preset_id: id,
					mut context,
				} => {
					let mut preset = self.get_templating_preset(&id)?;

					preset.context.append(&mut context);

					preset
				}
				TemplatingPresetReference::Definition(mut preset) => {
					if !preset.extends_presets.is_empty() {
						preset = preset.merge_presets("__inlined", &self.templating_presets)?;
					}

					preset
				}
			};

			template_context.apply_local_context(&preset.context);

			let mut render_ctx = RenderCtx {
				tera: &mut tera,
				overwrite,
				context: template_context.as_ref(),
				output_root,
			};

			for template in preset.templates {
				match template {
					TemplateKind::Remote(remote_preset) => {
						render_ctx.render_remote_preset(&remote_preset)?;
					}
					TemplateKind::Single(template) => {
						render_ctx.render_single_template(&template)?;
					}

					TemplateKind::Structured(StructuredPreset { dir, exclude }) => {
						render_ctx.render_structured_preset(
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

pub(crate) fn create_context(context: &IndexMap<String, Value>) -> Result<Context, AppError> {
	Ok(Context::from_serialize(context).context("Failed to parse the templating context")?)
}

pub(crate) struct TemplateContext<'a> {
	global_ctx: &'a Context,
	cli_overrides: &'a IndexMap<String, Value>,
	status: ContextStatus,
	local_ctx: Option<Context>,
}

#[derive(Clone, Copy)]
enum ContextStatus {
	NoCliOverrides,
	CliOverridesOnly,
	Dirty,
}

impl ContextStatus {
	/// Returns `true` if the context status is [`Dirty`].
	///
	/// [`Dirty`]: ContextStatus::Dirty
	#[must_use]
	const fn is_dirty(self) -> bool {
		matches!(self, Self::Dirty)
	}
}

impl AsRef<Context> for TemplateContext<'_> {
	fn as_ref(&self) -> &Context {
		if let Some(local_ctx) = &self.local_ctx {
			local_ctx
		} else {
			self.global_ctx
		}
	}
}

impl<'a> TemplateContext<'a> {
	pub(crate) fn apply_local_context(&mut self, local_context: &IndexMap<String, Value>) {
		if !local_context.is_empty() {
			let mut new_ctx = self.global_ctx.clone();

			for (key, val) in local_context {
				new_ctx.insert(key, val);
			}

			for (key, val) in self.cli_overrides {
				new_ctx.insert(key, val);
			}

			self.local_ctx = Some(new_ctx);
			self.status = ContextStatus::Dirty;
		} else if self.status.is_dirty() {
			*self = Self::new(self.global_ctx, self.cli_overrides);
		}
	}

	pub(crate) fn new(global_ctx: &'a Context, cli_overrides: &'a IndexMap<String, Value>) -> Self {
		let local_ctx = if !cli_overrides.is_empty() {
			let mut new_ctx = global_ctx.clone();

			for (key, val) in cli_overrides {
				new_ctx.insert(key, val);
			}

			Some(new_ctx)
		} else {
			None
		};

		let status = if local_ctx.is_some() {
			ContextStatus::CliOverridesOnly
		} else {
			ContextStatus::NoCliOverrides
		};

		Self {
			global_ctx,
			cli_overrides,
			status,
			local_ctx,
		}
	}
}

impl RenderCtx<'_> {
	fn render_template(&self, template_name: &str, output_path: &Path) -> Result<(), AppError> {
		create_all_dirs(get_parent_dir(output_path)?)?;

		let mut output_file = open_file_if_overwriting(self.overwrite, output_path)?;

		self.tera
			.render_to(template_name, self.context, &mut output_file)
			.map_err(|e| AppError::TemplateRendering {
				template: template_name.to_string(),
				source: e,
			})
	}

	pub(crate) fn render_remote_preset(
		&mut self,
		remote_preset: &RemotePreset,
	) -> Result<(), AppError> {
		let RemotePreset { repo, exclude } = remote_preset;

		let tmp_dir = env::temp_dir().join("sketch/repo");

		if tmp_dir.exists() {
			remove_dir_all(&tmp_dir).with_context(|| {
				format!("Could not empty the directory `{}`", tmp_dir.display())
			})?;
		}

		let clone_result = Command::new("git")
			.arg("clone")
			.arg("--depth=1")
			.arg(repo)
			.arg(&tmp_dir)
			.output()
			.with_context(|| format!("Could not clone git repo `{repo}`"))?;

		if !clone_result.status.success() {
			let stderr = String::from_utf8_lossy(&clone_result.stderr);
			return Err(anyhow!("Could not clone git repo `{repo}`: {stderr}").into());
		}

		remove_dir_all(tmp_dir.join(".git"))
			.with_context(|| format!("Could not empty the directory `{}`", tmp_dir.display()))?;

		let load_error = || format!("Failed to load the templates from remote template `{repo}`");

		let new_tera =
			Tera::new(&format!("{}/**/*", tmp_dir.display())).with_context(load_error)?;

		self.tera
			.extend(&new_tera)
			.with_context(load_error)?;

		self.render_structured_preset(&tmp_dir, &tmp_dir, exclude)?;

		Ok(())
	}

	pub(crate) fn render_structured_preset(
		&self,
		dir: &Path,
		templates_dir: &Path,
		exclude: &[String],
	) -> Result<(), AppError> {
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

				self.render_template(
					&template_path_from_templates_dir.to_string_lossy(),
					&self.output_root.join(output_path_from_root_dir),
				)?;
			}
		};

		Ok(())
	}

	pub(crate) fn render_single_template(
		&mut self,
		template_data: &TemplateData,
	) -> Result<(), AppError> {
		let TemplateData { template, output } = template_data;

		let template_name = template.name();

		if let TemplateRef::Inline { name, content } = template {
			self.tera
				.add_raw_template(name, content)
				.map_err(|e| AppError::TemplateParsing {
					template: name.clone(),
					source: e,
				})?;
		}

		match output {
			TemplateOutputKind::Stdout => {
				let output = self
					.tera
					.render(template_name, self.context)
					.map_err(|e| AppError::TemplateRendering {
						template: template_name.to_string(),
						source: e,
					})?;

				println!("{output}");
			}
			TemplateOutputKind::Path(path) => {
				self.render_template(template_name, &self.output_root.join(path))?;
			}
		};

		Ok(())
	}
}

impl Config {
	pub fn get_templating_preset(&self, id: &str) -> AppResult<TemplatingPreset> {
		self.templating_presets
			.get(id)
			.ok_or(AppError::PresetNotFound {
				kind: PresetKind::Templates,
				name: id.to_string(),
			})?
			.clone()
			.merge_presets(id, &self.templating_presets)
	}
}
