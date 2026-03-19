use std::path::Path;
use std::process::Stdio;

use crate::common::{generate_random_alphanumeric, CrylError, CrylResult};

/// Generate random password with argon2 hashing
///
/// Generates a random alphanumeric password and its argon2id hash.
/// The plaintext password is saved to the private path, and the
/// argon2-encoded hash is saved to the public path.
///
/// Uses argon2 parameters: -id (argon2id variant), -k 19456 (19MB memory),
/// -t 2 (2 iterations), -p 1 (1 parallel thread)
pub fn generate_password(
  public: &Path,
  private: &Path,
  length: usize,
  renew: bool,
) -> CrylResult<()> {
  // Generate random alphanumeric password
  let password = generate_random_alphanumeric(length)?;

  // Generate salt using openssl
  let salt_output = std::process::Command::new("openssl")
    .args(["rand", "-base64", "32"])
    .output()?;

  if !salt_output.status.success() {
    return Err(CrylError::ToolExecution {
      tool: "openssl".to_string(),
      exit_code: salt_output.status.code().unwrap_or(-1),
      stderr: String::from_utf8_lossy(&salt_output.stderr).to_string(),
    });
  }

  let salt = String::from_utf8_lossy(&salt_output.stdout);
  let salt = salt.trim();

  // Run argon2 with the password piped to stdin
  // argon2 $salt -e -id -k 19456 -t 2 -p 1
  let mut argon2_child = std::process::Command::new("argon2")
    .arg(salt)
    .args(["-e", "-id", "-k", "19456", "-t", "2", "-p", "1"])
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()?;

  // Write password to argon2 stdin
  if let Some(mut stdin) = argon2_child.stdin.take() {
    std::io::Write::write_all(&mut stdin, password.as_bytes())?;
    // stdin is closed when it goes out of scope
  }

  let argon2_output = argon2_child.wait_with_output()?;

  if !argon2_output.status.success() {
    return Err(CrylError::ToolExecution {
      tool: "argon2".to_string(),
      exit_code: argon2_output.status.code().unwrap_or(-1),
      stderr: String::from_utf8_lossy(&argon2_output.stderr).to_string(),
    });
  }

  let hash = String::from_utf8_lossy(&argon2_output.stdout);
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
  fn test_generate_password_argon2() -> anyhow::Result<()> {
    let temp = TempDir::new().unwrap();
    let public_path = temp.path().join("public");
    let private_path = temp.path().join("private");

    generate_password(&public_path, &private_path, 16, true)?;

    // Check private file contains plaintext password
    let password = std::fs::read_to_string(&private_path)?;
    assert_eq!(password.len(), 16);
    assert!(password.chars().all(|c| c.is_ascii_alphanumeric()));

    // Check public file contains argon2 hash
    let hash = std::fs::read_to_string(&public_path)?;
    // Argon2 hashes start with $argon2
    assert!(hash.starts_with("$argon2"));
    // Should contain the id variant marker
    assert!(hash.contains("$argon2id$"));

    Ok(())
  }

  #[test]
  fn test_generate_password_argon2_length() -> anyhow::Result<()> {
    let temp = TempDir::new().unwrap();
    let public_path = temp.path().join("public");
    let private_path = temp.path().join("private");

    // Test different lengths
    for length in [8, 12, 24, 32] {
      let public_path = temp.path().join(format!("public_{}", length));
      let private_path = temp.path().join(format!("private_{}", length));

      generate_password(&public_path, &private_path, length, true)?;

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
  fn test_generate_password_argon2_no_overwrite_without_renew(
  ) -> anyhow::Result<()> {
    let temp = TempDir::new().unwrap();
    let public_path = temp.path().join("public");
    let private_path = temp.path().join("private");

    std::fs::write(&public_path, "existing_public")?;
    std::fs::write(&private_path, "existing_private")?;

    generate_password(&public_path, &private_path, 16, false)?;

    // Files should not be overwritten
    assert_eq!(std::fs::read_to_string(&public_path)?, "existing_public");
    assert_eq!(std::fs::read_to_string(&private_path)?, "existing_private");

    Ok(())
  }

  #[test]
  fn test_generate_password_argon2_permissions() -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let temp = TempDir::new().unwrap();
    let public_path = temp.path().join("public");
    let private_path = temp.path().join("private");

    generate_password(&public_path, &private_path, 16, true)?;

    // Check private file has 600 permissions
    let private_meta = std::fs::metadata(&private_path)?;
    assert_eq!(private_meta.permissions().mode() & 0o777, 0o600);

    // Check public file has 644 permissions
    let public_meta = std::fs::metadata(&public_path)?;
    assert_eq!(public_meta.permissions().mode() & 0o777, 0o644);

    Ok(())
  }
}
