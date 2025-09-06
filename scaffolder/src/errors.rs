use std::{io, path::PathBuf};

use thiserror::Error;

use crate::Preset;

/// The errors that can occur while generating new files.
#[derive(Debug, Error)]
pub enum GenError {
  #[error("Could not create the dir '{path}': {source}")]
  DirCreation { path: PathBuf, source: io::Error },
  #[error("Could not create the file '{path}': {source}")]
  FileCreation { path: PathBuf, source: io::Error },
  #[error("Failed to parse the configuration: {source}")]
  ConfigParsing { source: figment::Error },
  #[error("{kind:?} preset '{name}' not found")]
  PresetNotFound { kind: Preset, name: String },
  #[error("Failed to parse the template '{template}': {source}")]
  TemplateParsing {
    template: String,
    source: ::tera::Error,
  },
  #[error("Failed to read the templates directory: {source}")]
  TemplateDirLoading { source: ::tera::Error },
  #[error("Failed to parse the templating context: {source}")]
  TemplateContextParsing { source: ::tera::Error },
  #[error("Could not create the parent directory for '{path}': {source}")]
  ParentDirCreation { path: PathBuf, source: io::Error },
  #[error("Failed to render the template '{template}': {source}")]
  TemplateRendering {
    template: String,
    source: ::tera::Error,
  },
  #[error("Failed to write to the file '{path}': {source}")]
  WriteError { path: PathBuf, source: io::Error },
  #[error("Person '{name}' not found")]
  PersonNotFound { name: String },
  #[error("Could not read the contents of '{path}': {source}")]
  ReadError { path: PathBuf, source: io::Error },
  #[error("Could not deserialize '{path}': {source}")]
  YamlDeserialization {
    path: PathBuf,
    source: serde_yaml_ng::Error,
  },
  #[error("Could not deserialize '{path}': {source}")]
  JsonDeserialization {
    path: PathBuf,
    source: serde_json::Error,
  },
  #[error("Failed to canonicalize the path '{path}': {source}")]
  PathCanonicalization { path: PathBuf, source: io::Error },
  #[error("Invalid config format for '{file:?}'. Allowed formats are: yaml, toml")]
  InvalidConfigFormat { file: PathBuf },
  #[error("The file '{path}' already exists. Set `overwrite` to true in the config to overwrite existing files.")]
  FileExists { path: PathBuf },
  #[error("{0}")]
  CircularDependency(String),
  #[error("{0}")]
  Custom(String),
}
