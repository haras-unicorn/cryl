use crate::common::{save_atomic, CrylError, CrylResult};
use std::path::Path;

/// Copy importer - copies a file from source to destination
pub fn import_copy(from: &Path, to: &Path, allow_fail: bool) -> CrylResult<()> {
  // Check if source exists
  if !from.exists() {
    if allow_fail {
      return Ok(());
    }
    return Err(CrylError::Import {
      importer: "copy".to_string(),
      message: format!("Source file not found: {:?}", from),
    });
  }

  // Read source content
  let content = std::fs::read(from)?;

  // Write to destination
  save_atomic(to, content.as_slice(), true, false)?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use tempfile::TempDir;

  #[test]
  fn test_import_copy_success() {
    let temp = TempDir::new().unwrap();
    let from = temp.path().join("source");
    let to = temp.path().join("dest");

    std::fs::write(&from, "test content").unwrap();
    import_copy(&from, &to, false).unwrap();

    assert!(to.exists());
    let content = std::fs::read_to_string(&to).unwrap();
    assert_eq!(content, "test content");
  }

  #[test]
  fn test_import_copy_missing_allow_fail() {
    let temp = TempDir::new().unwrap();
    let from = temp.path().join("nonexistent");
    let to = temp.path().join("dest");

    // Should succeed when allow_fail is true
    import_copy(&from, &to, true).unwrap();
    assert!(!to.exists());
  }

  #[test]
  fn test_import_copy_missing_no_allow_fail() {
    let temp = TempDir::new().unwrap();
    let from = temp.path().join("nonexistent");
    let to = temp.path().join("dest");

    // Should fail when allow_fail is false
    let result = import_copy(&from, &to, false);
    assert!(result.is_err());
  }
}
