use std::fs;
use std::path::Path;
use std::process::Command;

use crate::common::{save_atomic, CrylError, CrylResult};

/// Split a key into Shamir shares and save them
///
/// # Arguments
/// * `key` - Path to the source key file (raw content is split)
/// * `prefix` - Filename prefix for each generated share
/// * `threshold` - Minimum number of shares required to reconstruct
/// * `shares` - Total number of shares to generate
/// * `renew` - Overwrite destinations if they exist
pub fn generate_key_split(
  key: &Path,
  prefix: &str,
  threshold: usize,
  shares: usize,
  renew: bool,
) -> CrylResult<()> {
  // Validate parameters
  if threshold == 0 {
    return Err(CrylError::Generation {
      generator: "key_split".to_string(),
      message: "threshold must be greater than 0".to_string(),
    });
  }

  if shares == 0 {
    return Err(CrylError::Generation {
      generator: "key_split".to_string(),
      message: "shares must be greater than 0".to_string(),
    });
  }

  if threshold > shares {
    return Err(CrylError::Generation {
      generator: "key_split".to_string(),
      message: "threshold cannot be greater than shares".to_string(),
    });
  }

  // Read the key content
  let key_content = fs::read_to_string(key)?;

  // Run ssss-split to generate shares
  let mut child = Command::new("ssss-split")
    .arg("-t")
    .arg(threshold.to_string())
    .arg("-n")
    .arg(shares.to_string())
    .arg("-q")
    .stdin(std::process::Stdio::piped())
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped())
    .spawn()?;

  // Write key content to stdin of the child process
  if let Some(ref mut stdin) = child.stdin {
    use std::io::Write;
    stdin.write_all(key_content.as_bytes())?;
    // stdin is dropped here when the scope ends, closing the pipe
  }

  let output = child.wait_with_output()?;

  if !output.status.success() {
    return Err(CrylError::ToolExecution {
      tool: "ssss-split".to_string(),
      exit_code: output.status.code().unwrap_or(-1),
      stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    });
  }

  // Parse output - each line is a share
  let output_str = String::from_utf8_lossy(&output.stdout);
  let share_lines: Vec<&str> = output_str
    .lines()
    .map(|line| line.trim())
    .filter(|line| !line.is_empty())
    .collect();

  // Save each share to a file
  for (index, share_content) in share_lines.iter().enumerate() {
    let share_name = format!("{}-{}", prefix, index);
    let share_path = Path::new(&share_name);
    save_atomic(share_path, share_content.as_bytes(), renew, false)?;
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::os::unix::fs::PermissionsExt;
  use tempfile::TempDir;

  #[test]
  fn test_generate_key_split_success() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let key_path = temp.path().join("test_key");
    fs::write(&key_path, "my_secret_key_12345")?;

    let prefix = temp.path().join("share").to_string_lossy().to_string();
    generate_key_split(&key_path, &prefix, 2, 3, true)?;

    // Check that 3 share files were created
    for i in 0..3 {
      let share_path = temp.path().join(format!("share-{}", i));
      assert!(share_path.exists(), "Share {} should exist", i);

      // Check permissions - should be private (600)
      let metadata = fs::metadata(&share_path)?;
      let perms = metadata.permissions();
      assert_eq!(
        perms.mode() & 0o777,
        0o600,
        "Share {} should have 600 permissions",
        i
      );
    }

    Ok(())
  }

  #[test]
  fn test_generate_key_split_threshold_equals_shares() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let key_path = temp.path().join("test_key");
    fs::write(&key_path, "my_secret_key_12345")?;

    let prefix = temp.path().join("share").to_string_lossy().to_string();
    // Test when threshold equals shares (all shares needed to reconstruct)
    generate_key_split(&key_path, &prefix, 3, 3, true)?;

    // Check that 3 share files were created
    for i in 0..3 {
      let share_path = temp.path().join(format!("share-{}", i));
      assert!(share_path.exists(), "Share {} should exist", i);
    }

    Ok(())
  }

  #[test]
  fn test_generate_key_split_no_renew() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let key_path = temp.path().join("test_key");
    fs::write(&key_path, "my_secret_key_12345")?;

    let prefix = temp.path().join("share").to_string_lossy().to_string();

    // Pre-create share files
    fs::write(temp.path().join("share-0"), "existing_share_0")?;
    fs::write(temp.path().join("share-1"), "existing_share_1")?;
    fs::write(temp.path().join("share-2"), "existing_share_2")?;

    // Generate with renew=false should not overwrite
    generate_key_split(&key_path, &prefix, 2, 3, false)?;

    let content0 = fs::read_to_string(temp.path().join("share-0"))?;
    let content1 = fs::read_to_string(temp.path().join("share-1"))?;
    let content2 = fs::read_to_string(temp.path().join("share-2"))?;

    assert_eq!(content0, "existing_share_0");
    assert_eq!(content1, "existing_share_1");
    assert_eq!(content2, "existing_share_2");

    Ok(())
  }

  #[test]
  fn test_generate_key_split_renew_overwrites() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let key_path = temp.path().join("test_key");
    fs::write(&key_path, "my_secret_key_12345")?;

    let prefix = temp.path().join("share").to_string_lossy().to_string();

    // Pre-create share files
    fs::write(temp.path().join("share-0"), "existing_share_0")?;
    fs::write(temp.path().join("share-1"), "existing_share_1")?;
    fs::write(temp.path().join("share-2"), "existing_share_2")?;

    // Generate with renew=true should overwrite
    generate_key_split(&key_path, &prefix, 2, 3, true)?;

    // Content should be different now (ssss shares)
    let content0 = fs::read_to_string(temp.path().join("share-0"))?;
    assert_ne!(content0, "existing_share_0");

    Ok(())
  }

  #[test]
  fn test_generate_key_split_zero_threshold() {
    let temp = TempDir::new().unwrap();
    let key_path = temp.path().join("test_key");
    fs::write(&key_path, "my_secret_key_12345").unwrap();

    let prefix = temp.path().join("share").to_string_lossy().to_string();
    let result = generate_key_split(&key_path, &prefix, 0, 3, true);

    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("threshold must be greater than 0"));
  }

  #[test]
  fn test_generate_key_split_zero_shares() {
    let temp = TempDir::new().unwrap();
    let key_path = temp.path().join("test_key");
    fs::write(&key_path, "my_secret_key_12345").unwrap();

    let prefix = temp.path().join("share").to_string_lossy().to_string();
    let result = generate_key_split(&key_path, &prefix, 2, 0, true);

    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("shares must be greater than 0"));
  }

  #[test]
  fn test_generate_key_split_threshold_greater_than_shares() {
    let temp = TempDir::new().unwrap();
    let key_path = temp.path().join("test_key");
    fs::write(&key_path, "my_secret_key_12345").unwrap();

    let prefix = temp.path().join("share").to_string_lossy().to_string();
    let result = generate_key_split(&key_path, &prefix, 5, 3, true);

    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("threshold cannot be greater than shares"));
  }
}
