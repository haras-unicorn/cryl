use std::path::Path;

use crate::common::{CrylResult, generate_random_alphanumeric, save_atomic};

/// Generate a random alphanumeric key and save it to a file
///
/// # Arguments
/// * `name` - Path to save the key
/// * `length` - Number of characters in the key (default: 32)
/// * `renew` - Overwrite destination if it exists
pub fn generate_key(name: &Path, length: u32, renew: bool) -> CrylResult<()> {
  let length = if length == 0 { 32 } else { length };
  let key = generate_random_alphanumeric(length as usize)?;
  save_atomic(name, key.as_bytes(), renew, false)?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;
  use tempfile::TempDir;

  #[test]
  fn test_generate_key_default_length() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("test_key");

    generate_key(&path, 0, false).unwrap();

    assert!(path.exists());
    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content.len(), 32);
    assert!(content.chars().all(|c| c.is_ascii_alphanumeric()));
  }

  #[test]
  fn test_generate_key_custom_length() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("test_key");

    generate_key(&path, 64, false).unwrap();

    assert!(path.exists());
    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content.len(), 64);
    assert!(content.chars().all(|c| c.is_ascii_alphanumeric()));
  }

  #[test]
  fn test_generate_key_renew_false_no_overwrite() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("test_key");

    fs::write(&path, "original_key").unwrap();
    generate_key(&path, 32, false).unwrap();

    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, "original_key");
  }

  #[test]
  fn test_generate_key_renew_true_overwrites() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("test_key");

    fs::write(&path, "original_key").unwrap();
    generate_key(&path, 32, true).unwrap();

    let content = fs::read_to_string(&path).unwrap();
    assert_ne!(content, "original_key");
    assert_eq!(content.len(), 32);
    assert!(content.chars().all(|c| c.is_ascii_alphanumeric()));
  }

  #[test]
  fn test_generate_key_private_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let temp = TempDir::new().unwrap();
    let path = temp.path().join("test_key");

    generate_key(&path, 32, false).unwrap();

    let metadata = fs::metadata(&path).unwrap();
    let perms = metadata.permissions();
    assert_eq!(perms.mode() & 0o777, 0o600);
  }
}
