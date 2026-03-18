use crate::common::{CrylError, CrylResult};
use std::path::Path;

/// Copy generator - copies a file from source to destination
pub fn generate_copy(from: &Path, to: &Path, renew: bool) -> CrylResult<()> {
  // Check if source exists
  if !from.exists() {
    return Err(CrylError::Export {
      exporter: "copy".to_string(),
      message: format!("Source file not found: {:?}", from),
    });
  }

  // Read source content
  let content = std::fs::read(from)?;

  // Write to destination
  std::fs::write(to, content)?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use tempfile::TempDir;

  #[test]
  fn test_export_copy_success() {
    let temp = TempDir::new().unwrap();
    let from = temp.path().join("source");
    let to = temp.path().join("dest");

    std::fs::write(&from, "test content").unwrap();
    generate_copy(&from, &to, true).unwrap();

    assert!(to.exists());
    let content = std::fs::read_to_string(&to).unwrap();
    assert_eq!(content, "test content");
  }

  #[test]
  fn test_export_copy_missing() {
    let temp = TempDir::new().unwrap();
    let from = temp.path().join("nonexistent");
    let to = temp.path().join("dest");

    let result = generate_copy(&from, &to, true);
    assert!(result.is_err());
  }
}
