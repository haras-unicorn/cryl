use std::path::Path;
use std::process::Command;

use crate::common::{CrylError, CrylResult, read_file_if_exists, save_atomic};

/// Generate a Nebula node certificate (signed by a Nebula CA)
///
/// # Arguments
/// * `ca_public` - Path to the Nebula CA certificate (public)
/// * `ca_private` - Path to the Nebula CA private key
/// * `name` - Common name for the node cert
/// * `ip` - Node IP in CIDR or IP form (e.g., "10.1.1.5/24" or "10.1.1.5")
/// * `public` - Path to save the node certificate
/// * `private` - Path to save the node private key
/// * `renew` - Overwrite destinations if they exist
pub fn generate_nebula_cert(
  ca_public: &Path,
  ca_private: &Path,
  name: &str,
  ip: &str,
  public: &Path,
  private: &Path,
  renew: bool,
) -> CrylResult<()> {
  // Create temp file paths with .tmp suffix appended (matching original nushell implementation)
  let tmp_public = public.as_os_str().to_string_lossy().to_string() + ".tmp";
  let tmp_private = private.as_os_str().to_string_lossy().to_string() + ".tmp";

  // Run nebula-cert sign to generate the node certificate
  let output = Command::new("nebula-cert")
    .arg("sign")
    .arg("-ca-crt")
    .arg(ca_public)
    .arg("-ca-key")
    .arg(ca_private)
    .arg("-name")
    .arg(name)
    .arg("-ip")
    .arg(ip)
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
      tool: "nebula-cert sign".to_string(),
      exit_code: output.status.code().unwrap_or(-1),
      stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    });
  }

  // Read generated certificate files
  let public_content = read_file_if_exists(Path::new(&tmp_public))?
    .ok_or_else(|| CrylError::Generation {
      generator: "nebula-cert".to_string(),
      message: "Public certificate file not generated".to_string(),
    })?;

  let private_content = read_file_if_exists(Path::new(&tmp_private))?
    .ok_or_else(|| CrylError::Generation {
      generator: "nebula-cert".to_string(),
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

  use crate::generators::generate_nebula_ca;

  #[test]
  fn test_generate_nebula_cert_success() -> anyhow::Result<()> {
    let temp = TempDir::new()?;

    // First, create a CA
    let ca_public_path = temp.path().join("ca.crt");
    let ca_private_path = temp.path().join("ca.key");
    generate_nebula_ca(
      "Test CA",
      &ca_public_path,
      &ca_private_path,
      3650,
      true,
    )?;

    // Now generate a node cert
    let node_public_path = temp.path().join("node.crt");
    let node_private_path = temp.path().join("node.key");

    generate_nebula_cert(
      &ca_public_path,
      &ca_private_path,
      "test-node",
      "10.1.1.5/24",
      &node_public_path,
      &node_private_path,
      true,
    )?;

    // Check that both files exist
    assert!(node_public_path.exists());
    assert!(node_private_path.exists());

    // Check public certificate content - should be a PEM certificate
    let public_content = std::fs::read_to_string(&node_public_path)?;
    assert!(public_content.contains("-----BEGIN NEBULA CERTIFICATE"));
    assert!(public_content.contains("-----END NEBULA CERTIFICATE"));

    // Check private key content - should be a PEM private key
    let private_content = std::fs::read_to_string(&node_private_path)?;
    assert!(private_content.contains("-----BEGIN NEBULA"));
    assert!(private_content.contains("-----END NEBULA"));

    // Check permissions - private should be 600
    let private_metadata = std::fs::metadata(&node_private_path)?;
    let private_perms = private_metadata.permissions();
    assert_eq!(private_perms.mode() & 0o777, 0o600);

    // Check permissions - public should be 644
    let public_metadata = std::fs::metadata(&node_public_path)?;
    let public_perms = public_metadata.permissions();
    assert_eq!(public_perms.mode() & 0o777, 0o644);

    Ok(())
  }

  #[test]
  fn test_generate_nebula_cert_different_ip() -> anyhow::Result<()> {
    let temp = TempDir::new()?;

    // First, create a CA
    let ca_public_path = temp.path().join("ca.crt");
    let ca_private_path = temp.path().join("ca.key");
    generate_nebula_ca(
      "Test CA",
      &ca_public_path,
      &ca_private_path,
      3650,
      true,
    )?;

    // Generate a node cert with different IP/CIDR
    let node_public_path = temp.path().join("node.crt");
    let node_private_path = temp.path().join("node.key");

    generate_nebula_cert(
      &ca_public_path,
      &ca_private_path,
      "test-node",
      "192.168.1.10/24",
      &node_public_path,
      &node_private_path,
      true,
    )?;

    // Check that both files exist
    assert!(node_public_path.exists());
    assert!(node_private_path.exists());

    // Check certificate was generated successfully
    let public_content = std::fs::read_to_string(&node_public_path)?;
    assert!(public_content.contains("-----BEGIN NEBULA CERTIFICATE"));

    Ok(())
  }

  #[test]
  fn test_generate_nebula_cert_no_renew() -> anyhow::Result<()> {
    let temp = TempDir::new()?;

    // First, create a CA
    let ca_public_path = temp.path().join("ca.crt");
    let ca_private_path = temp.path().join("ca.key");
    generate_nebula_ca(
      "Test CA",
      &ca_public_path,
      &ca_private_path,
      3650,
      true,
    )?;

    let node_public_path = temp.path().join("node.crt");
    let node_private_path = temp.path().join("node.key");

    // Pre-create files
    std::fs::write(&node_public_path, "existing_public")?;
    std::fs::write(&node_private_path, "existing_private")?;

    // Generate with renew=false should not overwrite
    generate_nebula_cert(
      &ca_public_path,
      &ca_private_path,
      "test-node",
      "10.1.1.5/24",
      &node_public_path,
      &node_private_path,
      false,
    )?;

    let public_content = std::fs::read_to_string(&node_public_path)?;
    let private_content = std::fs::read_to_string(&node_private_path)?;

    assert_eq!(public_content, "existing_public");
    assert_eq!(private_content, "existing_private");

    Ok(())
  }

  #[test]
  fn test_generate_nebula_cert_renew() -> anyhow::Result<()> {
    let temp = TempDir::new()?;

    // First, create a CA
    let ca_public_path = temp.path().join("ca.crt");
    let ca_private_path = temp.path().join("ca.key");
    generate_nebula_ca(
      "Test CA",
      &ca_public_path,
      &ca_private_path,
      3650,
      true,
    )?;

    let node_public_path = temp.path().join("node.crt");
    let node_private_path = temp.path().join("node.key");

    // Pre-create files
    std::fs::write(&node_public_path, "existing_public")?;
    std::fs::write(&node_private_path, "existing_private")?;

    // Generate with renew=true should overwrite
    generate_nebula_cert(
      &ca_public_path,
      &ca_private_path,
      "test-node",
      "10.1.1.5/24",
      &node_public_path,
      &node_private_path,
      true,
    )?;

    let public_content = std::fs::read_to_string(&node_public_path)?;
    let private_content = std::fs::read_to_string(&node_private_path)?;

    // Should contain Nebula certificate content now
    assert!(public_content.contains("-----BEGIN NEBULA CERTIFICATE"));
    assert!(private_content.contains("-----BEGIN NEBULA"));

    Ok(())
  }

  #[test]
  fn test_generate_nebula_cert_deterministic() -> anyhow::Result<()> {
    let temp = TempDir::new()?;

    // Create a CA
    let ca_public_path = temp.path().join("ca.crt");
    let ca_private_path = temp.path().join("ca.key");
    generate_nebula_ca(
      "Test CA",
      &ca_public_path,
      &ca_private_path,
      3650,
      true,
    )?;

    // Generate two different node certs
    let node1_public_path = temp.path().join("node1.crt");
    let node1_private_path = temp.path().join("node1.key");
    let node2_public_path = temp.path().join("node2.crt");
    let node2_private_path = temp.path().join("node2.key");

    generate_nebula_cert(
      &ca_public_path,
      &ca_private_path,
      "node1",
      "10.1.1.5/24",
      &node1_public_path,
      &node1_private_path,
      true,
    )?;

    generate_nebula_cert(
      &ca_public_path,
      &ca_private_path,
      "node2",
      "10.1.1.6/24",
      &node2_public_path,
      &node2_private_path,
      true,
    )?;

    let private1 = std::fs::read_to_string(&node1_private_path)?;
    let private2 = std::fs::read_to_string(&node2_private_path)?;
    let public1 = std::fs::read_to_string(&node1_public_path)?;
    let public2 = std::fs::read_to_string(&node2_public_path)?;

    // Keys should be different
    assert_ne!(private1, private2);
    assert_ne!(public1, public2);

    // Both should be valid Nebula certificates
    assert!(public1.contains("-----BEGIN NEBULA CERTIFICATE"));
    assert!(public2.contains("-----BEGIN NEBULA CERTIFICATE"));

    Ok(())
  }

  #[test]
  fn test_generate_nebula_cert_invalid_ca() {
    let temp = TempDir::new().unwrap();

    // Create invalid CA files
    let ca_public_path = temp.path().join("ca.crt");
    let ca_private_path = temp.path().join("ca.key");
    std::fs::write(&ca_public_path, "invalid ca cert").unwrap();
    std::fs::write(&ca_private_path, "invalid ca key").unwrap();

    let node_public_path = temp.path().join("node.crt");
    let node_private_path = temp.path().join("node.key");

    let result = generate_nebula_cert(
      &ca_public_path,
      &ca_private_path,
      "test-node",
      "10.1.1.5/24",
      &node_public_path,
      &node_private_path,
      true,
    );

    // Should fail because the CA is invalid
    assert!(result.is_err());
  }
}
