use crate::common::{
  CrylError, CrylResult, DirectoryListing, Format, InMemoryFilesystem,
  deserialize_from_file, list_directory,
};
use std::path::Path;

/// Vault exporter - exports all files in directory listing
pub fn export_vault(
  path: &str,
  format: Format,
  listing: &Path,
) -> CrylResult<()> {
  // Get listing first to exit early
  let listing: DirectoryListing = deserialize_from_file(listing, Some(format))?;

  // Trim trailing slashes
  let trimmed_path = path.trim_end_matches('/');

  // List directory
  let files = list_directory(std::env::current_dir()?, &listing, false, "/")?;

  // Return early to avoid sending an empty request
  if files.is_empty() {
    return Ok(());
  }

  // Convert to intermediary filesystem
  let filesystem = InMemoryFilesystem::from_listing(files, "/");

  // Convert to yaml
  let yaml_map: serde_yaml::Value = filesystem.into();

  // Convert to yaml string
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
  use crate::common::{Format, TempCurrentDir, vault_container};
  use serial_test::serial;
  use std::{
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
  };

  #[tokio::test]
  #[serial(environment)]
  #[serial(working_directory)]
  async fn test_export_vault_success() -> anyhow::Result<()> {
    let _container = vault_container("vault-export-test").await?;

    // Create test files
    let _temp = TempCurrentDir::new()?;
    std::fs::write("secret.txt", "top-secret")?;
    std::fs::write("config.yaml", "port: 8080")?;
    std::fs::write("listing.json", r#"{ "type": "flat" }"#)?;

    // Export to vault
    super::export_vault("kv/my-app", Format::Json, &Path::new("listing.json"))?;

    // Verify using vault CLI
    let output = Command::new("vault")
      .args(["kv", "get", "-format=json", "kv/my-app"])
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
  #[serial(environment)]
  #[serial(working_directory)]
  async fn test_export_vault_empty_directory() -> anyhow::Result<()> {
    let _container = vault_container("vault-export-empty-test").await?;

    let _temp = TempCurrentDir::new()?;
    std::fs::write("listing.json", r#"{ "type": "flat" }"#)?;

    // Export from empty directory should succeed
    super::export_vault(
      "kv/empty-app",
      Format::Json,
      &Path::new("listing.json"),
    )?;

    Ok(())
  }

  #[tokio::test]
  #[serial(environment)]
  #[serial(working_directory)]
  async fn test_export_vault_with_trailing_slash() -> anyhow::Result<()> {
    let _container = vault_container("vault-export-slash-test").await?;

    let _temp = TempCurrentDir::new()?;
    std::fs::write("data.txt", "test-data")?;
    std::fs::write("listing.json", r#"{ "type": "flat" }"#)?;

    // Path with trailing slash should work
    super::export_vault(
      "kv/slash-app/",
      Format::Json,
      &Path::new("listing.json"),
    )?;

    // Verify
    let output = Command::new("vault")
      .args(["kv", "get", "-format=json", "kv/slash-app"])
      .output()?;

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(json["data"]["data"]["data.txt"], "test-data");

    Ok(())
  }

  #[tokio::test]
  #[serial(environment)]
  #[serial(working_directory)]
  async fn test_export_vault_subdir() -> anyhow::Result<()> {
    let _container = vault_container("vault-export-subdir-test").await?;
    let _temp = TempCurrentDir::new()?;

    let dir = PathBuf::from_str("subdir").unwrap();
    let first_key = "secret.txt";
    let first_file = dir.join(first_key);
    let first_content = "top-secret";
    let second_key = "config.yaml";
    let second_file = dir.join(second_key);
    let second_content = "port: 8080";
    let key = "kv/my-app";

    // Create test files
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(first_file, first_content)?;
    std::fs::write(second_file, second_content)?;
    std::fs::write("listing.json", r#"{ "type": "deep" }"#)?;

    // Export to vault
    super::export_vault(key, Format::Json, &Path::new("listing.json"))?;

    // Verify using medusa
    let output = Command::new("medusa").args(["export", "kv"]).output()?;

    if !output.status.success() {
      anyhow::bail!(
        "vault kv get failed: {}",
        String::from_utf8_lossy(&output.stderr)
      );
    }

    let yaml: serde_json::Value = serde_yaml::from_slice(&output.stdout)?;
    assert_eq!(yaml["my-app"]["subdir"][first_key], first_content);
    assert_eq!(yaml["my-app"]["subdir"][second_key], second_content);
    assert_eq!(yaml["my-app"]["listing.json"], r#"{ "type": "deep" }"#);

    Ok(())
  }
}
