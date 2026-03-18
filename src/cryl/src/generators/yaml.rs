use std::path::Path;

use crate::common::{deserialize, save_atomic, serialize, CrylResult, Format};

/// Generate a YAML file by converting data from one format to YAML
///
/// # Arguments
/// * `name` - Path to save the YAML file
/// * `in_format` - Input format of the source data ("json", "yaml", "yml", "toml")
/// * `data` - Path to the source data file
/// * `renew` - Overwrite destination if it exists
pub fn generate_yaml(
  name: &Path,
  in_format: &str,
  data: &Path,
  renew: bool,
) -> CrylResult<()> {
  // Parse the input format
  let input_format = Format::parse(in_format)?;

  // Read the source data
  let content = std::fs::read_to_string(data)?;

  // Deserialize from input format using serde_json::Value as intermediate
  let value: serde_json::Value = deserialize(&content, input_format)?;

  // Serialize to YAML
  let yaml_content = serialize(&value, Format::Yaml)?;

  // Save the YAML file (public format)
  save_atomic(name, yaml_content.as_bytes(), renew, true)?;

  // Save the format suffix file to indicate the output format
  let format_suffix_path = name.with_extension("format");
  save_atomic(&format_suffix_path, b"yaml", renew, true)?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;
  use tempfile::TempDir;

  #[test]
  fn test_generate_yaml_from_json() {
    let temp = TempDir::new().unwrap();
    let source_path = temp.path().join("source.json");
    let dest_path = temp.path().join("output.yaml");

    // Create source JSON file
    fs::write(&source_path, r#"{"name": "test", "value": 42}"#).unwrap();

    generate_yaml(&dest_path, "json", &source_path, false).unwrap();

    assert!(dest_path.exists());
    let content = fs::read_to_string(&dest_path).unwrap();
    assert!(content.contains("name:"));
    assert!(content.contains("test"));
    assert!(content.contains("value:"));
  }

  #[test]
  fn test_generate_yaml_from_yaml() {
    let temp = TempDir::new().unwrap();
    let source_path = temp.path().join("source.yaml");
    let dest_path = temp.path().join("output.yaml");

    // Create source YAML file
    fs::write(&source_path, "name: test\nvalue: 42").unwrap();

    generate_yaml(&dest_path, "yaml", &source_path, false).unwrap();

    assert!(dest_path.exists());
    let content = fs::read_to_string(&dest_path).unwrap();
    assert!(content.contains("name:"));
    assert!(content.contains("test"));
    assert!(content.contains("value:"));
  }

  #[test]
  fn test_generate_yaml_from_toml() {
    let temp = TempDir::new().unwrap();
    let source_path = temp.path().join("source.toml");
    let dest_path = temp.path().join("output.yaml");

    // Create source TOML file
    fs::write(&source_path, "name = \"test\"\nvalue = 42").unwrap();

    generate_yaml(&dest_path, "toml", &source_path, false).unwrap();

    assert!(dest_path.exists());
    let content = fs::read_to_string(&dest_path).unwrap();
    assert!(content.contains("name:"));
    assert!(content.contains("test"));
    assert!(content.contains("value:"));
  }

  #[test]
  fn test_generate_yaml_creates_format_file() {
    let temp = TempDir::new().unwrap();
    let source_path = temp.path().join("source.json");
    let dest_path = temp.path().join("output.yaml");
    let format_path = dest_path.with_extension("format");

    fs::write(&source_path, r#"{"test": true}"#).unwrap();

    generate_yaml(&dest_path, "json", &source_path, false).unwrap();

    assert!(format_path.exists());
    let format_content = fs::read_to_string(&format_path).unwrap();
    assert_eq!(format_content, "yaml");
  }

  #[test]
  fn test_generate_yaml_renew_false_no_overwrite() {
    let temp = TempDir::new().unwrap();
    let source_path = temp.path().join("source.json");
    let dest_path = temp.path().join("output.yaml");

    fs::write(&source_path, r#"{"new": true}"#).unwrap();
    fs::write(&dest_path, "original: true").unwrap();

    generate_yaml(&dest_path, "json", &source_path, false).unwrap();

    let content = fs::read_to_string(&dest_path).unwrap();
    assert!(content.contains("original"));
    assert!(!content.contains("new"));
  }

  #[test]
  fn test_generate_yaml_renew_true_overwrites() {
    let temp = TempDir::new().unwrap();
    let source_path = temp.path().join("source.json");
    let dest_path = temp.path().join("output.yaml");

    fs::write(&source_path, r#"{"new": true}"#).unwrap();
    fs::write(&dest_path, "original: true").unwrap();

    generate_yaml(&dest_path, "json", &source_path, true).unwrap();

    let content = fs::read_to_string(&dest_path).unwrap();
    assert!(!content.contains("original"));
    assert!(content.contains("new"));
  }

  #[test]
  fn test_generate_yaml_public_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let temp = TempDir::new().unwrap();
    let source_path = temp.path().join("source.json");
    let dest_path = temp.path().join("output.yaml");

    fs::write(&source_path, r#"{"test": true}"#).unwrap();

    generate_yaml(&dest_path, "json", &source_path, false).unwrap();

    let metadata = fs::metadata(&dest_path).unwrap();
    let perms = metadata.permissions();
    assert_eq!(perms.mode() & 0o777, 0o644);
  }

  #[test]
  fn test_generate_yaml_complex_nested_data() {
    let temp = TempDir::new().unwrap();
    let source_path = temp.path().join("source.json");
    let dest_path = temp.path().join("output.yaml");

    let json_content = r#"{
      "server": {
        "host": "localhost",
        "port": 8080
      },
      "database": {
        "connections": [
          {"name": "primary", "url": "postgres://localhost/db1"},
          {"name": "replica", "url": "postgres://localhost/db2"}
        ]
      },
      "features": ["auth", "logging", "caching"]
    }"#;

    fs::write(&source_path, json_content).unwrap();

    generate_yaml(&dest_path, "json", &source_path, false).unwrap();

    assert!(dest_path.exists());
    let content = fs::read_to_string(&dest_path).unwrap();

    assert!(content.contains("server:"));
    assert!(content.contains("host:"));
    assert!(content.contains("database:"));
    assert!(content.contains("connections:"));
    assert!(content.contains("features:"));
  }

  #[test]
  fn test_generate_yaml_invalid_format() {
    let temp = TempDir::new().unwrap();
    let source_path = temp.path().join("source.json");
    let dest_path = temp.path().join("output.yaml");

    fs::write(&source_path, r#"{"test": true}"#).unwrap();

    let result =
      generate_yaml(&dest_path, "invalid_format", &source_path, false);
    assert!(result.is_err());
  }

  #[test]
  fn test_generate_yaml_source_not_found() {
    let temp = TempDir::new().unwrap();
    let source_path = temp.path().join("nonexistent.json");
    let dest_path = temp.path().join("output.yaml");

    let result = generate_yaml(&dest_path, "json", &source_path, false);
    assert!(result.is_err());
  }
}
