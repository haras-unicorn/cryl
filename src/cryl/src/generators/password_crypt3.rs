use std::path::Path;
use std::process::Stdio;

use crate::common::{CrylError, CrylResult, generate_random_alphanumeric};

/// Generate random password with yescrypt hashing (crypt(3) format)
///
/// Generates a random alphanumeric password and its yescrypt hash.
/// The plaintext password is saved to the private path, and the
/// yescrypt-encoded hash (crypt(3) format) is saved to the public path.
///
/// Uses mkpasswd with --method=yescrypt to generate the hash.
pub fn generate_password_crypt3(
  public: &Path,
  private: &Path,
  length: usize,
  renew: bool,
) -> CrylResult<()> {
  // Generate random alphanumeric password
  let password = generate_random_alphanumeric(length)?;

  // Run mkpasswd with the password piped to stdin
  // mkpasswd --stdin --method=yescrypt
  let mut mkpasswd_child = std::process::Command::new("mkpasswd")
    .args(["--stdin", "--method=yescrypt"])
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()?;

  // Write password to mkpasswd stdin
  if let Some(mut stdin) = mkpasswd_child.stdin.take() {
    std::io::Write::write_all(&mut stdin, password.as_bytes())?;
    // stdin is closed when it goes out of scope
  }

  let mkpasswd_output = mkpasswd_child.wait_with_output()?;

  if !mkpasswd_output.status.success() {
    return Err(CrylError::ToolExecution {
      tool: "mkpasswd".to_string(),
      exit_code: mkpasswd_output.status.code().unwrap_or(-1),
      stderr: String::from_utf8_lossy(&mkpasswd_output.stderr).to_string(),
    });
  }

  let hash = String::from_utf8_lossy(&mkpasswd_output.stdout);
  let hash = hash.trim();

  // Save plaintext password to private path (private permissions)
  crate::common::save_atomic(private, password.as_bytes(), renew, false)?;

  // Save hash to public path (public permissions)
  crate::common::save_atomic(public, hash.as_bytes(), renew, true)?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use tempfile::TempDir;

  use super::*;

  #[test]
  fn test_generate_password_crypt3() -> anyhow::Result<()> {
    let temp = TempDir::new().unwrap();
    let public_path = temp.path().join("public");
    let private_path = temp.path().join("private");

    generate_password_crypt3(&public_path, &private_path, 16, true)?;

    // Check private file contains plaintext password
    let password = std::fs::read_to_string(&private_path)?;
    assert_eq!(password.len(), 16);
    assert!(password.chars().all(|c| c.is_ascii_alphanumeric()));

    // Check public file contains yescrypt hash
    let hash = std::fs::read_to_string(&public_path)?;
    // yescrypt hashes start with $y$
    assert!(hash.starts_with("$y$"));

    Ok(())
  }

  #[test]
  fn test_generate_password_crypt3_length() -> anyhow::Result<()> {
    let temp = TempDir::new().unwrap();

    // Test different lengths
    for length in [8, 12, 24, 32] {
      let public_path = temp.path().join(format!("public_{}", length));
      let private_path = temp.path().join(format!("private_{}", length));

      generate_password_crypt3(&public_path, &private_path, length, true)?;

      let password = std::fs::read_to_string(&private_path)?;
      assert_eq!(
        password.len(),
        length,
        "Password length mismatch for length={}",
        length
      );
    }

    Ok(())
  }

  #[test]
  fn test_generate_password_crypt3_no_overwrite_without_renew()
  -> anyhow::Result<()> {
    let temp = TempDir::new().unwrap();
    let public_path = temp.path().join("public");
    let private_path = temp.path().join("private");

    std::fs::write(&public_path, "existing_public")?;
    std::fs::write(&private_path, "existing_private")?;

    generate_password_crypt3(&public_path, &private_path, 16, false)?;

    // Files should not be overwritten
    assert_eq!(std::fs::read_to_string(&public_path)?, "existing_public");
    assert_eq!(std::fs::read_to_string(&private_path)?, "existing_private");

    Ok(())
  }

  #[test]
  fn test_generate_password_crypt3_permissions() -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let temp = TempDir::new().unwrap();
    let public_path = temp.path().join("public");
    let private_path = temp.path().join("private");

    generate_password_crypt3(&public_path, &private_path, 16, true)?;

    // Check private file has 600 permissions
    let private_meta = std::fs::metadata(&private_path)?;
    assert_eq!(private_meta.permissions().mode() & 0o777, 0o600);

    // Check public file has 644 permissions
    let public_meta = std::fs::metadata(&public_path)?;
    assert_eq!(public_meta.permissions().mode() & 0o777, 0o644);

    Ok(())
  }
}
