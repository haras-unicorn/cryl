use std::{path::Path, process::Command};

use crate::common::{CrylError, CrylResult, save_atomic};

/// Generate a CephFS key
///
/// # Arguments
/// * `name` - Path to save the key
/// * `renew` - Overwrite destination if it exists
pub fn generate_ceph_key(name: &Path, renew: bool) -> CrylResult<()> {
  // Run ceph-authtool
  let output = Command::new("ceph-authtool")
    .arg("--gen-print-key")
    .output()?;

  // Check command success
  if !output.status.success() {
    return Err(CrylError::ToolExecution {
      tool: "ceph-authtool".to_string(),
      exit_code: output.status.code().unwrap_or(-1),
      stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    });
  }

  // Read key contents
  let content = String::from_utf8_lossy(&output.stdout);

  // Save key with private permissions (600)
  save_atomic(name, content.as_bytes(), renew, false)?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;
  use tempfile::TempDir;

  #[test]
  fn test_generate_ceph_key_success() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("test_key");

    generate_ceph_key(&path, false).unwrap();

    assert!(path.exists());
  }

  #[test]
  fn test_generate_ceph_key_renew_false_no_overwrite() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("test_key");

    fs::write(&path, "original_key").unwrap();
    generate_ceph_key(&path, false).unwrap();

    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, "original_key");
  }

  #[test]
  fn test_generate_ceph_key_renew_true_overwrites() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("test_key");

    fs::write(&path, "original_key").unwrap();
    generate_ceph_key(&path, true).unwrap();

    let content = fs::read_to_string(&path).unwrap();
    assert_ne!(content, "original_key");
  }

  #[test]
  fn test_generate_ceph_key_private_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let temp = TempDir::new().unwrap();
    let path = temp.path().join("test_key");

    generate_ceph_key(&path, false).unwrap();

    let metadata = fs::metadata(&path).unwrap();
    let perms = metadata.permissions();
    assert_eq!(perms.mode() & 0o777, 0o600);
  }

  #[test]
  fn test_generate_ceph_key_subdir() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("subdir").join("test_key");

    generate_ceph_key(&path, false).unwrap();

    assert!(path.exists());
  }
}
