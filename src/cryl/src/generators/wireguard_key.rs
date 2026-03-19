use std::path::Path;
use std::process::Command;

use crate::common::{CrylError, CrylResult, save_atomic};

/// Generate a WireGuard key pair and save public + private keys
///
/// # Arguments
/// * `private` - Path to save the private key (with private permissions 600)
/// * `public` - Path to save the public key (with public permissions 644)
/// * `renew` - Overwrite destinations if they exist
pub fn generate_wireguard_key(
  private: &Path,
  public: &Path,
  renew: bool,
) -> CrylResult<()> {
  // Generate private key using wg genkey
  let private_output = Command::new("wg").arg("genkey").output()?;

  if !private_output.status.success() {
    return Err(CrylError::ToolExecution {
      tool: "wg genkey".to_string(),
      exit_code: private_output.status.code().unwrap_or(-1),
      stderr: String::from_utf8_lossy(&private_output.stderr).to_string(),
    });
  }

  let private_content = String::from_utf8_lossy(&private_output.stdout);
  let private_key = private_content.trim();

  // Generate public key by piping private key to wg pubkey
  let mut public_child = Command::new("wg")
    .arg("pubkey")
    .stdin(std::process::Stdio::piped())
    .stdout(std::process::Stdio::piped())
    .spawn()?;

  // Write private key to stdin of the child process
  if let Some(ref mut stdin) = public_child.stdin {
    use std::io::Write;
    stdin.write_all(private_key.as_bytes())?;
    // stdin is dropped here when the scope ends, closing the pipe
  }

  let public_result = public_child.wait_with_output()?;

  if !public_result.status.success() {
    return Err(CrylError::ToolExecution {
      tool: "wg pubkey".to_string(),
      exit_code: public_result.status.code().unwrap_or(-1),
      stderr: String::from_utf8_lossy(&public_result.stderr).to_string(),
    });
  }

  let public_content = String::from_utf8_lossy(&public_result.stdout);
  let public_key = public_content.trim();

  // Save private key with private permissions (600)
  save_atomic(private, private_key.as_bytes(), renew, false)?;

  // Save public key with public permissions (644)
  save_atomic(public, public_key.as_bytes(), renew, true)?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::os::unix::fs::PermissionsExt;
  use tempfile::TempDir;

  #[test]
  fn test_generate_wireguard_key_success() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let public_path = temp.path().join("wg_public.key");
    let private_path = temp.path().join("wg_private.key");

    generate_wireguard_key(&private_path, &public_path, true)?;

    // Check that both files exist
    assert!(public_path.exists());
    assert!(private_path.exists());

    // Check private key content - should be 44 chars base64 (trimmed)
    let private_content = std::fs::read_to_string(&private_path)?;
    let private_trimmed = private_content.trim();
    assert_eq!(private_trimmed.len(), 44);
    // WireGuard keys are base64 encoded, so should only contain valid chars
    assert!(private_trimmed.chars().all(|c| {
      c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='
    }));

    // Check public key content - should be 44 chars base64 (trimmed)
    let public_content = std::fs::read_to_string(&public_path)?;
    let public_trimmed = public_content.trim();
    assert_eq!(public_trimmed.len(), 44);
    // WireGuard public keys are also base64 encoded
    assert!(public_trimmed.chars().all(|c| {
      c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='
    }));

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
  fn test_generate_wireguard_key_no_renew() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let public_path = temp.path().join("wg_public.key");
    let private_path = temp.path().join("wg_private.key");

    // Pre-create files
    std::fs::write(&public_path, "existing_public")?;
    std::fs::write(&private_path, "existing_private")?;

    // Generate with renew=false should not overwrite
    generate_wireguard_key(&private_path, &public_path, false)?;

    let public_content = std::fs::read_to_string(&public_path)?;
    let private_content = std::fs::read_to_string(&private_path)?;

    assert_eq!(public_content, "existing_public");
    assert_eq!(private_content, "existing_private");

    Ok(())
  }

  #[test]
  fn test_generate_wireguard_key_renew() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let public_path = temp.path().join("wg_public.key");
    let private_path = temp.path().join("wg_private.key");

    // Pre-create files
    std::fs::write(&public_path, "existing_public")?;
    std::fs::write(&private_path, "existing_private")?;

    // Generate with renew=true should overwrite
    generate_wireguard_key(&private_path, &public_path, true)?;

    let public_content = std::fs::read_to_string(&public_path)?;
    let private_content = std::fs::read_to_string(&private_path)?;

    // Should contain WireGuard key content now (base64, 44 chars trimmed)
    assert_eq!(private_content.trim().len(), 44);
    assert_eq!(public_content.trim().len(), 44);

    Ok(())
  }

  #[test]
  fn test_generate_wireguard_key_deterministic() -> anyhow::Result<()> {
    // WireGuard keys should be different on each generation
    let temp1 = TempDir::new()?;
    let temp2 = TempDir::new()?;

    let public_path1 = temp1.path().join("wg_public.key");
    let private_path1 = temp1.path().join("wg_private.key");
    let public_path2 = temp2.path().join("wg_public.key");
    let private_path2 = temp2.path().join("wg_private.key");

    generate_wireguard_key(&private_path1, &public_path1, true)?;
    generate_wireguard_key(&private_path2, &public_path2, true)?;

    let private1 = std::fs::read_to_string(&private_path1)?;
    let private2 = std::fs::read_to_string(&private_path2)?;
    let public1 = std::fs::read_to_string(&public_path1)?;
    let public2 = std::fs::read_to_string(&public_path2)?;

    // Keys should be different
    assert_ne!(private1, private2);
    assert_ne!(public1, public2);

    Ok(())
  }
}
