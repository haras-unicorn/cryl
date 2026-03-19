use std::path::Path;

use crate::common::{CrylResult, save_atomic};

/// Write a text file as part of generation
///
/// # Arguments
/// * `name` - Path to save the text file
/// * `text` - Text content to write
/// * `renew` - Overwrite destination if it exists
pub fn generate_text(name: &Path, text: &str, renew: bool) -> CrylResult<()> {
  save_atomic(name, text.as_bytes(), renew, false)?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;
  use tempfile::TempDir;

  #[test]
  fn test_generate_text_creates_file() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("test_text.txt");

    generate_text(&path, "Hello, World!", false).unwrap();

    assert!(path.exists());
    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, "Hello, World!");
  }

  #[test]
  fn test_generate_text_empty_content() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("empty.txt");

    generate_text(&path, "", false).unwrap();

    assert!(path.exists());
    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, "");
  }

  #[test]
  fn test_generate_text_multiline_content() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("multiline.txt");

    let content = "Line 1\nLine 2\nLine 3";
    generate_text(&path, content, false).unwrap();

    assert!(path.exists());
    let read_content = fs::read_to_string(&path).unwrap();
    assert_eq!(read_content, content);
  }

  #[test]
  fn test_generate_text_renew_false_no_overwrite() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("test_text.txt");

    fs::write(&path, "Original content").unwrap();
    generate_text(&path, "New content", false).unwrap();

    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, "Original content");
  }

  #[test]
  fn test_generate_text_renew_true_overwrites() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("test_text.txt");

    fs::write(&path, "Original content").unwrap();
    generate_text(&path, "New content", true).unwrap();

    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, "New content");
  }

  #[test]
  fn test_generate_text_private_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let temp = TempDir::new().unwrap();
    let path = temp.path().join("test_text.txt");

    generate_text(&path, "Secret content", false).unwrap();

    let metadata = fs::metadata(&path).unwrap();
    let perms = metadata.permissions();
    assert_eq!(perms.mode() & 0o777, 0o600);
  }

  #[test]
  fn test_generate_text_unicode_content() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("unicode.txt");

    // cspell:disable-next-line
    let content = "Hello 世界! 🌍 émojis";
    generate_text(&path, content, false).unwrap();

    assert!(path.exists());
    let read_content = fs::read_to_string(&path).unwrap();
    assert_eq!(read_content, content);
  }
}
