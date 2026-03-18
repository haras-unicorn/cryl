use std::path::Path;
use std::process::Command;

use crate::common::{read_file_if_exists, save_atomic, CrylError, CrylResult};

/// Generate a CockroachDB client certificate (for a specific user)
///
/// # Arguments
/// * `ca_public` - Path to the CockroachDB CA certificate
/// * `ca_private` - Path to the CockroachDB CA private key
/// * `public` - Path to save the client certificate (with public permissions 644)
/// * `private` - Path to save the client private key (with private permissions 600)
/// * `user` - CockroachDB username for the client cert
/// * `renew` - Overwrite destinations if they exist
pub fn generate_cockroach_client_cert(
  ca_public: &Path,
  ca_private: &Path,
  public: &Path,
  private: &Path,
  user: &str,
  renew: bool,
) -> CrylResult<()> {
  // Create temp directory for cockroach cert generation
  let tmp_dir = public
    .parent()
    .map(|p| p.join("cockroach.tmp"))
    .unwrap_or_else(|| Path::new("cockroach.tmp").to_path_buf());

  // Clean up any existing temp directory
  let _ = std::fs::remove_dir_all(&tmp_dir);

  // Create temp directory
  std::fs::create_dir_all(&tmp_dir)?;

  // Copy CA files to temp directory
  let ca_key_dest = tmp_dir.join("ca.key");
  let ca_crt_dest = tmp_dir.join("ca.crt");

  let ca_key_content =
    read_file_if_exists(ca_private)?.ok_or_else(|| CrylError::Generation {
      generator: "cockroach-client-cert".to_string(),
      message: format!("CA private key not found at {}", ca_private.display()),
    })?;

  let ca_crt_content =
    read_file_if_exists(ca_public)?.ok_or_else(|| CrylError::Generation {
      generator: "cockroach-client-cert".to_string(),
      message: format!(
        "CA public certificate not found at {}",
        ca_public.display()
      ),
    })?;

  std::fs::write(&ca_key_dest, ca_key_content)?;
  std::fs::write(&ca_crt_dest, ca_crt_content)?;

  // Build cockroach cert create-client command
  let output = Command::new("cockroach")
    .arg("cert")
    .arg("create-client")
    .arg(user)
    .arg("--certs-dir")
    .arg(&tmp_dir)
    .arg("--ca-key")
    .arg(&ca_key_dest)
    .output()?;

  if !output.status.success() {
    // Clean up temp directory on failure
    let _ = std::fs::remove_dir_all(&tmp_dir);

    return Err(CrylError::ToolExecution {
      tool: "cockroach cert create-client".to_string(),
      exit_code: output.status.code().unwrap_or(-1),
      stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    });
  }

  // Read generated certificate files
  let public_path = tmp_dir.join(format!("client.{}.crt", user));
  let private_path = tmp_dir.join(format!("client.{}.key", user));

  let public_content = read_file_if_exists(&public_path)?.ok_or_else(|| {
    CrylError::Generation {
      generator: "cockroach-client-cert".to_string(),
      message: "Client certificate file not generated".to_string(),
    }
  })?;

  let private_content =
    read_file_if_exists(&private_path)?.ok_or_else(|| {
      CrylError::Generation {
        generator: "cockroach-client-cert".to_string(),
        message: "Client private key file not generated".to_string(),
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

  fn create_test_ca(
    temp: &TempDir,
  ) -> anyhow::Result<(std::path::PathBuf, std::path::PathBuf)> {
    use crate::generators::generate_cockroach_ca;

    let ca_public = temp.path().join("ca.crt");
    let ca_private = temp.path().join("ca.key");

    generate_cockroach_ca(&ca_public, &ca_private, true)?;

    Ok((ca_public, ca_private))
  }

  #[test]
  fn test_generate_cockroach_client_cert_success() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let (ca_public, ca_private) = create_test_ca(&temp)?;

    let public_path = temp.path().join("client.crt");
    let private_path = temp.path().join("client.key");

    generate_cockroach_client_cert(
      &ca_public,
      &ca_private,
      &public_path,
      &private_path,
      "test-user",
      true,
    )?;

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
  fn test_generate_cockroach_client_cert_different_users() -> anyhow::Result<()>
  {
    let temp = TempDir::new()?;
    let (ca_public, ca_private) = create_test_ca(&temp)?;

    let public_path1 = temp.path().join("client1.crt");
    let private_path1 = temp.path().join("client1.key");
    let public_path2 = temp.path().join("client2.crt");
    let private_path2 = temp.path().join("client2.key");

    // Generate certificates for different users
    generate_cockroach_client_cert(
      &ca_public,
      &ca_private,
      &public_path1,
      &private_path1,
      "user1",
      true,
    )?;
    generate_cockroach_client_cert(
      &ca_public,
      &ca_private,
      &public_path2,
      &private_path2,
      "user2",
      true,
    )?;

    // Both certificates should be valid but different
    let public1 = std::fs::read_to_string(&public_path1)?;
    let public2 = std::fs::read_to_string(&public_path2)?;
    let private1 = std::fs::read_to_string(&private_path1)?;
    let private2 = std::fs::read_to_string(&private_path2)?;

    // Both should be valid certificates
    assert!(public1.contains("-----BEGIN CERTIFICATE"));
    assert!(public2.contains("-----BEGIN CERTIFICATE"));

    // Keys should be different for different users
    assert_ne!(private1, private2);
    assert_ne!(public1, public2);

    Ok(())
  }

  #[test]
  fn test_generate_cockroach_client_cert_no_renew() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let (ca_public, ca_private) = create_test_ca(&temp)?;

    let public_path = temp.path().join("client.crt");
    let private_path = temp.path().join("client.key");

    // Pre-create files
    std::fs::write(&public_path, "existing_public")?;
    std::fs::write(&private_path, "existing_private")?;

    // Generate with renew=false should not overwrite
    generate_cockroach_client_cert(
      &ca_public,
      &ca_private,
      &public_path,
      &private_path,
      "test-user",
      false,
    )?;

    let public_content = std::fs::read_to_string(&public_path)?;
    let private_content = std::fs::read_to_string(&private_path)?;

    assert_eq!(public_content, "existing_public");
    assert_eq!(private_content, "existing_private");

    Ok(())
  }

  #[test]
  fn test_generate_cockroach_client_cert_renew() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let (ca_public, ca_private) = create_test_ca(&temp)?;

    let public_path = temp.path().join("client.crt");
    let private_path = temp.path().join("client.key");

    // Pre-create files
    std::fs::write(&public_path, "existing_public")?;
    std::fs::write(&private_path, "existing_private")?;

    // Generate with renew=true should overwrite
    generate_cockroach_client_cert(
      &ca_public,
      &ca_private,
      &public_path,
      &private_path,
      "test-user",
      true,
    )?;

    let public_content = std::fs::read_to_string(&public_path)?;
    let private_content = std::fs::read_to_string(&private_path)?;

    // Should contain Cockroach client certificate content now
    assert!(public_content.contains("-----BEGIN CERTIFICATE"));
    assert!(
      private_content.contains("-----BEGIN")
        && private_content.contains("PRIVATE KEY")
    );

    Ok(())
  }

  #[test]
  fn test_generate_cockroach_client_cert_deterministic() -> anyhow::Result<()> {
    // Client certificates should be different on each generation
    let temp = TempDir::new()?;
    let (ca_public, ca_private) = create_test_ca(&temp)?;

    let public_path1 = temp.path().join("client1.crt");
    let private_path1 = temp.path().join("client1.key");
    let public_path2 = temp.path().join("client2.crt");
    let private_path2 = temp.path().join("client2.key");

    generate_cockroach_client_cert(
      &ca_public,
      &ca_private,
      &public_path1,
      &private_path1,
      "test-user",
      true,
    )?;
    generate_cockroach_client_cert(
      &ca_public,
      &ca_private,
      &public_path2,
      &private_path2,
      "test-user",
      true,
    )?;

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

  #[test]
  fn test_generate_cockroach_client_cert_missing_ca() -> anyhow::Result<()> {
    let temp = TempDir::new()?;

    let ca_public = temp.path().join("nonexistent.crt");
    let ca_private = temp.path().join("nonexistent.key");
    let public_path = temp.path().join("client.crt");
    let private_path = temp.path().join("client.key");

    // Should fail when CA files don't exist
    let result = generate_cockroach_client_cert(
      &ca_public,
      &ca_private,
      &public_path,
      &private_path,
      "test-user",
      true,
    );

    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    // The error message should indicate that the CA file was not found
    assert!(
      err_msg.contains("not found") || err_msg.contains("No such file"),
      "Expected 'not found' error, got: {}",
      err_msg
    );

    Ok(())
  }
}
