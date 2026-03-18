use crate::common::{save_atomic, CrylError, CrylResult};

/// Vault file importer - imports a single file from a Vault KV path
pub fn import_vault_file(
  path: &str,
  file: &str,
  allow_fail: bool,
) -> CrylResult<()> {
  // Trim trailing slashes
  let trimmed_path = path.trim_end_matches('/');
  let full_path = format!("{}/current", trimmed_path);

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
    .and_then(|inner| inner.get(file))
    .and_then(|value| value.as_str())
  {
    Some(content) => content.to_string(),
    None => {
      if allow_fail {
        return Ok(());
      }
      return Err(CrylError::Import {
        importer: "vault-file".to_string(),
        message: format!("File '{}' not found in Vault path: {}", file, path),
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
  use std::{os::unix::fs::PermissionsExt, process::Command};
  use tempfile::TempDir;

  #[tokio::test]
  async fn test_import_vault_file_success() -> anyhow::Result<()> {
    let vault_container = vault_container("test-token").await?;
    let host_port = vault_container.get_host_port_ipv4(8200).await?;
    let vault_addr = format!("http://127.0.0.1:{}", host_port);

    // Wait for Vault to be ready
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Setup Vault
    Command::new("vault")
      .args(["login", "test-token"])
      .env("VAULT_ADDR", &vault_addr)
      .output()?;

    Command::new("vault")
      .args(["secrets", "enable", "-path=kv", "kv-v2"])
      .env("VAULT_ADDR", &vault_addr)
      .output()?;

    // Write test secret
    Command::new("vault")
      .args([
        "kv",
        "put",
        "kv/test-app",
        "secret.txt=top-secret-value",
        "config.yaml=port: 8080",
      ])
      .env("VAULT_ADDR", &vault_addr)
      .output()?;

    let temp_dir = TempDir::new()?;
    std::env::set_current_dir(&temp_dir)?;

    // Test import
    import_vault_file("kv/test-app", "secret.txt", false)?;

    // Verify file was created
    assert!(std::path::Path::new("secret.txt").exists());
    let content = std::fs::read_to_string("secret.txt")?;
    assert_eq!(content, "top-secret-value");

    // Check permissions are 600
    let metadata = std::fs::metadata("secret.txt")?;
    #[cfg(unix)]
    assert_eq!(metadata.permissions().mode() & 0o777, 0o600);

    Ok(())
  }

  #[tokio::test]
  async fn test_import_vault_file_missing_key_allow_fail() -> anyhow::Result<()>
  {
    let vault_container = vault_container("test-token").await?;
    let host_port = vault_container.get_host_port_ipv4(8200).await?;
    let vault_addr = format!("http://127.0.0.1:{}", host_port);

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    Command::new("vault")
      .args(["login", "test-token"])
      .env("VAULT_ADDR", &vault_addr)
      .output()?;

    Command::new("vault")
      .args(["secrets", "enable", "-path=kv", "kv-v2"])
      .env("VAULT_ADDR", &vault_addr)
      .output()?;

    // Create secret with different key
    Command::new("vault")
      .args(["kv", "put", "kv/test-app", "other.txt=value"])
      .env("VAULT_ADDR", &vault_addr)
      .output()?;

    let temp_dir = TempDir::new()?;
    std::env::set_current_dir(&temp_dir)?;

    // Should not error with allow_fail=true
    import_vault_file("kv/test-app", "secret.txt", true)?;

    // File should not exist
    assert!(!std::path::Path::new("secret.txt").exists());

    Ok(())
  }

  #[tokio::test]
  async fn test_import_vault_file_missing_key_no_allow_fail(
  ) -> anyhow::Result<()> {
    let vault_container = vault_container("test-token").await?;
    let host_port = vault_container.get_host_port_ipv4(8200).await?;
    let vault_addr = format!("http://127.0.0.1:{}", host_port);

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    Command::new("vault")
      .args(["login", "test-token"])
      .env("VAULT_ADDR", &vault_addr)
      .output()?;

    Command::new("vault")
      .args(["secrets", "enable", "-path=kv", "kv-v2"])
      .env("VAULT_ADDR", &vault_addr)
      .output()?;

    Command::new("vault")
      .args(["kv", "put", "kv/test-app", "other.txt=value"])
      .env("VAULT_ADDR", &vault_addr)
      .output()?;

    let temp_dir = TempDir::new()?;
    std::env::set_current_dir(&temp_dir)?;

    // Should error with allow_fail=false
    let result = import_vault_file("kv/test-app", "secret.txt", false);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(matches!(err, CrylError::Import { importer, message: _ }
      if importer == "vault-file"));

    Ok(())
  }

  #[tokio::test]
  async fn test_import_vault_file_missing_path_allow_fail() -> anyhow::Result<()>
  {
    let vault_container = vault_container("test-token").await?;
    let host_port = vault_container.get_host_port_ipv4(8200).await?;
    let vault_addr = format!("http://127.0.0.1:{}", host_port);

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    Command::new("vault")
      .args(["login", "test-token"])
      .env("VAULT_ADDR", &vault_addr)
      .output()?;

    Command::new("vault")
      .args(["secrets", "enable", "-path=kv", "kv-v2"])
      .env("VAULT_ADDR", &vault_addr)
      .output()?;

    let temp_dir = TempDir::new()?;
    std::env::set_current_dir(&temp_dir)?;

    // Non-existent path with allow_fail=true should succeed
    import_vault_file("kv/nonexistent", "any.txt", true)?;
    assert!(!std::path::Path::new("any.txt").exists());

    Ok(())
  }

  #[tokio::test]
  async fn test_import_vault_file_vault_not_running_allow_fail(
  ) -> anyhow::Result<()> {
    // Test without starting container
    let temp_dir = TempDir::new()?;
    std::env::set_current_dir(&temp_dir)?;

    // Should succeed with allow_fail=true when vault command fails
    import_vault_file("kv/test", "file.txt", true)?;
    assert!(!std::path::Path::new("file.txt").exists());

    Ok(())
  }

  #[tokio::test]
  async fn test_import_vault_file_vault_not_running_no_allow_fail(
  ) -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    std::env::set_current_dir(&temp_dir)?;

    let result = import_vault_file("kv/test", "file.txt", false);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(matches!(err, CrylError::Import { importer, message: _ }
      if importer == "vault-file"));

    Ok(())
  }

  #[tokio::test]
  async fn test_import_vault_file_nested_path() -> anyhow::Result<()> {
    let vault_container = vault_container("test-token").await?;
    let host_port = vault_container.get_host_port_ipv4(8200).await?;
    let vault_addr = format!("http://127.0.0.1:{}", host_port);

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    Command::new("vault")
      .args(["login", "test-token"])
      .env("VAULT_ADDR", &vault_addr)
      .output()?;

    Command::new("vault")
      .args(["secrets", "enable", "-path=kv", "kv-v2"])
      .env("VAULT_ADDR", &vault_addr)
      .output()?;

    // Test with nested path
    Command::new("vault")
      .args(["kv", "put", "kv/team/project/env", "password=s3cr3t"])
      .env("VAULT_ADDR", &vault_addr)
      .output()?;

    let temp_dir = TempDir::new()?;
    std::env::set_current_dir(&temp_dir)?;

    // Import from nested path
    import_vault_file("kv/team/project/env", "password", false)?;

    assert!(std::path::Path::new("password").exists());
    let content = std::fs::read_to_string("password")?;
    assert_eq!(content, "s3cr3t");

    Ok(())
  }

  #[tokio::test]
  async fn test_import_vault_file_with_trailing_slash() -> anyhow::Result<()> {
    let vault_container = vault_container("test-token").await?;
    let host_port = vault_container.get_host_port_ipv4(8200).await?;
    let vault_addr = format!("http://127.0.0.1:{}", host_port);

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    Command::new("vault")
      .args(["login", "test-token"])
      .env("VAULT_ADDR", &vault_addr)
      .output()?;

    Command::new("vault")
      .args(["secrets", "enable", "-path=kv", "kv-v2"])
      .env("VAULT_ADDR", &vault_addr)
      .output()?;

    Command::new("vault")
      .args(["kv", "put", "kv/my-app", "config.yaml=key: value"])
      .env("VAULT_ADDR", &vault_addr)
      .output()?;

    let temp_dir = TempDir::new()?;
    std::env::set_current_dir(&temp_dir)?;

    // Path with trailing slash should still work
    import_vault_file("kv/my-app/", "config.yaml", false)?;

    assert!(std::path::Path::new("config.yaml").exists());
    let content = std::fs::read_to_string("config.yaml")?;
    assert_eq!(content, "key: value");

    Ok(())
  }

  #[tokio::test]
  async fn test_import_vault_file_binary_data() -> anyhow::Result<()> {
    let vault_container = vault_container("test-token").await?;
    let host_port = vault_container.get_host_port_ipv4(8200).await?;
    let vault_addr = format!("http://127.0.0.1:{}", host_port);

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    Command::new("vault")
      .args(["login", "test-token"])
      .env("VAULT_ADDR", &vault_addr)
      .output()?;

    Command::new("vault")
      .args(["secrets", "enable", "-path=kv", "kv-v2"])
      .env("VAULT_ADDR", &vault_addr)
      .output()?;

    // Store binary data (Vault encodes as base64 in JSON)
    let binary_data = vec![0x00, 0x01, 0x02, 0xFF];
    let encoded =
      base64::engine::general_purpose::STANDARD.encode(&binary_data);

    Command::new("vault")
      .args(["kv", "put", "kv/binary", &format!("data={}", encoded)])
      .env("VAULT_ADDR", &vault_addr)
      .output()?;

    let temp_dir = TempDir::new()?;
    std::env::set_current_dir(&temp_dir)?;

    import_vault_file("kv/binary", "data", false)?;

    // Vault returns base64-encoded strings in JSON, which gets decoded by serde_json
    // The function saves the string value as-is
    let content = std::fs::read("data")?;
    assert_eq!(content, encoded.as_bytes());

    Ok(())
  }

  #[tokio::test]
  async fn test_import_vault_file_permissions() -> anyhow::Result<()> {
    let vault_container = vault_container("test-token").await?;
    let host_port = vault_container.get_host_port_ipv4(8200).await?;
    let vault_addr = format!("http://127.0.0.1:{}", host_port);

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    Command::new("vault")
      .args(["login", "test-token"])
      .env("VAULT_ADDR", &vault_addr)
      .output()?;

    Command::new("vault")
      .args(["secrets", "enable", "-path=kv", "kv-v2"])
      .env("VAULT_ADDR", &vault_addr)
      .output()?;

    Command::new("vault")
      .args(["kv", "put", "kv/permissions", "secret=very-secret"])
      .env("VAULT_ADDR", &vault_addr)
      .output()?;

    let temp_dir = TempDir::new()?;
    std::env::set_current_dir(&temp_dir)?;

    import_vault_file("kv/permissions", "secret", false)?;

    // Check file has 600 permissions (owner read/write only)
    let metadata = std::fs::metadata("secret")?;
    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt;
      let mode = metadata.permissions().mode();
      assert_eq!(mode & 0o777, 0o600, "File should have 600 permissions");
    }

    Ok(())
  }
}
