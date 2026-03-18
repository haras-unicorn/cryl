use std::path::Path;
use std::process::Command;

use crate::common::{read_file_if_exists, save_atomic, CrylError, CrylResult};

/// Generate a Nebula CA (certificate + key)
///
/// # Arguments
/// * `name` - Common name for the Nebula CA
/// * `public` - Path to save the CA certificate (with public permissions 644)
/// * `private` - Path to save the CA private key (with private permissions 600)
/// * `days` - Certificate validity in days
/// * `renew` - Overwrite destinations if they exist
pub fn generate_nebula_ca(
  name: &str,
  public: &Path,
  private: &Path,
  days: u32,
  renew: bool,
) -> CrylResult<()> {
  // Create temp file paths with .tmp suffix appended (matching original nushell implementation)
  let tmp_public = public.as_os_str().to_string_lossy().to_string() + ".tmp";
  let tmp_private = private.as_os_str().to_string_lossy().to_string() + ".tmp";

  // Calculate duration in hours
  let duration = format!("{}h", days * 24);

  // Run nebula-cert ca to generate the CA
  let output = Command::new("nebula-cert")
    .arg("ca")
    .arg("-name")
    .arg(name)
    .arg("-duration")
    .arg(&duration)
    .arg("-out-crt")
    .arg(&tmp_public)
    .arg("-out-key")
    .arg(&tmp_private)
    .output()?;

  if !output.status.success() {
    // Clean up temp files on failure
    let _ = std::fs::remove_file(Path::new(&tmp_public));
    let _ = std::fs::remove_file(Path::new(&tmp_private));

    return Err(CrylError::ToolExecution {
      tool: "nebula-cert ca".to_string(),
      exit_code: output.status.code().unwrap_or(-1),
      stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    });
  }

  // Read generated certificate files
  let public_content = read_file_if_exists(Path::new(&tmp_public))?
    .ok_or_else(|| CrylError::Generation {
      generator: "nebula-ca".to_string(),
      message: "Public certificate file not generated".to_string(),
    })?;

  let private_content = read_file_if_exists(Path::new(&tmp_private))?
    .ok_or_else(|| CrylError::Generation {
      generator: "nebula-ca".to_string(),
      message: "Private key file not generated".to_string(),
    })?;

  // Clean up temp files
  let _ = std::fs::remove_file(Path::new(&tmp_public));
  let _ = std::fs::remove_file(Path::new(&tmp_private));

  // Save public certificate with public permissions (644)
  save_atomic(public, public_content.as_bytes(), renew, true)?;

  // Save private key with private permissions (600)
  save_atomic(private, private_content.as_bytes(), renew, false)?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::os::unix::fs::PermissionsExt;
  use tempfile::TempDir;

  #[test]
  fn test_generate_nebula_ca_success() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let public_path = temp.path().join("ca.crt");
    let private_path = temp.path().join("ca.key");

    generate_nebula_ca("Test CA", &public_path, &private_path, 3650, true)?;

    // Check that both files exist
    assert!(public_path.exists());
    assert!(private_path.exists());

    // Check public certificate content - should be a PEM certificate
    let public_content = std::fs::read_to_string(&public_path)?;
    assert!(public_content.contains("-----BEGIN NEBULA CERTIFICATE"));
    assert!(public_content.contains("-----END NEBULA CERTIFICATE"));

    // Check private key content - should be a PEM private key
    let private_content = std::fs::read_to_string(&private_path)?;
    assert!(private_content.contains("-----BEGIN NEBULA"));
    assert!(private_content.contains("-----END NEBULA"));

    // Check permissions - private should be 600
    let private_metadata = std::fs::metadata(&private_path)?;
    let private_perms = private_metadata.permissions();
    assert_eq!(private_perms.mode() & 0o777, 0o600);

    // Check permissions - public should be 644
    let public_metadata = std::fs::metadata(&public_path)?;
    let public_perms = public_metadata.permissions();
    assert_eq!(public_perms.mode() & 0o777, 0o644);

    Ok(())
  }

  #[test]
  fn test_generate_nebula_ca_custom_days() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let public_path = temp.path().join("ca.crt");
    let private_path = temp.path().join("ca.key");

    generate_nebula_ca("Test CA", &public_path, &private_path, 365, true)?;

    // Check that both files exist
    assert!(public_path.exists());
    assert!(private_path.exists());

    // Check certificate was generated successfully
    let public_content = std::fs::read_to_string(&public_path)?;
    assert!(public_content.contains("-----BEGIN NEBULA CERTIFICATE"));

    Ok(())
  }

  #[test]
  fn test_generate_nebula_ca_no_renew() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let public_path = temp.path().join("ca.crt");
    let private_path = temp.path().join("ca.key");

    // Pre-create files
    std::fs::write(&public_path, "existing_public")?;
    std::fs::write(&private_path, "existing_private")?;

    // Generate with renew=false should not overwrite
    generate_nebula_ca("Test CA", &public_path, &private_path, 3650, false)?;

    let public_content = std::fs::read_to_string(&public_path)?;
    let private_content = std::fs::read_to_string(&private_path)?;

    assert_eq!(public_content, "existing_public");
    assert_eq!(private_content, "existing_private");

    Ok(())
  }

  #[test]
  fn test_generate_nebula_ca_renew() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let public_path = temp.path().join("ca.crt");
    let private_path = temp.path().join("ca.key");

    // Pre-create files
    std::fs::write(&public_path, "existing_public")?;
    std::fs::write(&private_path, "existing_private")?;

    // Generate with renew=true should overwrite
    generate_nebula_ca("Test CA", &public_path, &private_path, 3650, true)?;

    let public_content = std::fs::read_to_string(&public_path)?;
    let private_content = std::fs::read_to_string(&private_path)?;

    // Should contain Nebula CA content now
    assert!(public_content.contains("-----BEGIN NEBULA CERTIFICATE"));
    assert!(private_content.contains("-----BEGIN NEBULA"));

    Ok(())
  }

  #[test]
  fn test_generate_nebula_ca_deterministic() -> anyhow::Result<()> {
    // Nebula CA keys should be different on each generation
    let temp1 = TempDir::new()?;
    let temp2 = TempDir::new()?;

    let public_path1 = temp1.path().join("ca.crt");
    let private_path1 = temp1.path().join("ca.key");
    let public_path2 = temp2.path().join("ca.crt");
    let private_path2 = temp2.path().join("ca.key");

    generate_nebula_ca("Test CA 1", &public_path1, &private_path1, 3650, true)?;
    generate_nebula_ca("Test CA 2", &public_path2, &private_path2, 3650, true)?;

    let private1 = std::fs::read_to_string(&private_path1)?;
    let private2 = std::fs::read_to_string(&private_path2)?;
    let public1 = std::fs::read_to_string(&public_path1)?;
    let public2 = std::fs::read_to_string(&public_path2)?;

    // Keys should be different
    assert_ne!(private1, private2);
    assert_ne!(public1, public2);

    // Both should be valid Nebula certificates
    assert!(public1.contains("-----BEGIN NEBULA CERTIFICATE"));
    assert!(public2.contains("-----BEGIN NEBULA CERTIFICATE"));

    Ok(())
  }
}
