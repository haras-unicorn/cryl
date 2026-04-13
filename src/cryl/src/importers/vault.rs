use crate::common::{CrylError, CrylResult, save_atomic};

/// Vault importer - imports all files from a Vault KV path
pub fn import_vault(path: &str, allow_fail: bool) -> CrylResult<()> {
  // Trim trailing slashes as per Nu script
  let trimmed_path = path.trim_end_matches('/');

  // Extract last component now to exit early
  // and because medusa puts everything under that
  let Some(last_component) = trimmed_path.split("/").last().to_owned() else {
    return Err(CrylError::Import {
      importer: "vault".to_string(),
      message: format!("Path is empty"),
    });
  };

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
    .get(last_component)
    .and_then(|value| value.get("current"))
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
  use super::*;
  use crate::common::vault_container;
  use serial_test::serial;
  use std::process::Command;
  use tempfile::TempDir;

  #[tokio::test]
  #[serial]
  async fn test_import_vault_with_real_vault() -> anyhow::Result<()> {
    let _container = vault_container("vault-import-test").await?;
    let key = "kv/my-app";
    let key_current = format!("{}/current", key);
    let file = "secret.txt";
    let content = "top-secret";

    // Write test data
    Command::new("vault")
      .args(["kv", "put", &key_current, &format!("{file}={content}")])
      .output()?;

    // Now test import_vault using medusa (which uses Vault API)
    let temp_dir = TempDir::new()?;
    std::env::set_current_dir(&temp_dir)?;
    import_vault(key, false)?;

    // Check file content is ok
    let result = std::fs::read_to_string(file)?;
    assert_eq!(result, content);

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_import_vault_multiple_with_real_vault() -> anyhow::Result<()> {
    let _container = vault_container("vault-file-test").await?;
    let key = "kv/my-app";
    let key_current = format!("{}/current", key);
    let first_file = "secret.txt";
    let first_content = "top-secret";
    let second_file = "config.yaml";
    let second_content = "port: 8080";

    // Write multiple values
    Command::new("vault")
      .args([
        "kv",
        "put",
        &key_current,
        &format!("{first_file}={first_content}"),
        &format!("{second_file}={second_content}"),
      ])
      .output()?;

    // Now test import_vault using medusa (which uses Vault API)
    let temp_dir = TempDir::new()?;
    std::env::set_current_dir(&temp_dir)?;
    import_vault(key, false)?;

    // Check first file content is ok
    let result = std::fs::read_to_string(first_file)?;
    assert_eq!(result, first_content);

    // Check second file content is ok
    let result = std::fs::read_to_string(second_file)?;
    assert_eq!(result, second_content);

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_import_vault_missing_path_allow_fail() -> anyhow::Result<()> {
    let _container = vault_container("vault-missing-test").await?;

    // Test random non-existent key
    let temp_dir = TempDir::new()?;
    std::env::set_current_dir(&temp_dir)?;
    import_vault("kv/nonexistent", true)?;

    Ok(())
  }
}
