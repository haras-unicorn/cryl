use crate::common::CrylResult;
use std::path::Path;

/// Change working directory by creating a subdirectory during export phase
///
/// # Arguments
/// * `path` - Path to the new working directory (relative to current)
///
/// # Description
/// Creates the specified directory and all parent directories if they don't exist.
/// Changes the current working directory to the specified path.
/// This exporter doesn't export any files - it only changes the working directory
/// context for subsequent operations.
pub fn export_working_directory(path: &Path) -> CrylResult<()> {
  // Create the directory and all parent directories
  std::fs::create_dir_all(path)?;

  // Change the current working directory
  std::env::set_current_dir(path)?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use serial_test::serial;
  use tempfile::TempDir;

  #[test]
  #[serial(working_directory)]
  fn test_export_working_directory_creates_dir() {
    let temp = TempDir::new().unwrap();
    let new_dir = temp.path().join("new_workdir");
    let cwd = std::env::current_dir().unwrap();

    export_working_directory(&new_dir).unwrap();

    assert!(new_dir.exists());
    assert!(new_dir.is_dir());
    assert_eq!(std::env::current_dir().unwrap(), new_dir);

    std::env::set_current_dir(cwd).unwrap();
  }

  #[test]
  #[serial(working_directory)]
  fn test_export_working_directory_nested() {
    let temp = TempDir::new().unwrap();
    let nested_dir = temp.path().join("a").join("b").join("c");
    let cwd = std::env::current_dir().unwrap();

    export_working_directory(&nested_dir).unwrap();

    assert!(nested_dir.exists());
    assert!(nested_dir.is_dir());
    assert_eq!(std::env::current_dir().unwrap(), nested_dir);

    std::env::set_current_dir(cwd).unwrap();
  }

  #[test]
  #[serial(working_directory)]
  fn test_export_working_directory_already_exists() {
    let temp = TempDir::new().unwrap();
    let existing_dir = temp.path().join("existing");
    let cwd = std::env::current_dir().unwrap();

    // Create the directory first
    std::fs::create_dir(&existing_dir).unwrap();

    // Should not fail if directory already exists
    export_working_directory(&existing_dir).unwrap();

    assert!(existing_dir.exists());
    assert!(existing_dir.is_dir());
    assert_eq!(std::env::current_dir().unwrap(), existing_dir);

    std::env::set_current_dir(cwd).unwrap();
  }

  #[test]
  #[serial(working_directory)]
  fn test_export_working_directory_absolute_path() {
    let temp = TempDir::new().unwrap();
    // Use absolute path
    let abs_dir = temp.path().canonicalize().unwrap().join("absolute_subdir");
    let cwd = std::env::current_dir().unwrap();

    export_working_directory(&abs_dir).unwrap();

    assert!(abs_dir.exists());
    assert!(abs_dir.is_dir());
    assert_eq!(std::env::current_dir().unwrap(), abs_dir);

    std::env::set_current_dir(cwd).unwrap();
  }
}
