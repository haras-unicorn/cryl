use std::path::Path;
use std::process::Command;

use crate::common::{read_file_if_exists, save_atomic, CrylError, CrylResult};

/// Generate a CockroachDB CA (certificate + key)
///
/// # Arguments
/// * `public` - Path to save the CA certificate (with public permissions 644)
/// * `private` - Path to save the CA private key (with private permissions 600)
/// * `renew` - Overwrite destinations if they exist
pub fn generate_cockroach_ca(
  public: &Path,
  private: &Path,
  renew: bool,
) -> CrylResult<()> {
  // Create temp directory for cockroach cert generation
  // Use the parent directory of the public path to ensure it's writable
  let tmp_dir = public
    .parent()
    .map(|p| p.join("cockroach.tmp"))
    .unwrap_or_else(|| Path::new("cockroach.tmp").to_path_buf());

  // Clean up any existing temp directory
  let _ = std::fs::remove_dir_all(&tmp_dir);

  // Create temp directory
  std::fs::create_dir_all(&tmp_dir)?;

  // Run cockroach cert create-ca to generate the CA
  let output = Command::new("cockroach")
    .arg("cert")
    .arg("create-ca")
    .arg("--certs-dir")
    .arg(&tmp_dir)
    .arg("--ca-key")
    .arg(tmp_dir.join("ca.key"))
    .output()?;

  if !output.status.success() {
    // Clean up temp directory on failure
    let _ = std::fs::remove_dir_all(&tmp_dir);

    return Err(CrylError::ToolExecution {
      tool: "cockroach cert create-ca".to_string(),
      exit_code: output.status.code().unwrap_or(-1),
      stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    });
  }

  // Read generated certificate files
  let public_path = tmp_dir.join("ca.crt");
  let private_path = tmp_dir.join("ca.key");

  let public_content = read_file_if_exists(&public_path)?.ok_or_else(|| {
    CrylError::Generation {
      generator: "cockroach-ca".to_string(),
      message: "Public certificate file not generated".to_string(),
    }
  })?;

  let private_content =
    read_file_if_exists(&private_path)?.ok_or_else(|| {
      CrylError::Generation {
        generator: "cockroach-ca".to_string(),
        message: "Private key file not generated".to_string(),
      }
    })?;

  // Clean up temp directory
  let _ = std::fs::remove_dir_all(&tmp_dir);

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
  fn test_generate_cockroach_ca_success() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let public_path = temp.path().join("ca.crt");
    let private_path = temp.path().join("ca.key");

    generate_cockroach_ca(&public_path, &private_path, true)?;

    // Check that both files exist
    assert!(public_path.exists());
    assert!(private_path.exists());

    // Check public certificate content - should be a PEM certificate
    let public_content = std::fs::read_to_string(&public_path)?;
    assert!(public_content.contains("-----BEGIN CERTIFICATE"));
    assert!(public_content.contains("-----END CERTIFICATE"));

    // Check private key content - should be a PEM private key
    let private_content = std::fs::read_to_string(&private_path)?;
    assert!(
      private_content.contains("-----BEGIN RSA PRIVATE KEY")
        || private_content.contains("-----BEGIN PRIVATE KEY")
    );
    assert!(
      private_content.contains("-----END RSA PRIVATE KEY")
        || private_content.contains("-----END PRIVATE KEY")
    );

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
  fn test_generate_cockroach_ca_no_renew() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let public_path = temp.path().join("ca.crt");
    let private_path = temp.path().join("ca.key");

    // Pre-create files
    std::fs::write(&public_path, "existing_public")?;
    std::fs::write(&private_path, "existing_private")?;

    // Generate with renew=false should not overwrite
    generate_cockroach_ca(&public_path, &private_path, false)?;

    let public_content = std::fs::read_to_string(&public_path)?;
    let private_content = std::fs::read_to_string(&private_path)?;

    assert_eq!(public_content, "existing_public");
    assert_eq!(private_content, "existing_private");

    Ok(())
  }

  #[test]
  fn test_generate_cockroach_ca_renew() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let public_path = temp.path().join("ca.crt");
    let private_path = temp.path().join("ca.key");

    // Pre-create files
    std::fs::write(&public_path, "existing_public")?;
    std::fs::write(&private_path, "existing_private")?;

    // Generate with renew=true should overwrite
    generate_cockroach_ca(&public_path, &private_path, true)?;

    let public_content = std::fs::read_to_string(&public_path)?;
    let private_content = std::fs::read_to_string(&private_path)?;

    // Should contain Cockroach CA content now
    assert!(public_content.contains("-----BEGIN CERTIFICATE"));
    assert!(
      private_content.contains("-----BEGIN")
        && private_content.contains("PRIVATE KEY")
    );

    Ok(())
  }

  #[test]
  fn test_generate_cockroach_ca_deterministic() -> anyhow::Result<()> {
    // Cockroach CA keys should be different on each generation
    let temp1 = TempDir::new()?;
    let temp2 = TempDir::new()?;

    let public_path1 = temp1.path().join("ca.crt");
    let private_path1 = temp1.path().join("ca.key");
    let public_path2 = temp2.path().join("ca.crt");
    let private_path2 = temp2.path().join("ca.key");

    generate_cockroach_ca(&public_path1, &private_path1, true)?;
    generate_cockroach_ca(&public_path2, &private_path2, true)?;

    let private1 = std::fs::read_to_string(&private_path1)?;
    let private2 = std::fs::read_to_string(&private_path2)?;
    let public1 = std::fs::read_to_string(&public_path1)?;
    let public2 = std::fs::read_to_string(&public_path2)?;

    // Keys should be different
    assert_ne!(private1, private2);
    assert_ne!(public1, public2);

    // Both should be valid certificates
    assert!(public1.contains("-----BEGIN CERTIFICATE"));
    assert!(public2.contains("-----BEGIN CERTIFICATE"));

    Ok(())
  }
}
