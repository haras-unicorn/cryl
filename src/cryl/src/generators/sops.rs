use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use crate::common::{
  CrylError, CrylResult, DirectoryListing, Format, deserialize_from_file,
  list_directory, save_atomic, serialize,
};

/// Generate SOPS-encrypted secrets from key-value inputs
///
/// # Arguments
/// * `age` - Path to file containing Age recipient(s)
/// * `public` - Path to save encrypted YAML (public permissions)
/// * `private` - Path to save plaintext YAML (private permissions)
/// * `format` - Input format for listing (json, yaml, toml)
/// * `listing` - Path to file containing directory listing
/// * `renew` - Overwrite destinations if they exist
///
/// # Description
/// Reads key-value pairs from the specified file. For each value, if it exists as a
/// file path, reads the file content; otherwise uses the value directly. Values are then
/// output as a YAML file. The YAML is then encrypted using SOPS
/// with the specified Age recipient(s) and saved as the public file.
pub fn generate_sops(
  age: &Path,
  public: &Path,
  private: &Path,
  format: Format,
  listing: &Path,
  renew: bool,
) -> CrylResult<()> {
  // If renew is false and both files exist, return early
  if !renew && public.exists() && private.exists() {
    return Ok(());
  }

  // Read and deserialize the listing file
  let listing: DirectoryListing = deserialize_from_file(listing, Some(format))?;

  // list directory
  let listed = list_directory(std::env::current_dir()?, &listing, false, "-")?;

  // Serialize to YAML for the private file
  let yaml_content = serialize(
    &listed
      .into_iter()
      .map(|(key, value)| {
        (key, String::from_utf8_lossy(value.as_slice()).to_string())
      })
      .collect::<HashMap<_, _>>(),
    Format::Yaml,
  )?;

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
  use serial_test::serial;

  use super::*;
  use crate::common::TempCurrentDir;
  use crate::generators::generate_age_key;
  use std::fs;
  use std::os::unix::fs::PermissionsExt;
  use std::path::PathBuf;
  use std::str::FromStr;

  #[test]
  #[serial(working_directory)]
  fn test_generate_sops_basic() -> anyhow::Result<()> {
    let _temp = TempCurrentDir::new()?;
    let age_path = PathBuf::from_str("age.key")?;
    let age_public_path = PathBuf::from_str("age_public.key")?;
    let values_path = PathBuf::from_str("values.json")?;
    let public_path = PathBuf::from_str("secrets.enc.yaml")?;
    let private_path = PathBuf::from_str("secrets.yaml")?;

    // Generate age key for testing
    generate_age_key(&age_public_path, &age_path, true)?;

    // Create listing file
    std::fs::write("api_key", "secret123")?;
    std::fs::write("db_password", "my password")?;
    let listing = serde_json::json!({
      "type": "map",
      "value": {
        "API_KEY": "api_key",
        "DB_PASSWORD": "db_password"
      }
    });
    fs::write(&values_path, listing.to_string())?;

    generate_sops(
      &age_public_path,
      &public_path,
      &private_path,
      Format::Json,
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
  #[serial(working_directory)]
  fn test_generate_sops_reads_file_values() -> anyhow::Result<()> {
    let _temp = TempCurrentDir::new()?;
    let age_path = PathBuf::from_str("age.key")?;
    let age_public_path = PathBuf::from_str("age_public.key")?;
    let secret_path = PathBuf::from_str("secret.txt")?;
    let values_path = PathBuf::from_str("values.json")?;
    let public_path = PathBuf::from_str("secrets.enc.yaml")?;
    let private_path = PathBuf::from_str("secrets.yaml")?;

    // Generate age key for testing
    generate_age_key(&age_public_path, &age_path, true)?;

    // Create a secret file
    fs::write(&secret_path, "my secret password")?;

    // Create values file referencing the secret file
    let values = serde_json::json!({
      "type": "map",
      "value": {
        "PASSWORD": secret_path.to_str().unwrap()
      }
    });
    fs::write(&values_path, values.to_string())?;

    generate_sops(
      &age_public_path,
      &public_path,
      &private_path,
      Format::Json,
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
  #[serial(working_directory)]
  fn test_generate_sops_does_not_trim_whitespace() -> anyhow::Result<()> {
    let _temp = TempCurrentDir::new()?;
    let age_path = PathBuf::from_str("age.key")?;
    let age_public_path = PathBuf::from_str("age_public.key")?;
    let values_path = PathBuf::from_str("values.json")?;
    let public_path = PathBuf::from_str("secrets.enc.yaml")?;
    let private_path = PathBuf::from_str("secrets.yaml")?;

    // Generate age key for testing
    generate_age_key(&age_public_path, &age_path, true)?;

    // Create values file with whitespace
    fs::write("key", "  value with spaces  ")?;
    let values = serde_json::json!({
      "type": "map",
      "value": {
        "KEY": "./key"
      }
    });
    fs::write(&values_path, values.to_string())?;

    generate_sops(
      &age_public_path,
      &public_path,
      &private_path,
      Format::Json,
      &values_path,
      true,
    )?;

    // Check that whitespace is trimmed
    let private_content = fs::read_to_string(&private_path)?;
    assert!(private_content.contains("KEY:"));
    // The value should not be trimmed
    assert!(private_content.contains("'  value with spaces  '"));

    Ok(())
  }

  #[test]
  #[serial(working_directory)]
  fn test_generate_sops_yaml_format() -> anyhow::Result<()> {
    let _temp = TempCurrentDir::new()?;
    let age_path = PathBuf::from_str("age.key")?;
    let age_public_path = PathBuf::from_str("age_public.key")?;
    let values_path = PathBuf::from_str("values.yaml")?;
    let public_path = PathBuf::from_str("secrets.enc.yaml")?;
    let private_path = PathBuf::from_str("secrets.yaml")?;

    // Generate age key for testing
    generate_age_key(&age_public_path, &age_path, true)?;

    // Create YAML values file
    fs::write("api_key", "secret123")?;
    fs::write("db_password", "my password")?;
    fs::write(
      &values_path,
      r#"
      type: map
      value:
        API_KEY: api_key
        DB_PASSWORD: ./db_password
    "#,
    )?;

    generate_sops(
      &age_public_path,
      &public_path,
      &private_path,
      Format::Yaml,
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
  #[serial(working_directory)]
  fn test_generate_sops_toml_format() -> anyhow::Result<()> {
    let _temp = TempCurrentDir::new()?;
    let age_path = PathBuf::from_str("age.key")?;
    let age_public_path = PathBuf::from_str("age_public.key")?;
    let values_path = PathBuf::from_str("values.toml")?;
    let public_path = PathBuf::from_str("secrets.enc.yaml")?;
    let private_path = PathBuf::from_str("secrets.yaml")?;

    // Generate age key for testing
    generate_age_key(&age_public_path, &age_path, true)?;

    // Create TOML values file
    fs::write("api_key", "secret123")?;
    fs::write("db_password", "my password")?;
    fs::write(
      &values_path,
      r#"
        type = "map"

        [value]
        API_KEY = "api_key"
        DB_PASSWORD = "db_password"
      "#,
    )?;

    generate_sops(
      &age_public_path,
      &public_path,
      &private_path,
      Format::Toml,
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
  #[serial(working_directory)]
  fn test_generate_sops_empty_values() -> anyhow::Result<()> {
    let _temp = TempCurrentDir::new()?;
    let age_path = PathBuf::from_str("age.key")?;
    let age_public_path = PathBuf::from_str("age_public.key")?;
    let values_path = PathBuf::from_str("values.json")?;
    let public_path = PathBuf::from_str("secrets.enc.yaml")?;
    let private_path = PathBuf::from_str("secrets.yaml")?;

    // Generate age key for testing
    generate_age_key(&age_public_path, &age_path, true)?;

    // Create empty values file
    fs::write(
      &values_path,
      r#"{
        "type": "map",
        "value": { }
      }"#,
    )?;

    generate_sops(
      &age_public_path,
      &public_path,
      &private_path,
      Format::Json,
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
  #[serial(working_directory)]
  fn test_generate_sops_renew_false_no_overwrite() -> anyhow::Result<()> {
    let _temp = TempCurrentDir::new()?;
    let age_path = PathBuf::from_str("age.key")?;
    let age_public_path = PathBuf::from_str("age_public.key")?;
    let values_path = PathBuf::from_str("values.json")?;
    let public_path = PathBuf::from_str("secrets.enc.yaml")?;
    let private_path = PathBuf::from_str("secrets.yaml")?;

    // Generate age key for testing
    generate_age_key(&age_public_path, &age_path, true)?;

    // Pre-create files
    fs::write(&public_path, "existing_public")?;
    fs::write(&private_path, "existing_private")?;

    // Create values file
    fs::write("new", "new")?;
    let values = serde_json::json!({
      "type": "map",
      "value": {
         "KEY": "new"
      }
    });
    fs::write(&values_path, values.to_string())?;

    // Generate with renew=false should not overwrite
    generate_sops(
      &age_public_path,
      &public_path,
      &private_path,
      Format::Json,
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
  #[serial(working_directory)]
  fn test_generate_sops_renew_true_overwrites() -> anyhow::Result<()> {
    let _temp = TempCurrentDir::new()?;
    let age_path = PathBuf::from_str("age.key")?;
    let age_public_path = PathBuf::from_str("age_public.key")?;
    let values_path = PathBuf::from_str("values.json")?;
    let public_path = PathBuf::from_str("secrets.enc.yaml")?;
    let private_path = PathBuf::from_str("secrets.yaml")?;

    // Generate age key for testing
    generate_age_key(&age_public_path, &age_path, true)?;

    // Pre-create files
    fs::write(&public_path, "existing_public")?;
    fs::write(&private_path, "existing_private")?;

    // Create values file
    fs::write("new", "new")?;
    let values = serde_json::json!({
      "type": "map",
      "value": {
         "KEY": "new"
      }
    });
    fs::write(&values_path, values.to_string())?;

    // Generate with renew=true should overwrite
    generate_sops(
      &age_public_path,
      &public_path,
      &private_path,
      Format::Json,
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
  #[serial(working_directory)]
  fn test_generate_sops_multiline_value() -> anyhow::Result<()> {
    let _temp = TempCurrentDir::new()?;
    let age_path = PathBuf::from_str("age.key")?;
    let age_public_path = PathBuf::from_str("age_public.key")?;
    let values_path = PathBuf::from_str("values.json")?;
    let public_path = PathBuf::from_str("secrets.enc.yaml")?;
    let private_path = PathBuf::from_str("secrets.yaml")?;

    // Generate age key for testing
    generate_age_key(&age_public_path, &age_path, true)?;

    // Create values file with multiline value
    fs::write("cert", "line1\nline2\nline3")?;
    let values = serde_json::json!({
      "type": "map",
      "value": {
        "CERT": "./cert"
      }
    });
    fs::write(&values_path, values.to_string())?;

    generate_sops(
      &age_public_path,
      &public_path,
      &private_path,
      Format::Json,
      &values_path,
      true,
    )?;

    // Check private file contains the multiline value
    let private_content = fs::read_to_string(&private_path)?;
    assert!(private_content.contains("CERT:"));
    // In YAML, newlines in values are preserved
    assert!(private_content.contains("line1\n"));
    assert!(private_content.contains("line2\n"));
    assert!(private_content.contains("line3\n"));

    Ok(())
  }

  #[test]
  #[serial(working_directory)]
  fn test_generate_sops_subdirs() -> anyhow::Result<()> {
    let _temp = TempCurrentDir::new()?;
    let age_path = PathBuf::from_str("subdir1")?.join("age.key");
    let age_public_path = PathBuf::from_str("subdir2")?.join("age_public.key");
    let values_path = PathBuf::from_str("subdir3")?.join("values.json");
    let public_path = PathBuf::from_str("subdir4")?.join("secrets.enc.yaml");
    let private_path = PathBuf::from_str("subdir5")?.join("secrets.yaml");

    // Generate age key for testing
    generate_age_key(&age_public_path, &age_path, true)?;

    // Create values file
    std::fs::create_dir("subdir6")?;
    std::fs::write("subdir6/api_key", "secret123")?;
    std::fs::write("subdir6/db_password", "my password")?;
    let values = serde_json::json!({
      "type": "map",
      "value": {
        "API_KEY": "subdir6/api_key",
        "DB_PASSWORD": "subdir6/db_password"
      }
    });
    fs::create_dir_all(values_path.parent().unwrap()).unwrap();
    fs::write(&values_path, values.to_string())?;

    generate_sops(
      &age_public_path,
      &public_path,
      &private_path,
      Format::Json,
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
  #[serial(working_directory)]
  fn test_generate_sops_flat_variant() -> anyhow::Result<()> {
    let _temp = TempCurrentDir::new()?;
    let age_path = PathBuf::from_str("age.key")?;
    let age_public_path = PathBuf::from_str("age_public.key")?;
    let values_path = PathBuf::from_str("values.json")?;
    let public_path = PathBuf::from_str("secrets.enc.yaml")?;
    let private_path = PathBuf::from_str("secrets.yaml")?;

    generate_age_key(&age_public_path, &age_path, true)?;

    // Create test files in current directory
    fs::write("api_key", "secret123")?;
    fs::write("config", "value456")?;
    // Create subdir file - should NOT be included in Flat
    fs::create_dir("subdir")?;
    fs::write("subdir/hidden", "not-included")?;

    // Flat variant listing
    let values = serde_json::json!({
        "type": "flat"
    });
    fs::write(&values_path, values.to_string())?;

    generate_sops(
      &age_public_path,
      &public_path,
      &private_path,
      Format::Json,
      &values_path,
      true,
    )?;

    let private_content = fs::read_to_string(&private_path)?;
    println!("{private_content}");
    assert!(private_content.contains("api_key:"));
    assert!(private_content.contains("secret123"));
    assert!(private_content.contains("config:"));
    assert!(private_content.contains("value456"));
    // Flat should NOT include subdirectory files
    assert!(!private_content.contains("hidden"));
    assert!(!private_content.contains("not-included"));

    Ok(())
  }

  #[test]
  #[serial(working_directory)]
  fn test_generate_sops_deep_variant() -> anyhow::Result<()> {
    let _temp = TempCurrentDir::new()?;
    let age_path = PathBuf::from_str("age.key")?;
    let age_public_path = PathBuf::from_str("age_public.key")?;
    let values_path = PathBuf::from_str("values.json")?;
    let public_path = PathBuf::from_str("secrets.enc.yaml")?;
    let private_path = PathBuf::from_str("secrets.yaml")?;

    generate_age_key(&age_public_path, &age_path, true)?;

    // Create nested directory structure
    fs::create_dir_all("nested/deep/very")?;
    fs::write("top.txt", "top_value")?;
    fs::write("nested/mid.txt", "mid_value")?;
    fs::write("nested/deep/bottom.txt", "bottom_value")?;
    fs::write("nested/deep/very/final.txt", "final_value")?;

    // Deep variant listing
    let values = serde_json::json!({
        "type": "deep"
    });
    fs::write(&values_path, values.to_string())?;

    generate_sops(
      &age_public_path,
      &public_path,
      &private_path,
      Format::Json,
      &values_path,
      true,
    )?;

    let private_content = fs::read_to_string(&private_path)?;
    // Deep should find all files recursively
    assert!(private_content.contains("top.txt:"));
    assert!(private_content.contains("top_value"));
    // Check kebab-case conversion for paths
    assert!(private_content.contains("nested-mid.txt:"));
    assert!(private_content.contains("mid_value"));
    assert!(private_content.contains("nested-deep-bottom.txt:"));
    assert!(private_content.contains("bottom_value"));
    assert!(private_content.contains("nested-deep-very-final.txt:"));
    assert!(private_content.contains("final_value"));

    Ok(())
  }

  #[test]
  #[serial(working_directory)]
  fn test_generate_sops_list_variant() -> anyhow::Result<()> {
    let _temp = TempCurrentDir::new()?;
    let age_path = PathBuf::from_str("age.key")?;
    let age_public_path = PathBuf::from_str("age_public.key")?;
    let values_path = PathBuf::from_str("values.json")?;
    let public_path = PathBuf::from_str("secrets.enc.yaml")?;
    let private_path = PathBuf::from_str("secrets.yaml")?;

    generate_age_key(&age_public_path, &age_path, true)?;

    // Create files for list
    fs::create_dir("dir_a")?;
    fs::create_dir("dir_b")?;
    fs::write("root_file", "root_content")?;
    fs::write("dir_a/file_a", "content_a")?;
    fs::write("dir_b/file_b", "content_b")?;
    // This file should NOT be included (not in list)
    fs::write("excluded.txt", "should_not_appear")?;

    // List variant with specific paths
    let values = serde_json::json!({
        "type": "list",
        "value": [
            "root_file",
            "./dir_a/file_a",
            "dir_b/file_b"
        ]
    });
    fs::write(&values_path, values.to_string())?;

    generate_sops(
      &age_public_path,
      &public_path,
      &private_path,
      Format::Json,
      &values_path,
      true,
    )?;

    let private_content = fs::read_to_string(&private_path)?;
    // Only listed files should appear
    assert!(private_content.contains("root_file:"));
    assert!(private_content.contains("root_content"));
    assert!(private_content.contains("dir_a-file_a:"));
    assert!(private_content.contains("content_a"));
    assert!(private_content.contains("dir_b-file_b:"));
    assert!(private_content.contains("content_b"));
    // Excluded file should NOT appear
    assert!(!private_content.contains("excluded.txt"));
    assert!(!private_content.contains("should_not_appear"));

    Ok(())
  }
}
