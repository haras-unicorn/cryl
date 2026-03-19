use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use crate::common::{
  CrylError, CrylResult, Format, deserialize, read_file_if_exists, save_atomic,
  serialize,
};

/// Generate SOPS-encrypted secrets from key-value inputs
///
/// # Arguments
/// * `age` - Path to file containing Age recipient(s)
/// * `public` - Path to save encrypted YAML (public permissions)
/// * `private` - Path to save plaintext YAML (private permissions)
/// * `format` - Input format for values (json, yaml, toml)
/// * `values` - Path to file containing key-value pairs (values can be strings or file paths)
/// * `renew` - Overwrite destinations if they exist
///
/// # Description
/// Reads key-value pairs from the specified file. For each value, if it exists as a
/// file path, reads the file content; otherwise uses the value directly. Values are
/// trimmed and then output as a YAML file. The YAML is then encrypted using SOPS
/// with the specified Age recipient(s) and saved as the public file.
pub fn generate_sops(
  age: &Path,
  public: &Path,
  private: &Path,
  format: &str,
  values: &Path,
  renew: bool,
) -> CrylResult<()> {
  // If renew is false and both files exist, return early
  if !renew && public.exists() && private.exists() {
    return Ok(());
  }

  // Parse the input format
  let format = Format::parse(format)?;

  // Read and deserialize the values file
  let values_content = std::fs::read_to_string(values)?;
  let values: HashMap<String, String> = deserialize(&values_content, format)?;

  // Process each value: check if it's a file path, trim whitespace
  let mut processed: HashMap<String, String> = HashMap::new();
  for (key, value) in values {
    // Check if value is a file path and read it if so
    let raw_value = if let Some(content) = read_file_if_exists(&value)? {
      content
    } else {
      value
    };

    // Trim whitespace (matching nushell implementation)
    let trimmed = raw_value.trim();
    processed.insert(key, trimmed.to_string());
  }

  // Serialize to YAML for the private file
  let yaml_content = serialize(&processed, Format::Yaml)?;

  // Save plaintext YAML with private permissions (600)
  save_atomic(private, yaml_content.as_bytes(), renew, false)?;

  // Read the age recipient(s) from the age file
  let age_content = std::fs::read_to_string(age)?;
  let age_recipient = age_content.trim();

  // Encrypt the plaintext using sops
  let encrypted_output = Command::new("sops")
    .arg("encrypt")
    .arg(private)
    .arg("--input-type")
    .arg("yaml")
    .arg("--age")
    .arg(age_recipient)
    .arg("--output-type")
    .arg("yaml")
    .output()?;

  if !encrypted_output.status.success() {
    return Err(CrylError::ToolExecution {
      tool: "sops encrypt".to_string(),
      exit_code: encrypted_output.status.code().unwrap_or(-1),
      stderr: String::from_utf8_lossy(&encrypted_output.stderr).to_string(),
    });
  }

  // Save encrypted content with public permissions (644)
  save_atomic(public, &encrypted_output.stdout, renew, true)?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::generators::generate_age_key;
  use std::fs;
  use std::os::unix::fs::PermissionsExt;
  use tempfile::TempDir;

  #[test]
  fn test_generate_sops_basic() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let age_path = temp.path().join("age.key");
    let age_public_path = temp.path().join("age_public.key");
    let values_path = temp.path().join("values.json");
    let public_path = temp.path().join("secrets.enc.yaml");
    let private_path = temp.path().join("secrets.yaml");

    // Generate age key for testing
    generate_age_key(&age_public_path, &age_path, true)?;

    // Create values file
    let values = serde_json::json!({
      "API_KEY": "secret123",
      "DB_PASSWORD": "my password"
    });
    fs::write(&values_path, values.to_string())?;

    generate_sops(
      &age_public_path,
      &public_path,
      &private_path,
      "json",
      &values_path,
      true,
    )?;

    // Check that both files exist
    assert!(public_path.exists());
    assert!(private_path.exists());

    // Check private file contains plaintext YAML
    let private_content = fs::read_to_string(&private_path)?;
    assert!(private_content.contains("API_KEY:"));
    assert!(private_content.contains("secret123"));
    assert!(private_content.contains("DB_PASSWORD:"));
    assert!(private_content.contains("my password"));

    // Check public file contains encrypted content (SOPS metadata)
    let public_content = fs::read_to_string(&public_path)?;
    assert!(public_content.contains("sops:"));
    assert!(public_content.contains("age:"));

    // Check permissions
    let private_metadata = fs::metadata(&private_path)?;
    let private_perms = private_metadata.permissions();
    assert_eq!(private_perms.mode() & 0o777, 0o600);

    let public_metadata = fs::metadata(&public_path)?;
    let public_perms = public_metadata.permissions();
    assert_eq!(public_perms.mode() & 0o777, 0o644);

    Ok(())
  }

  #[test]
  fn test_generate_sops_reads_file_values() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let age_path = temp.path().join("age.key");
    let age_public_path = temp.path().join("age_public.key");
    let secret_path = temp.path().join("secret.txt");
    let values_path = temp.path().join("values.json");
    let public_path = temp.path().join("secrets.enc.yaml");
    let private_path = temp.path().join("secrets.yaml");

    // Generate age key for testing
    generate_age_key(&age_public_path, &age_path, true)?;

    // Create a secret file
    fs::write(&secret_path, "my secret password")?;

    // Create values file referencing the secret file
    let values = serde_json::json!({
      "PASSWORD": secret_path.to_str().unwrap()
    });
    fs::write(&values_path, values.to_string())?;

    generate_sops(
      &age_public_path,
      &public_path,
      &private_path,
      "json",
      &values_path,
      true,
    )?;

    // Check private file contains the content from the referenced file
    let private_content = fs::read_to_string(&private_path)?;
    assert!(private_content.contains("PASSWORD:"));
    assert!(private_content.contains("my secret password"));

    Ok(())
  }

  #[test]
  fn test_generate_sops_trims_whitespace() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let age_path = temp.path().join("age.key");
    let age_public_path = temp.path().join("age_public.key");
    let values_path = temp.path().join("values.json");
    let public_path = temp.path().join("secrets.enc.yaml");
    let private_path = temp.path().join("secrets.yaml");

    // Generate age key for testing
    generate_age_key(&age_public_path, &age_path, true)?;

    // Create values file with whitespace
    let values = serde_json::json!({
      "KEY": "  value with spaces  "
    });
    fs::write(&values_path, values.to_string())?;

    generate_sops(
      &age_public_path,
      &public_path,
      &private_path,
      "json",
      &values_path,
      true,
    )?;

    // Check that whitespace is trimmed
    let private_content = fs::read_to_string(&private_path)?;
    assert!(private_content.contains("KEY:"));
    // The value should be trimmed, so "  value with spaces  " becomes "value with spaces"
    assert!(!private_content.contains("\"  value with spaces  \""));

    Ok(())
  }

  #[test]
  fn test_generate_sops_yaml_format() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let age_path = temp.path().join("age.key");
    let age_public_path = temp.path().join("age_public.key");
    let values_path = temp.path().join("values.yaml");
    let public_path = temp.path().join("secrets.enc.yaml");
    let private_path = temp.path().join("secrets.yaml");

    // Generate age key for testing
    generate_age_key(&age_public_path, &age_path, true)?;

    // Create YAML values file
    fs::write(&values_path, "API_KEY: secret123\nDB_PASSWORD: my password")?;

    generate_sops(
      &age_public_path,
      &public_path,
      &private_path,
      "yaml",
      &values_path,
      true,
    )?;

    // Check private file contains the values
    let private_content = fs::read_to_string(&private_path)?;
    assert!(private_content.contains("API_KEY:"));
    assert!(private_content.contains("secret123"));

    Ok(())
  }

  #[test]
  fn test_generate_sops_toml_format() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let age_path = temp.path().join("age.key");
    let age_public_path = temp.path().join("age_public.key");
    let values_path = temp.path().join("values.toml");
    let public_path = temp.path().join("secrets.enc.yaml");
    let private_path = temp.path().join("secrets.yaml");

    // Generate age key for testing
    generate_age_key(&age_public_path, &age_path, true)?;

    // Create TOML values file
    fs::write(
      &values_path,
      "API_KEY = \"secret123\"\nDB_PASSWORD = \"my password\"",
    )?;

    generate_sops(
      &age_public_path,
      &public_path,
      &private_path,
      "toml",
      &values_path,
      true,
    )?;

    // Check private file contains the values
    let private_content = fs::read_to_string(&private_path)?;
    assert!(private_content.contains("API_KEY:"));
    assert!(private_content.contains("secret123"));

    Ok(())
  }

  #[test]
  fn test_generate_sops_empty_values() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let age_path = temp.path().join("age.key");
    let age_public_path = temp.path().join("age_public.key");
    let values_path = temp.path().join("values.json");
    let public_path = temp.path().join("secrets.enc.yaml");
    let private_path = temp.path().join("secrets.yaml");

    // Generate age key for testing
    generate_age_key(&age_public_path, &age_path, true)?;

    // Create empty values file
    fs::write(&values_path, "{}")?;

    generate_sops(
      &age_public_path,
      &public_path,
      &private_path,
      "json",
      &values_path,
      true,
    )?;

    // Check both files exist even with empty values
    assert!(public_path.exists());
    assert!(private_path.exists());

    // Private file should contain empty YAML
    let private_content = fs::read_to_string(&private_path)?;
    assert!(private_content.contains("{}"));

    Ok(())
  }

  #[test]
  fn test_generate_sops_renew_false_no_overwrite() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let age_path = temp.path().join("age.key");
    let age_public_path = temp.path().join("age_public.key");
    let values_path = temp.path().join("values.json");
    let public_path = temp.path().join("secrets.enc.yaml");
    let private_path = temp.path().join("secrets.yaml");

    // Generate age key for testing
    generate_age_key(&age_public_path, &age_path, true)?;

    // Pre-create files
    fs::write(&public_path, "existing_public")?;
    fs::write(&private_path, "existing_private")?;

    // Create values file
    let values = serde_json::json!({"KEY": "new"});
    fs::write(&values_path, values.to_string())?;

    // Generate with renew=false should not overwrite
    generate_sops(
      &age_public_path,
      &public_path,
      &private_path,
      "json",
      &values_path,
      false,
    )?;

    let public_content = fs::read_to_string(&public_path)?;
    let private_content = fs::read_to_string(&private_path)?;

    assert_eq!(public_content, "existing_public");
    assert_eq!(private_content, "existing_private");

    Ok(())
  }

  #[test]
  fn test_generate_sops_renew_true_overwrites() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let age_path = temp.path().join("age.key");
    let age_public_path = temp.path().join("age_public.key");
    let values_path = temp.path().join("values.json");
    let public_path = temp.path().join("secrets.enc.yaml");
    let private_path = temp.path().join("secrets.yaml");

    // Generate age key for testing
    generate_age_key(&age_public_path, &age_path, true)?;

    // Pre-create files
    fs::write(&public_path, "existing_public")?;
    fs::write(&private_path, "existing_private")?;

    // Create values file
    let values = serde_json::json!({"KEY": "new"});
    fs::write(&values_path, values.to_string())?;

    // Generate with renew=true should overwrite
    generate_sops(
      &age_public_path,
      &public_path,
      &private_path,
      "json",
      &values_path,
      true,
    )?;

    let private_content = fs::read_to_string(&private_path)?;
    assert!(private_content.contains("KEY:"));
    assert!(private_content.contains("new"));

    let public_content = fs::read_to_string(&public_path)?;
    assert!(public_content.contains("sops:"));

    Ok(())
  }

  #[test]
  fn test_generate_sops_multiline_value() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let age_path = temp.path().join("age.key");
    let age_public_path = temp.path().join("age_public.key");
    let values_path = temp.path().join("values.json");
    let public_path = temp.path().join("secrets.enc.yaml");
    let private_path = temp.path().join("secrets.yaml");

    // Generate age key for testing
    generate_age_key(&age_public_path, &age_path, true)?;

    // Create values file with multiline value
    let values = serde_json::json!({
      "CERT": "line1\nline2\nline3"
    });
    fs::write(&values_path, values.to_string())?;

    generate_sops(
      &age_public_path,
      &public_path,
      &private_path,
      "json",
      &values_path,
      true,
    )?;

    // Check private file contains the multiline value
    let private_content = fs::read_to_string(&private_path)?;
    assert!(private_content.contains("CERT:"));
    // In YAML, newlines in values are preserved
    assert!(private_content.contains("line1"));

    Ok(())
  }
}
