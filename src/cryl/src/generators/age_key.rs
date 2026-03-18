use std::path::Path;
use std::process::Command;

use crate::common::{save_atomic, CrylError, CrylResult};

/// Generate an age key pair and save public + private keys
///
/// # Arguments
/// * `public` - Path to save the public key (with public permissions 644)
/// * `private` - Path to save the private key (with private permissions 600)
/// * `renew` - Overwrite destinations if they exist
pub fn generate_age_key(
  public: &Path,
  private: &Path,
  renew: bool,
) -> CrylResult<()> {
  // Generate private key using age-keygen
  let private_output = Command::new("age-keygen").output()?;

  if !private_output.status.success() {
    return Err(CrylError::ToolExecution {
      tool: "age-keygen".to_string(),
      exit_code: private_output.status.code().unwrap_or(-1),
      stderr: String::from_utf8_lossy(&private_output.stderr).to_string(),
    });
  }

  let private_content = String::from_utf8_lossy(&private_output.stdout);

  // Generate public key by piping private key to age-keygen -y
  let mut public_child = Command::new("age-keygen")
    .arg("-y")
    .stdin(std::process::Stdio::piped())
    .stdout(std::process::Stdio::piped())
    .spawn()?;

  // Write private key to stdin of the child process
  if let Some(ref mut stdin) = public_child.stdin {
    use std::io::Write;
    stdin.write_all(private_content.as_bytes())?;
    // stdin is dropped here when the scope ends, closing the pipe
  }

  let public_result = public_child.wait_with_output()?;

  if !public_result.status.success() {
    return Err(CrylError::ToolExecution {
      tool: "age-keygen -y".to_string(),
      exit_code: public_result.status.code().unwrap_or(-1),
      stderr: String::from_utf8_lossy(&public_result.stderr).to_string(),
    });
  }

  let public_content = String::from_utf8_lossy(&public_result.stdout);

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
  fn test_generate_age_key_success() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let public_path = temp.path().join("age_public.key");
    let private_path = temp.path().join("age_private.key");

    generate_age_key(&public_path, &private_path, true)?;

    // Check that both files exist
    assert!(public_path.exists());
    assert!(private_path.exists());

    // Check private key content
    let private_content = std::fs::read_to_string(&private_path)?;
    assert!(private_content.contains("AGE-SECRET-KEY-"));

    // Check public key content
    let public_content = std::fs::read_to_string(&public_path)?;
    assert!(public_content.starts_with("age1"));

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
  fn test_generate_age_key_no_renew() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let public_path = temp.path().join("age_public.key");
    let private_path = temp.path().join("age_private.key");

    // Pre-create files
    std::fs::write(&public_path, "existing_public")?;
    std::fs::write(&private_path, "existing_private")?;

    // Generate with renew=false should not overwrite
    generate_age_key(&public_path, &private_path, false)?;

    let public_content = std::fs::read_to_string(&public_path)?;
    let private_content = std::fs::read_to_string(&private_path)?;

    assert_eq!(public_content, "existing_public");
    assert_eq!(private_content, "existing_private");

    Ok(())
  }

  #[test]
  fn test_generate_age_key_renew() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let public_path = temp.path().join("age_public.key");
    let private_path = temp.path().join("age_private.key");

    // Pre-create files
    std::fs::write(&public_path, "existing_public")?;
    std::fs::write(&private_path, "existing_private")?;

    // Generate with renew=true should overwrite
    generate_age_key(&public_path, &private_path, true)?;

    let public_content = std::fs::read_to_string(&public_path)?;
    let private_content = std::fs::read_to_string(&private_path)?;

    // Should contain age key content now
    assert!(private_content.contains("AGE-SECRET-KEY-"));
    assert!(public_content.starts_with("age1"));

    Ok(())
  }
}
