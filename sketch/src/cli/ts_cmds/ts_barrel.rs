use super::*;

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

const JS_EXTENSIONS: &[&str] = &["vue", "svelte", "jsx", "tsx", "ts", "js"];

impl TsBarrelArgs {
	pub(crate) fn create_ts_barrel(self, overwrite: bool) -> AppResult {
		let Self {
			dir,
			keep_extensions,
			exclude,
			js_ext,
			output,
		} = self;

		let dir = dir.unwrap_or_else(get_cwd);

		if !dir.is_dir() {
			return Err(anyhow!("`{}` is not a directory", dir.display()).into());
		}

		let mut glob_builder = GlobSetBuilder::new();

		glob_builder.add(Glob::new("index.ts").unwrap());

		if let Some(ref patterns) = exclude {
			for pattern in patterns {
				glob_builder.add(
					Glob::new(pattern)
						.with_context(|| format!("Could not parse glob pattern `{pattern}`"))?,
				);
			}
		}

		let globset = glob_builder
			.build()
			.context("Could not build globset")?;

		let mut paths: BTreeSet<PathBuf> = BTreeSet::new();

		for entry in WalkDir::new(&dir)
			.into_iter()
			.filter_map(|e| e.ok())
			.filter(|e| e.file_type().is_file())
		{
			let mut path = entry
				.path()
				.strip_prefix(&dir)
				.unwrap()
				.to_path_buf();

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

		let template = read_to_string(concat!(
			env!("CARGO_MANIFEST_DIR"),
			"/templates/ts/barrel.ts.j2"
		))
		.context("Failed to read template for barrel file")?;

		let mut context = tera::Context::new();

		context.insert("files", &paths);

		let file_content =
			Tera::one_off(&template, &context, false).context("Failed to create barrel file")?;

		file.write_all(file_content.as_bytes())
			.context("Failed to write barrel file")?;

		Ok(())
	}
}
