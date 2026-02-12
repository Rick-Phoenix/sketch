use std::{io, path::PathBuf};

use thiserror::Error;

use crate::PresetKind;

/// The kinds of errors that can occur during operations.
#[derive(Debug, Error)]
pub enum GenError {
	// I/O errors
	#[error("Could not create the dir `{path}`: {source}")]
	DirCreation { path: PathBuf, source: io::Error },

	#[error("Failed to create or write to the file `{path}`: {source}")]
	WriteError { path: PathBuf, source: io::Error },

	#[error("Could not read the contents of `{path}`: {source}")]
	ReadError { path: PathBuf, source: io::Error },

	#[error("Failed to canonicalize the path `{path}`: {source}")]
	PathCanonicalization { path: PathBuf, source: io::Error },

	// Invalid values
	#[error("{kind:?} preset `{name}` not found")]
	PresetNotFound { kind: PresetKind, name: String },

	#[error("Failed to parse the template `{template}`: {source}")]
	TemplateParsing {
		template: String,
		source: ::tera::Error,
	},

	#[error("Failed to parse the templating context: {source}")]
	TemplateContextParsing { source: ::tera::Error },

	#[error("Failed to render the template `{template}`: {source}")]
	TemplateRendering {
		template: String,
		source: ::tera::Error,
	},

	#[error("{0}")]
	CircularDependency(String),

	// Serde errors
	#[error("Error while serializing the content for `{file:?}`: {error}")]
	SerializationError { file: PathBuf, error: String },

	#[error("Error while deserializing the contents of `{file:?}`: {error}")]
	DeserializationError { file: PathBuf, error: String },

	#[error(transparent)]
	Other(#[from] anyhow::Error),
}
