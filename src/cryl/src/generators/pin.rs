use std::path::Path;

use crate::common::{generate_random_digits, save_atomic, CrylResult};

/// Generate a numeric PIN and save it to a file
///
/// # Arguments
/// * `name` - Path to save the PIN
/// * `length` - Number of digits in the PIN (default: 8)
/// * `renew` - Overwrite destination if it exists
pub fn generate_pin(name: &Path, length: u32, renew: bool) -> CrylResult<()> {
  let length = if length == 0 { 8 } else { length };
  let pin = generate_random_digits(length as usize)?;
  save_atomic(name, pin.as_bytes(), renew, false)?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;
  use tempfile::TempDir;

  #[test]
  fn test_generate_pin_default_length() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("test_pin");

    generate_pin(&path, 0, false).unwrap();

    assert!(path.exists());
    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content.len(), 8);
    assert!(content.chars().all(|c| c.is_ascii_digit()));
  }

  #[test]
  fn test_generate_pin_custom_length() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("test_pin");

    generate_pin(&path, 16, false).unwrap();

    assert!(path.exists());
    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content.len(), 16);
    assert!(content.chars().all(|c| c.is_ascii_digit()));
  }

  #[test]
  fn test_generate_pin_renew_false_no_overwrite() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("test_pin");

    fs::write(&path, "12345678").unwrap();
    generate_pin(&path, 8, false).unwrap();

    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, "12345678");
  }

  #[test]
  fn test_generate_pin_renew_true_overwrites() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("test_pin");

    fs::write(&path, "12345678").unwrap();
    generate_pin(&path, 8, true).unwrap();

    let content = fs::read_to_string(&path).unwrap();
    assert_ne!(content, "12345678");
    assert_eq!(content.len(), 8);
    assert!(content.chars().all(|c| c.is_ascii_digit()));
  }

  #[test]
  fn test_generate_pin_private_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let temp = TempDir::new().unwrap();
    let path = temp.path().join("test_pin");

    generate_pin(&path, 8, false).unwrap();

    let metadata = fs::metadata(&path).unwrap();
    let perms = metadata.permissions();
    assert_eq!(perms.mode() & 0o777, 0o600);
  }

  #[test]
  fn test_generate_pin_various_lengths() {
    let temp = TempDir::new().unwrap();

    for length in [4, 6, 10, 20] {
      let path = temp.path().join(format!("pin_{}", length));
      generate_pin(&path, length, false).unwrap();

      assert!(path.exists());
      let content = fs::read_to_string(&path).unwrap();
      assert_eq!(content.len(), length as usize);
      assert!(content.chars().all(|c| c.is_ascii_digit()));
    }
  }
}
