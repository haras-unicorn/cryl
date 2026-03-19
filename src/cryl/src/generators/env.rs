use std::collections::HashMap;
use std::path::Path;

use crate::common::{
  CrylResult, Format, deserialize, read_file_if_exists, save_atomic,
};

/// Generate an environment (.env-style) file from key-value pairs
///
/// # Arguments
/// * `name` - Path to save the environment file
/// * `format` - Input format of variables (json, yaml, toml)
/// * `vars` - Path to file containing key-value pairs
/// * `renew` - Overwrite destination if it exists
///
/// # Description
/// Reads variables from the specified file. For each value, if it exists as a
/// file path, reads the file content; otherwise uses the value directly.
/// Escapes backslashes, newlines, and double quotes in values, then outputs
/// as KEY="value" format.
pub fn generate_env(
  name: &Path,
  format: &str,
  vars: &Path,
  renew: bool,
) -> CrylResult<()> {
  // Read and deserialize the variables file
  let format = Format::parse(format)?;
  let vars_content = std::fs::read_to_string(vars)?;
  let variables: HashMap<String, String> = deserialize(&vars_content, format)?;

  // Process each variable
  let mut lines: Vec<String> = Vec::new();
  for (key, value) in variables {
    // Check if value is a file path and read it if so
    let raw_value = if let Some(content) = read_file_if_exists(&value)? {
      content
    } else {
      value
    };

    // Trim whitespace
    let trimmed = raw_value.trim();

    // Escape special characters: backslash, newline, double quote
    let escaped = trimmed
      .replace('\\', "\\\\")
      .replace('\n', "\\n")
      .replace('\r', "\\r")
      .replace('"', "\\\"");

    lines.push(format!("{}=\"{}\"", key, escaped));
  }

  // Join lines with newlines
  let output = lines.join("\n");

  // Save the file (private permissions since it may contain secrets)
  save_atomic(name, output.as_bytes(), renew, false)?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;
  use std::os::unix::fs::PermissionsExt;
  use tempfile::TempDir;

  #[test]
  fn test_generate_env_basic() {
    let temp = TempDir::new().unwrap();
    let vars_path = temp.path().join("vars.json");
    let env_path = temp.path().join(".env");

    // Create variables file
    fs::write(&vars_path, r#"{"KEY1": "value1", "KEY2": "value2"}"#).unwrap();

    generate_env(&env_path, "json", &vars_path, false).unwrap();

    assert!(env_path.exists());
    let content = fs::read_to_string(&env_path).unwrap();
    assert!(content.contains("KEY1=\"value1\""));
    assert!(content.contains("KEY2=\"value2\""));
  }

  #[test]
  fn test_generate_env_reads_file_values() {
    let temp = TempDir::new().unwrap();
    let vars_path = temp.path().join("vars.json");
    let secret_path = temp.path().join("secret.txt");
    let env_path = temp.path().join(".env");

    // Create a secret file
    fs::write(&secret_path, "my secret password").unwrap();

    // Create variables file referencing the secret file
    fs::write(
      &vars_path,
      format!(r#"{{"PASSWORD": "{}"}}"#, secret_path.display()),
    )
    .unwrap();

    generate_env(&env_path, "json", &vars_path, false).unwrap();

    let content = fs::read_to_string(&env_path).unwrap();
    assert!(content.contains("PASSWORD=\"my secret password\""));
  }

  #[test]
  fn test_generate_env_escapes_special_chars() {
    let temp = TempDir::new().unwrap();
    let vars_path = temp.path().join("vars.json");
    let env_path = temp.path().join(".env");

    // Create variables with special characters (using \\n for newline in JSON)
    fs::write(
      &vars_path,
      r#"{"PATH": "/home/user/docs", "MSG": "Hello\nWorld", "QUOTE": "say \"hi\""}"#,
    )
    .unwrap();

    generate_env(&env_path, "json", &vars_path, false).unwrap();

    let content = fs::read_to_string(&env_path).unwrap();
    assert!(content.contains("PATH=\"/home/user/docs\""));
    assert!(content.contains("MSG=\"Hello\\nWorld\""));
    assert!(content.contains("QUOTE=\"say \\\"hi\\\"\""));
  }

  #[test]
  fn test_generate_env_trims_whitespace() {
    let temp = TempDir::new().unwrap();
    let vars_path = temp.path().join("vars.json");
    let env_path = temp.path().join(".env");

    // Create variables with whitespace
    fs::write(&vars_path, r#"{"KEY": "  value with spaces  "}"#).unwrap();

    generate_env(&env_path, "json", &vars_path, false).unwrap();

    let content = fs::read_to_string(&env_path).unwrap();
    assert!(content.contains("KEY=\"value with spaces\""));
  }

  #[test]
  fn test_generate_env_renew_false_no_overwrite() {
    let temp = TempDir::new().unwrap();
    let vars_path = temp.path().join("vars.json");
    let env_path = temp.path().join(".env");

    fs::write(&vars_path, r#"{"KEY": "new"}"#).unwrap();
    fs::write(&env_path, "KEY=\"old\"").unwrap();

    generate_env(&env_path, "json", &vars_path, false).unwrap();

    let content = fs::read_to_string(&env_path).unwrap();
    assert_eq!(content, "KEY=\"old\"");
  }

  #[test]
  fn test_generate_env_renew_true_overwrites() {
    let temp = TempDir::new().unwrap();
    let vars_path = temp.path().join("vars.json");
    let env_path = temp.path().join(".env");

    fs::write(&vars_path, r#"{"KEY": "new"}"#).unwrap();
    fs::write(&env_path, "KEY=\"old\"").unwrap();

    generate_env(&env_path, "json", &vars_path, true).unwrap();

    let content = fs::read_to_string(&env_path).unwrap();
    assert_eq!(content, "KEY=\"new\"");
  }

  #[test]
  fn test_generate_env_private_permissions() {
    let temp = TempDir::new().unwrap();
    let vars_path = temp.path().join("vars.json");
    let env_path = temp.path().join(".env");

    fs::write(&vars_path, r#"{"KEY": "value"}"#).unwrap();

    generate_env(&env_path, "json", &vars_path, false).unwrap();

    let metadata = fs::metadata(&env_path).unwrap();
    let perms = metadata.permissions();
    assert_eq!(perms.mode() & 0o777, 0o600);
  }

  #[test]
  fn test_generate_env_yaml_format() {
    let temp = TempDir::new().unwrap();
    let vars_path = temp.path().join("vars.yaml");
    let env_path = temp.path().join(".env");

    fs::write(&vars_path, "KEY1: value1\nKEY2: value2").unwrap();

    generate_env(&env_path, "yaml", &vars_path, false).unwrap();

    let content = fs::read_to_string(&env_path).unwrap();
    assert!(content.contains("KEY1=\"value1\""));
    assert!(content.contains("KEY2=\"value2\""));
  }

  #[test]
  fn test_generate_env_toml_format() {
    let temp = TempDir::new().unwrap();
    let vars_path = temp.path().join("vars.toml");
    let env_path = temp.path().join(".env");

    fs::write(&vars_path, "KEY1 = \"value1\"\nKEY2 = \"value2\"").unwrap();

    generate_env(&env_path, "toml", &vars_path, false).unwrap();

    let content = fs::read_to_string(&env_path).unwrap();
    assert!(content.contains("KEY1=\"value1\""));
    assert!(content.contains("KEY2=\"value2\""));
  }

  #[test]
  fn test_generate_env_empty_variables() {
    let temp = TempDir::new().unwrap();
    let vars_path = temp.path().join("vars.json");
    let env_path = temp.path().join(".env");

    fs::write(&vars_path, "{}").unwrap();

    generate_env(&env_path, "json", &vars_path, false).unwrap();

    let content = fs::read_to_string(&env_path).unwrap();
    assert_eq!(content, "");
  }

  #[test]
  fn test_generate_env_handles_carriage_return() {
    let temp = TempDir::new().unwrap();
    let vars_path = temp.path().join("vars.json");
    let env_path = temp.path().join(".env");

    fs::write(&vars_path, r#"{"TEXT": "line1\r\nline2"}"#).unwrap();

    generate_env(&env_path, "json", &vars_path, false).unwrap();

    let content = fs::read_to_string(&env_path).unwrap();
    assert!(content.contains("TEXT=\"line1\\r\\nline2\""));
  }
}
