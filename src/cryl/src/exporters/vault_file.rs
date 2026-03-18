use crate::common::{CrylError, CrylResult};
use std::path::Path;

/// Vault file exporter - exports a single file to a Vault KV path
pub fn export_vault_file(path: &str, file: &str) -> CrylResult<()> {
  // Trim trailing slashes
  let trimmed_path = path.trim_end_matches('/');
  let full_path = format!("{}/current", trimmed_path);

  // Check if source file exists
  let file_path = Path::new(file);
  if !file_path.exists() {
    return Err(CrylError::Export {
      exporter: "vault-file".to_string(),
      message: format!("Source file not found: {}", file),
    });
  }

  // Read file content
  let content = std::fs::read_to_string(file_path)?;

  // Execute vault kv put
  let output = std::process::Command::new("vault")
    .arg("kv")
    .arg("put")
    .arg(&full_path)
    .arg(format!("{}={}", file, content))
    .output()
    .map_err(|e| CrylError::Export {
      exporter: "vault-file".to_string(),
      message: format!("Failed to execute vault kv put: {}", e),
    })?;

  if !output.status.success() {
    return Err(CrylError::Export {
      exporter: "vault-file".to_string(),
      message: format!(
        "vault kv put failed with status: {}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
      ),
    });
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::common::vault_container;
  use serial_test::serial;
  use std::process::Command;
  use tempfile::TempDir;

  #[tokio::test]
  #[serial]
  async fn test_export_vault_file_success() -> anyhow::Result<()> {
    let _container = vault_container("vfile-export-test").await?;

    let temp_dir = TempDir::new()?;
    std::env::set_current_dir(&temp_dir)?;

    // Create test file
    std::fs::write("secret.txt", "my-secret-value")?;

    // Export to vault
    export_vault_file("kv/my-app", "secret.txt")?;

    // Verify using vault CLI
    let output = Command::new("vault")
      .args(["kv", "get", "-format=json", "kv/my-app/current"])
      .output()?;

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(json["data"]["data"]["secret.txt"], "my-secret-value");

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_export_vault_file_missing_source() -> anyhow::Result<()> {
    let _container = vault_container("vfile-export-missing-test").await?;

    let temp_dir = TempDir::new()?;
    std::env::set_current_dir(&temp_dir)?;

    // Try to export non-existent file
    let result = export_vault_file("kv/my-app", "nonexistent.txt");
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(matches!(err, CrylError::Export { exporter, message: _ }
      if exporter == "vault-file"));

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_export_vault_file_nested_path() -> anyhow::Result<()> {
    let _container = vault_container("vfile-export-nested-test").await?;

    let temp_dir = TempDir::new()?;
    std::env::set_current_dir(&temp_dir)?;

    std::fs::write("password", "s3cr3t")?;

    // Export to nested path
    export_vault_file("kv/team/project/env", "password")?;

    // Verify
    let output = Command::new("vault")
      .args(["kv", "get", "-format=json", "kv/team/project/env/current"])
      .output()?;

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(json["data"]["data"]["password"], "s3cr3t");

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_export_vault_file_with_trailing_slash() -> anyhow::Result<()> {
    let _container = vault_container("vfile-export-slash-test").await?;

    let temp_dir = TempDir::new()?;
    std::env::set_current_dir(&temp_dir)?;

    std::fs::write("config.yaml", "key: value")?;

    // Path with trailing slash should work
    export_vault_file("kv/my-app/", "config.yaml")?;

    // Verify
    let output = Command::new("vault")
      .args(["kv", "get", "-format=json", "kv/my-app/current"])
      .output()?;

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(json["data"]["data"]["config.yaml"], "key: value");

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_export_vault_file_overwrites_existing() -> anyhow::Result<()> {
    let _container = vault_container("vfile-export-overwrite-test").await?;

    let temp_dir = TempDir::new()?;
    std::env::set_current_dir(&temp_dir)?;

    // First export
    std::fs::write("data.txt", "initial")?;
    export_vault_file("kv/overwrite-app", "data.txt")?;

    // Verify initial value
    let output = Command::new("vault")
      .args(["kv", "get", "-format=json", "kv/overwrite-app/current"])
      .output()?;
    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(json["data"]["data"]["data.txt"], "initial");

    // Update file and re-export
    std::fs::write("data.txt", "updated")?;
    export_vault_file("kv/overwrite-app", "data.txt")?;

    // Verify updated value
    let output = Command::new("vault")
      .args(["kv", "get", "-format=json", "kv/overwrite-app/current"])
      .output()?;
    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(json["data"]["data"]["data.txt"], "updated");

    Ok(())
  }
}
