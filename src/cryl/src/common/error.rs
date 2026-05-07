use std::{env::VarError, fmt::Display, path::PathBuf};

use clap_stdin::StdinError;
use thiserror::Error;

/// Errors that can occur during cryl operations
#[derive(Error, Debug)]
pub enum CrylError {
  #[error("IO error: {0}")]
  Io(#[from] std::io::Error),

  #[error("JSON serialization error: {0}")]
  JsonSerialization(#[from] serde_json::Error),

  #[error("YAML serialization error: {0}")]
  YamlSerialization(#[from] serde_yaml::Error),

  #[error("TOML serialization error: {0}")]
  TomlSerialization(#[from] toml::ser::Error),

  #[error("TOML deserialization error: {0}")]
  TomlDeserialization(#[from] toml::de::Error),

  #[error("Tool execution failed: {tool} exited with {exit_code}\n{stderr}")]
  ToolExecution {
    tool: String,
    exit_code: i32,
    stderr: String,
  },

  #[error("Sandbox error: {0}")]
  Sandbox(String),

  #[error("Import failed: {importer} - {message}")]
  Import { importer: String, message: String },

  #[error("Generation failed: {generator} - {message}")]
  Generation { generator: String, message: String },

  #[error("Export failed: {exporter} - {message}")]
  Export { exporter: String, message: String },

  #[error("Invalid format: {0}")]
  InvalidFormat(String),

  #[error("Validation failed: {0}")]
  Validation(String),

  #[error("Template error: {0}")]
  Template(#[from] mustache::Error),

  #[error("Infallible: {0}")]
  Infallible(#[from] std::convert::Infallible),

  #[error("No working directory")]
  WorkingDirectory,

  #[error("Listing directory failed because file '{0}' not found")]
  DirectoryListing(PathBuf),

  #[error("Error reading file from stdin: {0}")]
  Stdin(#[from] StdinError),

  #[error("Multiple errors occurred:\n{0}")]
  Multiple(MultiCrylError),

  #[error("Failed to get environment variable: {0}")]
  EnvVar(#[from] VarError),

  #[error("In memory filesystem failure: {0}")]
  InMemoryFilesystem(String),
}

impl CrylError {
  pub fn multiple<T: IntoIterator<Item = CrylError>>(iter: T) -> CrylError {
    CrylError::Multiple(MultiCrylError {
      errors: iter.into_iter().collect::<Vec<_>>(),
    })
  }
}

#[derive(Debug)]
pub struct MultiCrylError {
  pub errors: Vec<CrylError>,
}

impl Display for MultiCrylError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    for error in self.errors.iter() {
      error.fmt(f)?;
      f.write_str("\n")?;
    }
    Ok(())
  }
}

/// Result type alias for cryl operations
pub type CrylResult<T> = std::result::Result<T, CrylError>;
