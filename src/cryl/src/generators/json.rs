use std::path::Path;

use crate::common::{CrylResult, Format, deserialize, save_atomic, serialize};

/// Generate a JSON file by converting data from one format to JSON
///
/// # Arguments
/// * `name` - Path to save the JSON file
/// * `in_format` - Input format of the source data ("json", "yaml", "yml", "toml")
/// * `data` - Path to the source data file
/// * `renew` - Overwrite destination if it exists
pub fn generate_json(
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

  // Serialize to JSON
  let json_content = serialize(&value, Format::Json)?;

  // Save the JSON file (public format)
  save_atomic(name, json_content.as_bytes(), renew, true)?;

  // Save the format suffix file to indicate the output format
  let format_suffix_path = name.with_extension("format");
  save_atomic(&format_suffix_path, b"json", renew, true)?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;
  use tempfile::TempDir;

  #[test]
  fn test_generate_json_from_json() {
    let temp = TempDir::new().unwrap();
    let source_path = temp.path().join("source.json");
    let dest_path = temp.path().join("output.json");

    // Create source JSON file
    fs::write(&source_path, r#"{"name": "test", "value": 42}"#).unwrap();

    generate_json(&dest_path, "json", &source_path, false).unwrap();

    assert!(dest_path.exists());
    let content = fs::read_to_string(&dest_path).unwrap();
    assert!(content.contains("\"name\""));
    assert!(content.contains("\"test\""));
    assert!(content.contains("42"));
  }

  #[test]
  fn test_generate_json_from_yaml() {
    let temp = TempDir::new().unwrap();
    let source_path = temp.path().join("source.yaml");
    let dest_path = temp.path().join("output.json");

    // Create source YAML file
    fs::write(&source_path, "name: test\nvalue: 42").unwrap();

    generate_json(&dest_path, "yaml", &source_path, false).unwrap();

    assert!(dest_path.exists());
    let content = fs::read_to_string(&dest_path).unwrap();
    assert!(content.contains("\"name\""));
    assert!(content.contains("\"test\""));
    assert!(content.contains("42"));

    // Verify it's valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed["name"], "test");
    assert_eq!(parsed["value"], 42);
  }

  #[test]
  fn test_generate_json_from_toml() {
    let temp = TempDir::new().unwrap();
    let source_path = temp.path().join("source.toml");
    let dest_path = temp.path().join("output.json");

    // Create source TOML file
    fs::write(&source_path, "name = \"test\"\nvalue = 42").unwrap();

    generate_json(&dest_path, "toml", &source_path, false).unwrap();

    assert!(dest_path.exists());
    let content = fs::read_to_string(&dest_path).unwrap();
    assert!(content.contains("\"name\""));
    assert!(content.contains("\"test\""));
    assert!(content.contains("42"));
  }

  #[test]
  fn test_generate_json_creates_format_file() {
    let temp = TempDir::new().unwrap();
    let source_path = temp.path().join("source.json");
    let dest_path = temp.path().join("output.json");
    let format_path = dest_path.with_extension("format");

    fs::write(&source_path, r#"{"test": true}"#).unwrap();

    generate_json(&dest_path, "json", &source_path, false).unwrap();

    assert!(format_path.exists());
    let format_content = fs::read_to_string(&format_path).unwrap();
    assert_eq!(format_content, "json");
  }

  #[test]
  fn test_generate_json_renew_false_no_overwrite() {
    let temp = TempDir::new().unwrap();
    let source_path = temp.path().join("source.json");
    let dest_path = temp.path().join("output.json");

    fs::write(&source_path, r#"{"new": true}"#).unwrap();
    fs::write(&dest_path, r#"{"original": true}"#).unwrap();

    generate_json(&dest_path, "json", &source_path, false).unwrap();

    let content = fs::read_to_string(&dest_path).unwrap();
    assert!(content.contains("original"));
    assert!(!content.contains("new"));
  }

  #[test]
  fn test_generate_json_renew_true_overwrites() {
    let temp = TempDir::new().unwrap();
    let source_path = temp.path().join("source.json");
    let dest_path = temp.path().join("output.json");

    fs::write(&source_path, r#"{"new": true}"#).unwrap();
    fs::write(&dest_path, r#"{"original": true}"#).unwrap();

    generate_json(&dest_path, "json", &source_path, true).unwrap();

    let content = fs::read_to_string(&dest_path).unwrap();
    assert!(!content.contains("original"));
    assert!(content.contains("new"));
  }

  #[test]
  fn test_generate_json_public_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let temp = TempDir::new().unwrap();
    let source_path = temp.path().join("source.json");
    let dest_path = temp.path().join("output.json");

    fs::write(&source_path, r#"{"test": true}"#).unwrap();

    generate_json(&dest_path, "json", &source_path, false).unwrap();

    let metadata = fs::metadata(&dest_path).unwrap();
    let perms = metadata.permissions();
    assert_eq!(perms.mode() & 0o777, 0o644);
  }

  #[test]
  fn test_generate_json_complex_nested_data() {
    let temp = TempDir::new().unwrap();
    let source_path = temp.path().join("source.yaml");
    let dest_path = temp.path().join("output.json");

    let yaml_content = r#"
server:
  host: localhost
  port: 8080
database:
  connections:
    - name: primary
      url: postgres://localhost/db1
    - name: replica
      url: postgres://localhost/db2
features:
  - auth
  - logging
  - caching
"#;

    fs::write(&source_path, yaml_content).unwrap();

    generate_json(&dest_path, "yaml", &source_path, false).unwrap();

    assert!(dest_path.exists());
    let content = fs::read_to_string(&dest_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert_eq!(parsed["server"]["host"], "localhost");
    assert_eq!(parsed["server"]["port"], 8080);
    assert_eq!(
      parsed["database"]["connections"].as_array().unwrap().len(),
      2
    );
    assert_eq!(parsed["features"].as_array().unwrap().len(), 3);
  }

  #[test]
  fn test_generate_json_invalid_format() {
    let temp = TempDir::new().unwrap();
    let source_path = temp.path().join("source.json");
    let dest_path = temp.path().join("output.json");

    fs::write(&source_path, r#"{"test": true}"#).unwrap();

    let result =
      generate_json(&dest_path, "invalid_format", &source_path, false);
    assert!(result.is_err());
  }

  #[test]
  fn test_generate_json_source_not_found() {
    let temp = TempDir::new().unwrap();
    let source_path = temp.path().join("nonexistent.json");
    let dest_path = temp.path().join("output.json");

    let result = generate_json(&dest_path, "json", &source_path, false);
    assert!(result.is_err());
  }
}
