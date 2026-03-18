//! Format handling for JSON, YAML, and TOML serialization/deserialization

use crate::common::{CrylError, CrylResult};
use std::path::Path;

/// Supported serialization formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
  Json,
  Yaml,
  Toml,
}

impl Format {
  /// Parse format from string
  pub fn parse(s: &str) -> CrylResult<Self> {
    match s.to_lowercase().as_str() {
      "json" => Ok(Self::Json),
      "yaml" | "yml" => Ok(Self::Yaml),
      "toml" => Ok(Self::Toml),
      _ => Err(CrylError::InvalidFormat(format!(
        "Failed parsing format: {s}"
      ))),
    }
  }

  /// Get file extension for format
  pub fn extension(&self) -> &'static str {
    match self {
      Self::Json => "json",
      Self::Yaml => "yaml",
      Self::Toml => "toml",
    }
  }

  /// Detect format from file extension
  pub fn detect_from_path<P: AsRef<Path>>(path: P) -> CrylResult<Self> {
    let path = path.as_ref();
    let ext =
      path
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| {
          CrylError::InvalidFormat(format!(
            "Failed reading extension of {path:?}"
          ))
        })?;
    Self::parse(ext)
  }
}

/// Deserialize content from string based on format
pub fn deserialize<T: serde::de::DeserializeOwned>(
  content: &str,
  format: Format,
) -> CrylResult<T> {
  match format {
    Format::Json => serde_json::from_str(content).map_err(CrylError::from),
    Format::Yaml => {
      serde_yaml::from_str(content).map_err(CrylError::YamlSerialization)
    }
    Format::Toml => {
      toml::from_str(content).map_err(CrylError::TomlDeserialization)
    }
  }
}

/// Serialize content to string based on format
pub fn serialize<T: serde::Serialize>(
  value: &T,
  format: Format,
) -> CrylResult<String> {
  match format {
    Format::Json => {
      serde_json::to_string_pretty(value).map_err(CrylError::from)
    }
    Format::Yaml => {
      serde_yaml::to_string(value).map_err(CrylError::YamlSerialization)
    }
    Format::Toml => {
      toml::to_string_pretty(value).map_err(CrylError::TomlSerialization)
    }
  }
}

/// Deserialize from file path
pub fn deserialize_from_file<T: serde::de::DeserializeOwned, P: AsRef<Path>>(
  path: P,
) -> CrylResult<T> {
  let path = path.as_ref();
  let format = Format::detect_from_path(path)?;

  let content = std::fs::read_to_string(path)?;
  deserialize(&content, format)
}

/// Serialize to file
pub fn serialize_to_file<T: serde::Serialize, P: AsRef<Path>>(
  value: &T,
  path: P,
) -> CrylResult<()> {
  let path = path.as_ref();
  let format = Format::detect_from_path(path)?;

  let content = serialize(value, format)?;
  std::fs::write(path, content)?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde::{Deserialize, Serialize};

  #[derive(Debug, Serialize, Deserialize, PartialEq)]
  struct TestData {
    name: String,
    value: i32,
  }

  #[test]
  fn test_format_parse() {
    assert!(matches!(Format::parse("json"), Ok(Format::Json)));

    assert!(matches!(Format::parse("JSON"), Ok(Format::Json)));
    assert!(matches!(Format::parse("yaml"), Ok(Format::Yaml)));
    assert!(matches!(Format::parse("yml"), Ok(Format::Yaml)));
    assert!(matches!(Format::parse("toml"), Ok(Format::Toml)));
    assert!(matches!(Format::parse("unknown"), Err(_)));
  }

  #[test]
  fn test_format_extension() {
    assert_eq!(Format::Json.extension(), "json");
    assert_eq!(Format::Yaml.extension(), "yaml");
    assert_eq!(Format::Toml.extension(), "toml");
  }

  #[test]
  fn test_serialize_deserialize_json() {
    let data = TestData {
      name: "test".to_string(),
      value: 42,
    };

    let serialized = serialize(&data, Format::Json).unwrap();
    let deserialized: TestData =
      deserialize(&serialized, Format::Json).unwrap();
    assert_eq!(data, deserialized);
  }

  #[test]
  fn test_serialize_deserialize_yaml() {
    let data = TestData {
      name: "test".to_string(),
      value: 42,
    };

    let serialized = serialize(&data, Format::Yaml).unwrap();
    let deserialized: TestData =
      deserialize(&serialized, Format::Yaml).unwrap();
    assert_eq!(data, deserialized);
  }

  #[test]
  fn test_serialize_deserialize_toml() {
    let data = TestData {
      name: "test".to_string(),
      value: 42,
    };

    let serialized = serialize(&data, Format::Toml).unwrap();
    let deserialized: TestData =
      deserialize(&serialized, Format::Toml).unwrap();
    assert_eq!(data, deserialized);
  }
}
