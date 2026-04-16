use std::path::Path;

use crate::common::{CrylError, CrylResult, save_atomic};

/// Vault file importer - imports a single file from a Vault KV path
pub fn import_vault_file(
  path: &str,
  file: &Path,
  allow_fail: bool,
) -> CrylResult<()> {
  // Trim trailing slashes
  let trimmed_path = path.trim_end_matches('/');
  let full_path = format!("{}/current", trimmed_path);

  // Extract last component now to exit early
  // and because the file might be under a subdirectory
  let Some(last_component) = file
    .components()
    .next_back()
    .and_then(|component| component.as_os_str().to_str())
    .to_owned()
  else {
    return Err(CrylError::Import {
      importer: "vault".to_string(),
      message: "Path is empty".to_string(),
    });
  };

  // Execute vault kv get
  let output = match std::process::Command::new("vault")
    .arg("kv")
    .arg("get")
    .arg("-format=json")
    .arg(&full_path)
    .output()
  {
    Ok(output) => output,
    Err(_) if allow_fail => return Ok(()),
    Err(e) => {
      return Err(CrylError::Import {
        importer: "vault-file".to_string(),
        message: format!("Failed to execute vault kv get: {}", e),
      });
    }
  };

  if !output.status.success() {
    if allow_fail {
      return Ok(());
    }
    return Err(CrylError::Import {
      importer: "vault-file".to_string(),
      message: format!(
        "vault kv get failed with status: {}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
      ),
    });
  }

  // Parse JSON output
  let json_content = match String::from_utf8(output.stdout) {
    Ok(content) => content,
    Err(e) => {
      return Err(CrylError::Import {
        importer: "vault-file".to_string(),
        message: format!("Invalid UTF-8 from vault kv get: {}", e),
      });
    }
  };

  // Extract the specific file
  let parsed: serde_json::Value = match serde_json::from_str(&json_content) {
    Ok(parsed) => parsed,
    Err(e) => {
      return Err(CrylError::Import {
        importer: "vault-file".to_string(),
        message: format!("Failed to parse vault JSON output: {}", e),
      });
    }
  };

  let file_content = match parsed
    .get("data")
    .and_then(|data| data.get("data"))
    .and_then(|inner| inner.get(last_component))
    .and_then(|value| value.as_str())
  {
    Some(content) => content.to_string(),
    None => {
      if allow_fail {
        return Ok(());
      }
      return Err(CrylError::Import {
        importer: "vault-file".to_string(),
        message: format!(
          "File '{}' not found in Vault path: {}",
          last_component, path
        ),
      });
    }
  };

  save_atomic(file, file_content.as_bytes(), true, false)?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::common::vault_container;
  use base64::Engine;
  use serial_test::serial;
  use std::{os::unix::fs::PermissionsExt, process::Command};
  use tempfile::TempDir;

  #[tokio::test]
  #[serial(environment)]
  async fn test_import_vault_file_success() -> anyhow::Result<()> {
    let _container = vault_container("vfile-success-test").await?;

    // Write test secret
    Command::new("vault")
      .args([
        "kv",
        "put",
        "kv/test-app/current",
        "secret.txt=top-secret-value",
        "config.yaml=port: 8080",
      ])
      .output()?;

    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("secret.txt");

    // Test import
    import_vault_file("kv/test-app", &file_path, false)?;

    // Verify file was created
    assert!(file_path.exists());
    let content = std::fs::read_to_string(&file_path)?;
    assert_eq!(content, "top-secret-value");

    // Check permissions are 600
    let metadata = std::fs::metadata(&file_path)?;
    #[cfg(unix)]
    assert_eq!(metadata.permissions().mode() & 0o777, 0o600);

    Ok(())
  }

  #[tokio::test]
  #[serial(environment)]
  async fn test_import_vault_file_missing_key_allow_fail() -> anyhow::Result<()>
  {
    let _container = vault_container("vfile-missing-key-test").await?;

    // Create secret with different key
    Command::new("vault")
      .args(["kv", "put", "kv/test-app/current", "other.txt=value"])
      .output()?;

    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("secret.txt");

    // Should not error with allow_fail=true
    import_vault_file("kv/test-app", &file_path, true)?;

    // File should not exist
    assert!(!file_path.exists());

    Ok(())
  }

  #[tokio::test]
  #[serial(environment)]
  async fn test_import_vault_file_missing_key_no_allow_fail()
  -> anyhow::Result<()> {
    let _container = vault_container("vfile-missing-key-err-test").await?;

    Command::new("vault")
      .args(["kv", "put", "kv/test-app/current", "other.txt=value"])
      .output()?;

    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("secret.txt");

    // Should error with allow_fail=false
    let result = import_vault_file("kv/test-app", &file_path, false);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(matches!(err, CrylError::Import { importer, message: _ }
      if importer == "vault-file"));

    Ok(())
  }

  #[tokio::test]
  #[serial(environment)]
  async fn test_import_vault_file_missing_path_allow_fail() -> anyhow::Result<()>
  {
    let _container = vault_container("vfile-missing-path-test").await?;

    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("secret.txt");

    // Non-existent path with allow_fail=true should succeed
    import_vault_file("kv/nonexistent", &file_path, true)?;
    assert!(!file_path.exists());

    Ok(())
  }

  #[tokio::test]
  #[serial(environment)]
  async fn test_import_vault_file_vault_not_running_allow_fail()
  -> anyhow::Result<()> {
    // Test without starting container
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("secret.txt");

    // Should succeed with allow_fail=true when vault command fails
    import_vault_file("kv/test", &file_path, true)?;
    assert!(!file_path.exists());

    Ok(())
  }

  #[tokio::test]
  #[serial(environment)]
  async fn test_import_vault_file_vault_not_running_no_allow_fail()
  -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("secret.txt");

    let result = import_vault_file("kv/test", &file_path, false);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(matches!(err, CrylError::Import { importer, message: _ }
      if importer == "vault-file"));

    Ok(())
  }

  #[tokio::test]
  #[serial(environment)]
  async fn test_import_vault_file_nested_path() -> anyhow::Result<()> {
    let _container = vault_container("vfile-nested-test").await?;

    let temp_dir = TempDir::new()?;
    let file_name = "secret.txt";
    let file_path = temp_dir.path().join(file_name);
    let file_content = "s3cr3t";

    // Test with nested path
    Command::new("vault")
      .args([
        "kv",
        "put",
        "kv/team/project/env/current",
        &format!("{}={}", file_name, file_content),
      ])
      .output()?;

    // Import from nested path
    import_vault_file("kv/team/project/env", &file_path, false)?;

    assert!(file_path.exists());
    let content = std::fs::read_to_string(&file_path)?;
    assert_eq!(content, file_content);

    Ok(())
  }

  #[tokio::test]
  #[serial(environment)]
  async fn test_import_vault_file_with_trailing_slash() -> anyhow::Result<()> {
    let _container = vault_container("vfile-trailing-test").await?;

    let temp_dir = TempDir::new()?;
    let file_name = "secret.txt";
    let file_path = temp_dir.path().join(file_name);

    Command::new("vault")
      .args([
        "kv",
        "put",
        "kv/my-app/current",
        &format!("{}=key: value", file_name),
      ])
      .output()?;

    // Path with trailing slash should still work
    import_vault_file("kv/my-app/", &file_path, false)?;

    assert!(file_path.exists());
    let content = std::fs::read_to_string(&file_path)?;
    assert_eq!(content, "key: value");

    Ok(())
  }

  #[tokio::test]
  #[serial(environment)]
  async fn test_import_vault_file_binary_data() -> anyhow::Result<()> {
    let _container = vault_container("vfile-binary-test").await?;

    // Store binary data (Vault encodes as base64 in JSON)
    let binary_data = vec![0x00, 0x01, 0x02, 0xFF];
    let encoded =
      base64::engine::general_purpose::STANDARD.encode(&binary_data);

    let temp_dir = TempDir::new()?;
    let file_name = "secret.txt";
    let file_path = temp_dir.path().join(file_name);

    Command::new("vault")
      .args([
        "kv",
        "put",
        "kv/binary/current",
        &format!("{}={}", file_name, encoded),
      ])
      .output()?;

    import_vault_file("kv/binary", &file_path, false)?;

    // Vault returns base64-encoded strings in JSON, which gets decoded by serde_json
    // The function saves the string value as-is
    let content = std::fs::read(&file_path)?;
    assert_eq!(content, encoded.as_bytes());

    Ok(())
  }

  #[tokio::test]
  #[serial(environment)]
  async fn test_import_vault_file_permissions() -> anyhow::Result<()> {
    let _container = vault_container("vfile-permissions-test").await?;

    let temp_dir = TempDir::new()?;
    let file_name = "secret.txt";
    let file_path = temp_dir.path().join(file_name);

    Command::new("vault")
      .args([
        "kv",
        "put",
        "kv/permissions/current",
        &format!("{}=very-secret", file_name),
      ])
      .output()?;

    import_vault_file("kv/permissions", &file_path, false)?;

    // Check file has 600 permissions (owner read/write only)
    let metadata = std::fs::metadata(&file_path)?;
    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt;
      let mode = metadata.permissions().mode();
      assert_eq!(mode & 0o777, 0o600, "File should have 600 permissions");
    }

    Ok(())
  }

  #[tokio::test]
  #[serial(environment)]
  async fn test_import_vault_file_subdir() -> anyhow::Result<()> {
    let _container = vault_container("vfile-subdir-test").await?;

    let temp_dir = TempDir::new()?;
    let path = "kv/test-app";
    let file = "secret.txt";
    let value = "top-secret-value";
    let dest = temp_dir.path().join("secret.txt");

    // Write test secret
    Command::new("vault")
      .args([
        "kv",
        "put",
        &format!("{path}/current"),
        &format!("{file}={value}"),
      ])
      .output()?;

    // Test import
    import_vault_file(path, &dest, false)?;

    // Verify file was created
    assert!(dest.exists());
    let content = std::fs::read_to_string(&dest)?;
    assert_eq!(content, value);

    // Check permissions are 600
    let metadata = std::fs::metadata(&dest)?;
    #[cfg(unix)]
    assert_eq!(metadata.permissions().mode() & 0o777, 0o600);

    Ok(())
  }
}
