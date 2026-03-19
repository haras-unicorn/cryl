use std::path::Path;
use std::process::Command;

use crate::common::{CrylError, CrylResult, save_atomic};

/// Generate OpenSSL Diffie-Hellman parameters
///
/// # Arguments
/// * `name` - Path to save the DH parameters file
/// * `renew` - Overwrite destination if it exists
pub fn generate_tls_dhparam(name: &Path, renew: bool) -> CrylResult<()> {
  // Check if we should skip (file exists and no renew)
  if !renew && name.exists() {
    return Ok(());
  }

  let output = Command::new("openssl")
    .arg("dhparam")
    .arg("-quiet")
    .arg("2048")
    .output()?;

  if !output.status.success() {
    return Err(CrylError::ToolExecution {
      tool: "openssl dhparam".to_string(),
      exit_code: output.status.code().unwrap_or(-1),
      stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    });
  }

  let dhparam_content = String::from_utf8_lossy(&output.stdout);
  save_atomic(name, dhparam_content.as_bytes(), renew, false)?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;
  use tempfile::TempDir;

  #[test]
  fn test_generate_tls_dhparam_creates_file() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("dhparam.pem");

    generate_tls_dhparam(&path, false).unwrap();

    assert!(path.exists());
    let content = fs::read_to_string(&path).unwrap();
    // DH params should contain "BEGIN DH PARAMETERS" and "END DH PARAMETERS"
    assert!(content.contains("BEGIN DH PARAMETERS"));
    assert!(content.contains("END DH PARAMETERS"));
  }

  #[test]
  fn test_generate_tls_dhparam_renew_false_no_overwrite() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("dhparam.pem");

    fs::write(&path, "original content").unwrap();
    generate_tls_dhparam(&path, false).unwrap();

    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, "original content");
  }

  #[test]
  fn test_generate_tls_dhparam_renew_true_overwrites() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("dhparam.pem");

    fs::write(&path, "original content").unwrap();
    generate_tls_dhparam(&path, true).unwrap();

    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("BEGIN DH PARAMETERS"));
  }

  #[test]
  fn test_generate_tls_dhparam_private_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let temp = TempDir::new().unwrap();
    let path = temp.path().join("dhparam.pem");

    generate_tls_dhparam(&path, false).unwrap();

    let metadata = fs::metadata(&path).unwrap();
    let perms = metadata.permissions();
    assert_eq!(perms.mode() & 0o777, 0o600);
  }
}
