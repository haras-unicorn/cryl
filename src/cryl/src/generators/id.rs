use std::path::Path;

use crate::common::{generate_random_alphanumeric, save_atomic, CrylResult};

/// Generate a random alphanumeric id and save it to a file
///
/// # Arguments
/// * `name` - Path to save the id
/// * `length` - Number of characters in the id (default: 16)
/// * `renew` - Overwrite destination if it exists
pub fn generate_id(name: &Path, length: u32, renew: bool) -> CrylResult<()> {
  let length = if length == 0 { 16 } else { length };
  let id = generate_random_alphanumeric(length as usize)?;
  save_atomic(name, id.as_bytes(), renew, false)?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;
  use tempfile::TempDir;

  #[test]
  fn test_generate_id_default_length() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("test_id");

    generate_id(&path, 0, false).unwrap();

    assert!(path.exists());
    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content.len(), 16);
    assert!(content.chars().all(|c| c.is_ascii_alphanumeric()));
  }

  #[test]
  fn test_generate_id_custom_length() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("test_id");

    generate_id(&path, 32, false).unwrap();

    assert!(path.exists());
    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content.len(), 32);
    assert!(content.chars().all(|c| c.is_ascii_alphanumeric()));
  }

  #[test]
  fn test_generate_id_renew_false_no_overwrite() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("test_id");

    fs::write(&path, "original_id").unwrap();
    generate_id(&path, 16, false).unwrap();

    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, "original_id");
  }

  #[test]
  fn test_generate_id_renew_true_overwrites() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("test_id");

    fs::write(&path, "original_id").unwrap();
    generate_id(&path, 16, true).unwrap();

    let content = fs::read_to_string(&path).unwrap();
    assert_ne!(content, "original_id");
    assert_eq!(content.len(), 16);
    assert!(content.chars().all(|c| c.is_ascii_alphanumeric()));
  }

  #[test]
  fn test_generate_id_private_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let temp = TempDir::new().unwrap();
    let path = temp.path().join("test_id");

    generate_id(&path, 16, false).unwrap();

    let metadata = fs::metadata(&path).unwrap();
    let perms = metadata.permissions();
    assert_eq!(perms.mode() & 0o777, 0o600);
  }
}
