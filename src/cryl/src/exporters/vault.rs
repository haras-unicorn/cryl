use crate::common::{CrylError, CrylResult};
use std::collections::BTreeMap;

/// Vault exporter - exports all files in current directory to a Vault KV path
pub fn export_vault(path: &str) -> CrylResult<()> {
  // Trim trailing slashes
  let trimmed_path = path.trim_end_matches('/');

  // Collect all files in current directory
  let mut files = BTreeMap::new();
  for entry in std::fs::read_dir(".")? {
    let entry = entry?;
    let path = entry.path();
    if path.is_file() {
      let filename =
        path.file_name().and_then(|s| s.to_str()).ok_or_else(|| {
          CrylError::Export {
            exporter: "vault".to_string(),
            message: format!("Invalid filename: {:?}", path),
          }
        })?;
      let content = std::fs::read_to_string(&path)?;
      files.insert(filename.to_string(), content);
    }
  }

  if files.is_empty() {
    return Ok(());
  }

  // Build YAML structure with current/ key
  let mut yaml_map = serde_yaml::Mapping::new();
  let mut current_map = serde_yaml::Mapping::new();
  for (key, value) in &files {
    current_map.insert(
      serde_yaml::Value::String(key.clone()),
      serde_yaml::Value::String(value.clone()),
    );
  }
  yaml_map.insert(
    serde_yaml::Value::String("current".to_string()),
    serde_yaml::Value::Mapping(current_map),
  );

  let yaml_content =
    serde_yaml::to_string(&yaml_map).map_err(|e| CrylError::Export {
      exporter: "vault".to_string(),
      message: format!("Failed to serialize YAML: {}", e),
    })?;

  // Execute medusa import command
  let mut output = std::process::Command::new("medusa")
    .arg("import")
    .arg(trimmed_path)
    .arg("-")
    .stdin(std::process::Stdio::piped())
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped())
    .spawn()
    .map_err(|e| CrylError::Export {
      exporter: "vault".to_string(),
      message: format!("Failed to spawn medusa import: {}", e),
    })?;

  // Write YAML to medusa stdin
  use std::io::Write;
  {
    let stdin = output.stdin.as_mut().ok_or_else(|| CrylError::Export {
      exporter: "vault".to_string(),
      message: "Failed to open medusa stdin".to_string(),
    })?;
    stdin.write_all(yaml_content.as_bytes()).map_err(|e| {
      CrylError::Export {
        exporter: "vault".to_string(),
        message: format!("Failed to write to medusa stdin: {}", e),
      }
    })?;
  }

  let output = output.wait_with_output().map_err(|e| CrylError::Export {
    exporter: "vault".to_string(),
    message: format!("Failed to wait for medusa import: {}", e),
  })?;

  if !output.status.success() {
    return Err(CrylError::Export {
      exporter: "vault".to_string(),
      message: format!(
        "medusa import failed with status: {}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
      ),
    });
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use crate::common::vault_container;
  use serial_test::serial;
  use std::process::Command;
  use tempfile::TempDir;

  #[tokio::test]
  #[serial]
  async fn test_export_vault_success() -> anyhow::Result<()> {
    let _container = vault_container("vault-export-test").await?;

    let temp_dir = TempDir::new()?;
    std::env::set_current_dir(&temp_dir)?;

    // Create test files
    std::fs::write("secret.txt", "top-secret")?;
    std::fs::write("config.yaml", "port: 8080")?;

    // Export to vault
    super::export_vault("kv/my-app")?;

    // Verify using vault CLI
    let output = Command::new("vault")
      .args(["kv", "get", "-format=json", "kv/my-app/current"])
      .output()?;

    if !output.status.success() {
      anyhow::bail!(
        "vault kv get failed: {}",
        String::from_utf8_lossy(&output.stderr)
      );
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(json["data"]["data"]["secret.txt"], "top-secret");
    assert_eq!(json["data"]["data"]["config.yaml"], "port: 8080");

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_export_vault_empty_directory() -> anyhow::Result<()> {
    let _container = vault_container("vault-export-empty-test").await?;

    let temp_dir = TempDir::new()?;
    std::env::set_current_dir(&temp_dir)?;

    // Export from empty directory should succeed
    super::export_vault("kv/empty-app")?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_export_vault_with_trailing_slash() -> anyhow::Result<()> {
    let _container = vault_container("vault-export-slash-test").await?;

    let temp_dir = TempDir::new()?;
    std::env::set_current_dir(&temp_dir)?;

    std::fs::write("data.txt", "test-data")?;

    // Path with trailing slash should work
    super::export_vault("kv/slash-app/")?;

    // Verify
    let output = Command::new("vault")
      .args(["kv", "get", "-format=json", "kv/slash-app/current"])
      .output()?;

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(json["data"]["data"]["data.txt"], "test-data");

    Ok(())
  }
}
