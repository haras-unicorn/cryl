use super::CrylResult;
use std::path::Path;

/// Save content to a file atomically
///
/// # Arguments
/// * `path` - Target path
/// * `content` - Content to write
/// * `renew` - Overwrite if exists
/// * `public` - Set public permissions (644) vs private (600)
pub fn save_atomic<P: AsRef<Path>>(
  path: P,
  content: &[u8],
  renew: bool,
  public: bool,
) -> CrylResult<()> {
  use std::fs;
  use std::os::unix::fs::PermissionsExt;

  let path = path.as_ref();

  // Check if file exists and we shouldn't renew
  if !renew && path.exists() {
    return Ok(());
  }

  let tmp_path = path.with_extension("tmp");

  // Write to temp file
  fs::write(&tmp_path, content)?;

  // Set permissions
  let perms = if public { 0o644 } else { 0o600 };
  let mut permissions = fs::metadata(&tmp_path)?.permissions();
  permissions.set_mode(perms);
  fs::set_permissions(&tmp_path, permissions)?;

  // Atomic rename
  fs::rename(&tmp_path, path)?;

  Ok(())
}

/// Read file content if it exists, otherwise return None
pub fn read_file_if_exists<P: AsRef<Path>>(
  path: P,
) -> CrylResult<Option<String>> {
  match std::fs::read_to_string(path) {
    Ok(content) => Ok(Some(content)),
    Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
    Err(e) => Err(e.into()),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use tempfile::TempDir;

  #[test]
  fn test_save_atomic_private() {
    use std::os::unix::fs::PermissionsExt;
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("test");

    save_atomic(&path, b"content", false, false).unwrap();

    assert!(path.exists());
    let content = std::fs::read_to_string(&path).unwrap();
    assert_eq!(content, "content");

    let metadata = std::fs::metadata(&path).unwrap();
    let perms = metadata.permissions();
    assert_eq!(perms.mode() & 0o777, 0o600);
  }

  #[test]
  fn test_save_atomic_public() {
    use std::os::unix::fs::PermissionsExt;
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("test");

    save_atomic(&path, b"content", false, true).unwrap();

    let metadata = std::fs::metadata(&path).unwrap();
    let perms = metadata.permissions();
    assert_eq!(perms.mode() & 0o777, 0o644);
  }

  #[test]
  fn test_save_atomic_renew_false_no_overwrite() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("test");

    std::fs::write(&path, "original").unwrap();
    save_atomic(&path, b"new", false, false).unwrap();

    let content = std::fs::read_to_string(&path).unwrap();
    assert_eq!(content, "original");
  }

  #[test]
  fn test_save_atomic_renew_true_overwrites() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("test");

    std::fs::write(&path, "original").unwrap();
    save_atomic(&path, b"new", true, false).unwrap();

    let content = std::fs::read_to_string(&path).unwrap();
    assert_eq!(content, "new");
  }

  #[test]
  fn test_read_file_if_exists_found() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("test");
    std::fs::write(&path, "content").unwrap();

    let result = read_file_if_exists(&path).unwrap();
    assert_eq!(result, Some("content".to_string()));
  }

  #[test]
  fn test_read_file_if_exists_not_found() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("nonexistent");

    let result = read_file_if_exists(&path).unwrap();
    assert_eq!(result, None);
  }
}
