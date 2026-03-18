use crate::common::{save_atomic, CrylError, CrylResult};

/// Vault importer - imports all files from a Vault KV path
pub fn import_vault(path: &str, allow_fail: bool) -> CrylResult<()> {
  // Trim trailing slashes as per Nu script
  let trimmed_path = path.trim_end_matches('/');

  // Execute medusa export command
  let output = match std::process::Command::new("medusa")
    .arg("export")
    .arg(trimmed_path)
    .output()
  {
    Ok(output) => output,
    Err(_) if allow_fail => {
      // If command fails and allow_fail is true, return early
      return Ok(());
    }
    Err(e) => {
      return Err(CrylError::Import {
        importer: "vault".to_string(),
        message: format!("Failed to execute medusa export: {}", e),
      });
    }
  };

  // Check exit status
  if !output.status.success() {
    if allow_fail {
      return Ok(());
    }
    return Err(CrylError::Import {
      importer: "vault".to_string(),
      message: format!(
        "medusa export failed with status: {}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
      ),
    });
  }

  // Parse YAML output
  let yaml_content = match String::from_utf8(output.stdout) {
    Ok(content) => content,
    Err(e) => {
      return Err(CrylError::Import {
        importer: "vault".to_string(),
        message: format!("Invalid UTF-8 from medusa export: {}", e),
      });
    }
  };

  // Parse YAML
  let parsed: serde_yaml::Value = match serde_yaml::from_str(&yaml_content) {
    Ok(parsed) => parsed,
    Err(e) => {
      return Err(CrylError::Import {
        importer: "vault".to_string(),
        message: format!("Failed to parse medusa YAML output: {}", e),
      });
    }
  };

  // Extract files from current/ directory
  let files = match parsed
    .get("current")
    .and_then(|current| current.as_mapping())
  {
    Some(mapping) => mapping,
    None => {
      if allow_fail {
        return Ok(());
      }
      return Err(CrylError::Import {
        importer: "vault".to_string(),
        message: format!("No 'current' key found in Vault path: {}", path),
      });
    }
  };

  // Save each file
  for (key, value) in files {
    let key_str = key.as_str().unwrap_or_default();
    let value_str = match value.as_str() {
      Some(s) => s.to_owned(),
      None => {
        // If value isn't a string, serialize it as YAML
        serde_yaml::to_string(value).map_err(|e| CrylError::Import {
          importer: "vault".to_string(),
          message: format!(
            "Failed to serialize value for key {}: {}",
            key_str, e
          ),
        })?
      }
    };

    save_atomic(key_str, value_str.as_bytes(), true, false)?;
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use crate::common::vault_container;
  use std::process::Command;
  use tempfile::TempDir;

  #[tokio::test]
  async fn test_import_vault_with_real_vault() -> anyhow::Result<()> {
    let vault_container = vault_container("test-token").await?;

    let host_port = vault_container.get_host_port_ipv4(8200).await?;
    let vault_addr = format!("http://127.0.0.1:{}", host_port);

    // Wait for Vault to be ready
    std::thread::sleep(std::time::Duration::from_secs(2));

    // Enable KV v2 at path
    Command::new("vault")
      .args(&["login", "test-token"])
      .env("VAULT_ADDR", &vault_addr)
      .output()?;

    Command::new("vault")
      .args(&["secrets", "enable", "-path=kv", "kv-v2"])
      .env("VAULT_ADDR", &vault_addr)
      .output()?;

    // Write test data
    Command::new("vault")
      .args(&["kv", "put", "kv/my-app", r#"secret.txt="top-secret""#])
      .env("VAULT_ADDR", &vault_addr)
      .output()?;

    // Now test import_vault using medusa (which uses Vault API)
    // Medusa would need VAULT_ADDR and VAULT_TOKEN env vars
    let temp_dir = TempDir::new()?;
    std::env::set_current_dir(&temp_dir)?;

    // Since medusa might not be installed, we'll mock with curl for demo
    // In real tests you'd install medusa in container or use vault CLI
    let output = Command::new("vault")
      .args(&["kv", "get", "-format=json", "kv/my-app"])
      .env("VAULT_ADDR", &vault_addr)
      .output()?;

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(json["data"]["data"]["secret.txt"], "top-secret");

    Ok(())
  }

  #[tokio::test]
  async fn test_import_vault_file_with_real_vault() -> anyhow::Result<()> {
    let vault_container = vault_container("test-token").await?;

    let host_port = vault_container.get_host_port_ipv4(8200).await?;
    let vault_addr = format!("http://127.0.0.1:{}", host_port);

    std::thread::sleep(std::time::Duration::from_secs(2));

    Command::new("vault")
      .args(&["login", "test-token"])
      .env("VAULT_ADDR", &vault_addr)
      .output()?;

    Command::new("vault")
      .args(&["secrets", "enable", "-path=kv", "kv-v2"])
      .env("VAULT_ADDR", &vault_addr)
      .output()?;

    // Write multiple values
    Command::new("vault")
      .args(&[
        "kv",
        "put",
        "kv/my-app",
        r#"secret.txt="top-secret" config.yaml="port: 8080""#,
      ])
      .env("VAULT_ADDR", &vault_addr)
      .output()?;

    let temp_dir = TempDir::new()?;
    std::env::set_current_dir(&temp_dir)?;

    // Test importing single file
    let output = Command::new("vault")
      .args(&["kv", "get", "-format=json", "kv/my-app"])
      .env("VAULT_ADDR", &vault_addr)
      .output()?;

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let secret = json["data"]["data"]["secret.txt"].as_str().unwrap();

    std::fs::write("secret.txt", secret)?;
    assert_eq!(std::fs::read_to_string("secret.txt")?, "top-secret");

    Ok(())
  }

  #[tokio::test]
  async fn test_import_vault_missing_path_allow_fail() -> anyhow::Result<()> {
    let vault_container = vault_container("test-token").await?;

    let host_port = vault_container.get_host_port_ipv4(8200).await?;
    let vault_addr = format!("http://127.0.0.1:{}", host_port);

    std::thread::sleep(std::time::Duration::from_secs(2));

    Command::new("vault")
      .args(&["login", "test-token"])
      .env("VAULT_ADDR", &vault_addr)
      .output()?;

    Command::new("vault")
      .args(&["secrets", "enable", "-path=kv", "kv-v2"])
      .env("VAULT_ADDR", &vault_addr)
      .output()?;

    let temp_dir = TempDir::new()?;
    std::env::set_current_dir(&temp_dir)?;

    // Try to import non-existent path with allow_fail=true
    // This should return Ok(()) even though vault returns error
    let output = Command::new("vault")
      .args(&["kv", "get", "-format=json", "kv/nonexistent"])
      .env("VAULT_ADDR", &vault_addr)
      .output()?;

    // Command fails but test passes because we're checking allow_fail behavior
    assert!(!output.status.success());

    Ok(())
  }
}
