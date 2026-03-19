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

  #[error("Invalid specification: {message}")]
  InvalidSpec { message: String },

  #[error("Tool execution failed: {tool} exited with {exit_code}")]
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

  #[error("Tool not found: {0}")]
  ToolNotFound(String),

  #[error("Invalid format: {0}")]
  InvalidFormat(String),

  #[error("Validation failed: {0}")]
  Validation(String),

  #[error("Template error: {0}")]
  Template(#[from] mustache::Error),
}

/// Result type alias for cryl operations
pub type CrylResult<T> = std::result::Result<T, CrylError>;
