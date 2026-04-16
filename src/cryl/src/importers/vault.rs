use std::{path::Path, process::Output};

use crate::common::{CrylError, CrylResult, save_atomic};

/// Vault importer - imports all files from a Vault KV path to current directory
pub fn import_vault(path: &str, allow_fail: bool) -> CrylResult<()> {
  // Function to process output of commands into a YAML mapping
  fn process_output_into_yaml_mapping(
    path: &str,
    prefix: &[&str],
    allow_fail: bool,
    output_result: std::io::Result<Output>,
  ) -> CrylResult<Option<serde_yaml::Mapping>> {
    let output = match output_result {
      Ok(output) => output,
      Err(_) if allow_fail => {
        // If command fails and allow_fail is true, return early
        return Ok(None);
      }
      Err(e) => {
        return Err(CrylError::Import {
          importer: "vault".to_string(),
          message: format!("Failed to execute command: {}", e),
        });
      }
    };

    // Check exit status
    if !output.status.success() {
      if allow_fail {
        return Ok(None);
      }
      return Err(CrylError::Import {
        importer: "vault".to_string(),
        message: format!(
          "Command failed with status: {}\nstderr: {}",
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
          message: format!("Invalid UTF-8 from command: {}", e),
        });
      }
    };

    // Parse YAML
    let parsed: serde_yaml::Value = match serde_yaml::from_str(&yaml_content) {
      Ok(parsed) => parsed,
      Err(e) => {
        return Err(CrylError::Import {
          importer: "vault".to_string(),
          message: format!("Failed to parse command YAML output: {}", e),
        });
      }
    };

    // Extract files from current/ directory
    let files = match prefix
      .iter()
      .try_fold(&parsed, |acc, next| acc.get(*next))
      .and_then(|current| current.as_mapping())
    {
      Some(mapping) => mapping,
      None => {
        if allow_fail {
          return Ok(None);
        }
        return Err(CrylError::Import {
          importer: "vault".to_string(),
          message: format!("Invalid type for key: {}", path),
        });
      }
    };

    Ok(Some(files.clone()))
  }

  // Function to save files from YAML mapping
  fn save_atomic_recursive_from_yaml_mapping(
    path: &Path,
    files: &serde_yaml::Mapping,
  ) -> CrylResult<()> {
    for (key, value) in files {
      let key_str = key.as_str().unwrap_or_default();

      if let Some(value_str) = value.as_str() {
        save_atomic(path.join(key_str), value_str.as_bytes(), true, false)?;
      } else if let Some(next_files) = value.as_mapping() {
        save_atomic_recursive_from_yaml_mapping(
          &path.join(key_str),
          next_files,
        )?;
      } else {
        return Err(CrylError::Import {
          importer: "vault".to_string(),
          message: format!("Unknown file type of {key_str}"),
        });
      }
    }

    Ok(())
  }

  // Trim trailing slashes
  let trimmed_path = path.trim_end_matches('/');

  // Extract last component now to exit early
  // and because medusa puts everything under that
  let Some(last_component) = path.split("/").last().to_owned() else {
    return Err(CrylError::Import {
      importer: "vault".to_string(),
      message: "Path is empty".to_string(),
    });
  };

  // Execute medusa export command
  let medusa_result = process_output_into_yaml_mapping(
    path,
    &[last_component],
    allow_fail,
    std::process::Command::new("medusa")
      .arg("export")
      .arg(trimmed_path)
      .output(),
  );

  // Execute vault kv get command
  let vault_result = process_output_into_yaml_mapping(
    path,
    &["data", "data"],
    allow_fail,
    std::process::Command::new("vault")
      .arg("kv")
      .arg("get")
      .arg("-format=yaml")
      .arg(trimmed_path)
      .output(),
  );

  // Save whatever we got
  if let Ok(Some(medusa_files)) = &medusa_result {
    save_atomic_recursive_from_yaml_mapping(
      &std::env::current_dir()?,
      medusa_files,
    )?;
  }
  if let Ok(Some(vault_files)) = &vault_result {
    save_atomic_recursive_from_yaml_mapping(
      &std::env::current_dir()?,
      vault_files,
    )?;
  }

  // Exit if both failed
  // medusa fails if the target doesn't exist
  // or if the target is just a secret and not a directory
  // vault is the opposite
  // so we only return an error if they both fail
  // not a perfect implementation but a pragmatic one
  if let Err(medusa_error) = medusa_result
    && let Err(vault_error) = vault_result
  {
    return Err(CrylError::multiple([medusa_error, vault_error]));
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::common::{TempCurrentDir, vault_container};
  use serial_test::serial;
  use std::process::Command;

  #[tokio::test]
  #[serial(environment)]
  #[serial(working_directory)]
  async fn test_import_vault_with_real_vault() -> anyhow::Result<()> {
    let _container = vault_container("vault-import-test").await?;
    let key = "kv/my-app";
    let file = "secret.txt";
    let content = "top-secret";

    // Write test data
    Command::new("vault")
      .args(["kv", "put", &key, &format!("{file}={content}")])
      .output()?;

    // Now test import_vault using medusa (which uses Vault API)
    let _temp = TempCurrentDir::new()?;
    import_vault(key, false)?;

    // Check file content is ok
    let result = std::fs::read_to_string(file)?;
    assert_eq!(result, content);

    Ok(())
  }

  #[tokio::test]
  #[serial(environment)]
  #[serial(working_directory)]
  async fn test_import_vault_multiple_with_real_vault() -> anyhow::Result<()> {
    let _container = vault_container("vault-file-test").await?;
    let key = "kv/my-app";
    let first_file = "secret.txt";
    let first_content = "top-secret";
    let second_file = "config.yaml";
    let second_content = "port: 8080";

    // Write multiple values
    Command::new("vault")
      .args([
        "kv",
        "put",
        &key,
        &format!("{first_file}={first_content}"),
        &format!("{second_file}={second_content}"),
      ])
      .output()?;

    // Now test import_vault using medusa (which uses Vault API)
    let _temp = TempCurrentDir::new()?;
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
  #[serial(environment)]
  #[serial(working_directory)]
  async fn test_import_vault_mixed_depth() -> anyhow::Result<()> {
    let _container = vault_container("vault-mixed-test").await?;
    let key = "kv/my-app";

    // Root file + nested directory
    let root_file = "config.yaml";
    let root_content = "port: 8080";
    let subdir = "secrets";
    let nested_file = "api-key.txt";
    let nested_content = "abc123";

    Command::new("vault")
      .args([
        "kv",
        "put",
        &key,
        &format!("{root_file}={root_content}"),
        &format!("{subdir}/{nested_file}={nested_content}"),
      ])
      .output()?;

    let _temp = TempCurrentDir::new()?;
    import_vault(key, false)?;

    // Check root file
    assert_eq!(std::fs::read_to_string(root_file)?, root_content);

    // Check nested structure
    assert_eq!(
      std::fs::read_to_string(format!("{subdir}/{nested_file}"))?,
      nested_content
    );

    Ok(())
  }

  #[tokio::test]
  #[serial(environment)]
  #[serial(working_directory)]
  async fn test_import_vault_deep_nesting() -> anyhow::Result<()> {
    let _container = vault_container("vault-nested-test").await?;
    let key = "kv/my-app";

    // Only nested files at depth (no root files)
    let deep_path = "config/production/database";
    let deep_file = "connection.yaml";
    let deep_content = "host: localhost\nport: 5432";

    Command::new("vault")
      .args([
        "kv",
        "put",
        &key,
        &format!("{deep_path}/{deep_file}={deep_content}"),
      ])
      .output()?;

    let _temp = TempCurrentDir::new()?;
    import_vault(key, false)?;

    // Ensure directory structure created
    assert!(Path::new(&deep_path).exists());

    // Verify deep file content
    assert_eq!(
      std::fs::read_to_string(format!("{deep_path}/{deep_file}"))?,
      deep_content
    );

    // Verify no unexpected root files
    let root_entries: Vec<_> = std::fs::read_dir(std::env::current_dir()?)?
      .filter_map(|e| e.ok())
      .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
      .map(|e| e.file_name().into_string().unwrap())
      .collect();
    assert!(
      !root_entries.contains(&deep_file.to_string()),
      "Should have no root files with this vault structure"
    );

    Ok(())
  }

  #[tokio::test]
  #[serial(environment)]
  #[serial(working_directory)]
  async fn test_import_vault_missing_path_allow_fail() -> anyhow::Result<()> {
    let _container = vault_container("vault-missing-test").await?;

    // Test random non-existent key
    let _temp = TempCurrentDir::new()?;
    import_vault("kv/nonexistent", true)?;

    Ok(())
  }
}
