use std::fs;
use std::path::Path;
use std::process::Command;

use crate::common::{save_atomic, CrylError, CrylResult};

/// Combine Shamir shares back into a single key
///
/// # Arguments
/// * `shares` - Comma-separated list of share file paths (e.g., "share-0,share-1,share-2")
/// * `key` - Path to save the reconstructed key
/// * `threshold` - Number of shares required to reconstruct (must match split threshold)
/// * `renew` - Overwrite destination if it exists
pub fn generate_key_combine(
  shares: &str,
  key: &Path,
  threshold: usize,
  renew: bool,
) -> CrylResult<()> {
  // Validate parameters
  if threshold == 0 {
    return Err(CrylError::Generation {
      generator: "key_combine".to_string(),
      message: "threshold must be greater than 0".to_string(),
    });
  }

  // Parse the comma-separated list of share paths
  let share_paths: Vec<&str> = shares
    .split(',')
    .map(|s| s.trim())
    .filter(|s| !s.is_empty())
    .collect();

  if share_paths.is_empty() {
    return Err(CrylError::Generation {
      generator: "key_combine".to_string(),
      message: "no share files provided".to_string(),
    });
  }

  if share_paths.len() < threshold {
    return Err(CrylError::Generation {
      generator: "key_combine".to_string(),
      message: format!(
        "insufficient shares provided: got {}, need at least {}",
        share_paths.len(),
        threshold
      ),
    });
  }

  // Read each share file and join with newlines
  let mut combined_shares = String::new();
  for (index, share_path) in share_paths.iter().enumerate() {
    if share_path.is_empty() {
      continue;
    }

    let share_content =
      fs::read_to_string(share_path).map_err(|e| CrylError::Generation {
        generator: "key_combine".to_string(),
        message: format!("failed to read share file '{}': {}", share_path, e),
      })?;

    if index > 0 {
      combined_shares.push('\n');
    }
    combined_shares.push_str(share_content.trim());
  }

  // Ensure the combined shares end with a newline (as per nushell impl)
  combined_shares.push('\n');

  // Run ssss-combine to reconstruct the key
  let mut child = Command::new("ssss-combine")
    .arg("-t")
    .arg(threshold.to_string())
    .arg("-q")
    .stdin(std::process::Stdio::piped())
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped())
    .spawn()?;

  // Write combined shares to stdin of the child process
  if let Some(ref mut stdin) = child.stdin {
    use std::io::Write;
    stdin.write_all(combined_shares.as_bytes())?;
  }

  let output = child.wait_with_output()?;

  if !output.status.success() {
    return Err(CrylError::ToolExecution {
      tool: "ssss-combine".to_string(),
      exit_code: output.status.code().unwrap_or(-1),
      stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    });
  }

  // Parse output - the reconstructed key
  let reconstructed_key = String::from_utf8_lossy(&output.stdout);
  let trimmed_key = reconstructed_key.trim();

  // Save the reconstructed key
  save_atomic(key, trimmed_key.as_bytes(), renew, false)?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::os::unix::fs::PermissionsExt;
  use tempfile::TempDir;

  #[test]
  fn test_generate_key_combine_success() -> anyhow::Result<()> {
    let temp = TempDir::new()?;

    // First split a key
    let original_key = "my_super_secret_key_12345";
    let key_path = temp.path().join("original_key");
    fs::write(&key_path, original_key)?;

    let prefix = temp.path().join("share").to_string_lossy().to_string();
    crate::generators::generate_key_split(&key_path, &prefix, 2, 3, true)?;

    // Now combine 2 of the 3 shares to reconstruct
    let share_paths = format!(
      "{},{}",
      temp.path().join("share-0").display(),
      temp.path().join("share-1").display()
    );

    let reconstructed_path = temp.path().join("reconstructed_key");
    generate_key_combine(&share_paths, &reconstructed_path, 2, true)?;

    // Verify the reconstructed key matches the original
    let reconstructed = fs::read_to_string(&reconstructed_path)?;
    assert_eq!(reconstructed, original_key);

    // Check permissions - should be private (600)
    let metadata = fs::metadata(&reconstructed_path)?;
    let perms = metadata.permissions();
    assert_eq!(perms.mode() & 0o777, 0o600);

    Ok(())
  }

  #[test]
  fn test_generate_key_combine_with_different_shares() -> anyhow::Result<()> {
    let temp = TempDir::new()?;

    // First split a key
    let original_key = "another_secret_key_67890";
    let key_path = temp.path().join("original_key");
    fs::write(&key_path, original_key)?;

    let prefix = temp.path().join("share").to_string_lossy().to_string();
    crate::generators::generate_key_split(&key_path, &prefix, 2, 3, true)?;

    // Combine using shares 1 and 2 instead of 0 and 1
    let share_paths = format!(
      "{},{}",
      temp.path().join("share-1").display(),
      temp.path().join("share-2").display()
    );

    let reconstructed_path = temp.path().join("reconstructed_key");
    generate_key_combine(&share_paths, &reconstructed_path, 2, true)?;

    // Verify the reconstructed key matches the original
    let reconstructed = fs::read_to_string(&reconstructed_path)?;
    assert_eq!(reconstructed, original_key);

    Ok(())
  }

  #[test]
  fn test_generate_key_combine_all_shares() -> anyhow::Result<()> {
    let temp = TempDir::new()?;

    // First split a key
    let original_key = "yet_another_secret_key";
    let key_path = temp.path().join("original_key");
    fs::write(&key_path, original_key)?;

    let prefix = temp.path().join("share").to_string_lossy().to_string();
    crate::generators::generate_key_split(&key_path, &prefix, 3, 3, true)?;

    // Combine all 3 shares
    let share_paths = format!(
      "{},{},{}",
      temp.path().join("share-0").display(),
      temp.path().join("share-1").display(),
      temp.path().join("share-2").display()
    );

    let reconstructed_path = temp.path().join("reconstructed_key");
    generate_key_combine(&share_paths, &reconstructed_path, 3, true)?;

    // Verify the reconstructed key matches the original
    let reconstructed = fs::read_to_string(&reconstructed_path)?;
    assert_eq!(reconstructed, original_key);

    Ok(())
  }

  #[test]
  fn test_generate_key_combine_no_renew() -> anyhow::Result<()> {
    let temp = TempDir::new()?;

    // Create existing file
    let reconstructed_path = temp.path().join("reconstructed_key");
    fs::write(&reconstructed_path, "existing_content")?;

    // First split a key
    let original_key = "test_key_no_renew";
    let key_path = temp.path().join("original_key");
    fs::write(&key_path, original_key)?;

    let prefix = temp.path().join("share").to_string_lossy().to_string();
    crate::generators::generate_key_split(&key_path, &prefix, 2, 3, true)?;

    // Try to combine with renew=false
    let share_paths = format!(
      "{},{}",
      temp.path().join("share-0").display(),
      temp.path().join("share-1").display()
    );

    generate_key_combine(&share_paths, &reconstructed_path, 2, false)?;

    // Content should not have changed
    let content = fs::read_to_string(&reconstructed_path)?;
    assert_eq!(content, "existing_content");

    Ok(())
  }

  #[test]
  fn test_generate_key_combine_renew_overwrites() -> anyhow::Result<()> {
    let temp = TempDir::new()?;

    // Create existing file
    let reconstructed_path = temp.path().join("reconstructed_key");
    fs::write(&reconstructed_path, "existing_content")?;

    // First split a key
    let original_key = "test_key_overwrite";
    let key_path = temp.path().join("original_key");
    fs::write(&key_path, original_key)?;

    let prefix = temp.path().join("share").to_string_lossy().to_string();
    crate::generators::generate_key_split(&key_path, &prefix, 2, 3, true)?;

    // Try to combine with renew=true
    let share_paths = format!(
      "{},{}",
      temp.path().join("share-0").display(),
      temp.path().join("share-1").display()
    );

    generate_key_combine(&share_paths, &reconstructed_path, 2, true)?;

    // Content should be the reconstructed key
    let content = fs::read_to_string(&reconstructed_path)?;
    assert_eq!(content, original_key);

    Ok(())
  }

  #[test]
  fn test_generate_key_combine_zero_threshold() {
    let temp = TempDir::new().unwrap();
    let share_path = temp.path().join("share-0");
    fs::write(&share_path, "dummy_share").unwrap();

    let key_path = temp.path().join("reconstructed_key");
    let share_paths = share_path.display().to_string();
    let result = generate_key_combine(&share_paths, &key_path, 0, true);

    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("threshold must be greater than 0"));
  }

  #[test]
  fn test_generate_key_combine_empty_shares() {
    let temp = TempDir::new().unwrap();
    let key_path = temp.path().join("reconstructed_key");

    let result = generate_key_combine("", &key_path, 2, true);

    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("no share files provided"));
  }

  #[test]
  fn test_generate_key_combine_insufficient_shares() {
    let temp = TempDir::new().unwrap();

    // Create share files
    let share0 = temp.path().join("share-0");
    fs::write(&share0, "share1").unwrap();

    let key_path = temp.path().join("reconstructed_key");
    let share_paths = share0.display().to_string();
    let result = generate_key_combine(&share_paths, &key_path, 2, true);

    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("insufficient shares provided"));
  }

  #[test]
  fn test_generate_key_combine_missing_share_file() {
    let temp = TempDir::new().unwrap();

    // Create one valid share file but reference a second missing one
    let share1 = temp.path().join("share-1");
    fs::write(&share1, "dummy_share_content").unwrap();

    let missing_share = temp.path().join("nonexistent_share");
    let share_paths =
      format!("{},{}", share1.display(), missing_share.display());

    let key_path = temp.path().join("reconstructed_key");
    let result = generate_key_combine(&share_paths, &key_path, 2, true);

    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("failed to read share file"));
  }
}
