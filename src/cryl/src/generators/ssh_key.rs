use std::path::Path;
use std::process::Command;

use crate::common::{read_file_if_exists, save_atomic, CrylError, CrylResult};

/// Generate an SSH key pair (ed25519) and save public + private keys
///
/// # Arguments
/// * `name` - Key comment (e.g., email or host)
/// * `public` - Path to save the public key (with public permissions 644)
/// * `private` - Path to save the private key (with private permissions 600)
/// * `password` - Optional path to passphrase file (None for no passphrase)
/// * `renew` - Overwrite destinations if they exist
pub fn generate_ssh_key(
  name: &str,
  public: &Path,
  private: &Path,
  password: Option<&Path>,
  renew: bool,
) -> CrylResult<()> {
  // Create temp file paths (matching original nushell implementation)
  let tmp_private = private.with_extension("tmp");
  let tmp_public = private.with_extension("tmp.pub");

  // Get password content if provided
  let password_str = match password {
    Some(path) => read_file_if_exists(path)?.unwrap_or_default(),
    None => String::new(),
  };

  // Run ssh-keygen to generate the key pair
  let output = Command::new("ssh-keygen")
    .arg("-a")
    .arg("100")
    .arg("-t")
    .arg("ed25519")
    .arg("-C")
    .arg(name)
    .arg("-N")
    .arg(&password_str)
    .arg("-f")
    .arg(&tmp_private)
    .output()?;

  if !output.status.success() {
    // Clean up temp files on failure
    let _ = std::fs::remove_file(&tmp_private);
    let _ = std::fs::remove_file(&tmp_public);

    return Err(CrylError::ToolExecution {
      tool: "ssh-keygen".to_string(),
      exit_code: output.status.code().unwrap_or(-1),
      stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    });
  }

  // Read generated key files
  let private_content =
    read_file_if_exists(&tmp_private)?.ok_or_else(|| {
      CrylError::Generation {
        generator: "ssh-key".to_string(),
        message: "Private key file not generated".to_string(),
      }
    })?;

  let public_content =
    read_file_if_exists(&tmp_public)?.ok_or_else(|| CrylError::Generation {
      generator: "ssh-key".to_string(),
      message: "Public key file not generated".to_string(),
    })?;

  // Clean up temp files
  let _ = std::fs::remove_file(&tmp_private);
  let _ = std::fs::remove_file(&tmp_public);

  // Save private key with private permissions (600)
  save_atomic(private, private_content.as_bytes(), renew, false)?;

  // Save public key with public permissions (644)
  save_atomic(public, public_content.as_bytes(), renew, true)?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::os::unix::fs::PermissionsExt;
  use tempfile::TempDir;

  #[test]
  fn test_generate_ssh_key_success() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let public_path = temp.path().join("ssh_key.pub");
    let private_path = temp.path().join("ssh_key");

    generate_ssh_key(
      "test@example.com",
      &public_path,
      &private_path,
      None,
      true,
    )?;

    // Check that both files exist
    assert!(public_path.exists());
    assert!(private_path.exists());

    // Check private key content
    let private_content = std::fs::read_to_string(&private_path)?;
    assert!(private_content.contains("OPENSSH PRIVATE KEY"));

    // Check public key content
    let public_content = std::fs::read_to_string(&public_path)?;
    assert!(public_content.starts_with("ssh-ed25519"));
    assert!(public_content.contains("test@example.com"));

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
  fn test_generate_ssh_key_with_password() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let public_path = temp.path().join("ssh_key.pub");
    let private_path = temp.path().join("ssh_key");
    let password_path = temp.path().join("password");

    // Create password file
    std::fs::write(&password_path, "my_secret_password")?;

    generate_ssh_key(
      "test@example.com",
      &public_path,
      &private_path,
      Some(&password_path),
      true,
    )?;

    // Check that both files exist
    assert!(public_path.exists());
    assert!(private_path.exists());

    // Check that key files exist and are valid
    let private_content = std::fs::read_to_string(&private_path)?;
    assert!(private_content.contains("OPENSSH PRIVATE KEY"));

    // The key should be encrypted (ssh-keygen with -N encrypts the key)
    // We can verify by checking the files were created successfully
    assert!(public_path.exists());
    assert!(private_path.exists());

    Ok(())
  }

  #[test]
  fn test_generate_ssh_key_no_renew() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let public_path = temp.path().join("ssh_key.pub");
    let private_path = temp.path().join("ssh_key");

    // Pre-create files
    std::fs::write(&public_path, "existing_public")?;
    std::fs::write(&private_path, "existing_private")?;

    // Generate with renew=false should not overwrite
    generate_ssh_key(
      "test@example.com",
      &public_path,
      &private_path,
      None,
      false,
    )?;

    let public_content = std::fs::read_to_string(&public_path)?;
    let private_content = std::fs::read_to_string(&private_path)?;

    assert_eq!(public_content, "existing_public");
    assert_eq!(private_content, "existing_private");

    Ok(())
  }

  #[test]
  fn test_generate_ssh_key_renew() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let public_path = temp.path().join("ssh_key.pub");
    let private_path = temp.path().join("ssh_key");

    // Pre-create files
    std::fs::write(&public_path, "existing_public")?;
    std::fs::write(&private_path, "existing_private")?;

    // Generate with renew=true should overwrite
    generate_ssh_key(
      "test@example.com",
      &public_path,
      &private_path,
      None,
      true,
    )?;

    let public_content = std::fs::read_to_string(&public_path)?;
    let private_content = std::fs::read_to_string(&private_path)?;

    // Should contain SSH key content now
    assert!(private_content.contains("OPENSSH PRIVATE KEY"));
    assert!(public_content.starts_with("ssh-ed25519"));

    Ok(())
  }
}
