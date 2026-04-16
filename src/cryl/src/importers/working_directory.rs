use crate::common::CrylResult;
use std::path::Path;

/// Change working directory by creating a subdirectory during import phase
///
/// # Arguments
/// * `path` - Path to the new working directory (relative to current)
///
/// # Description
/// Creates the specified directory and all parent directories if they don't exist.
/// Changes the current working directory to the specified path.
/// This importer doesn't import any files - it only changes the working directory
/// context for subsequent operations.
pub fn import_working_directory(path: &Path) -> CrylResult<()> {
  // Create the directory and all parent directories
  std::fs::create_dir_all(path)?;

  // Change the current working directory
  std::env::set_current_dir(path)?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::importers::import_copy;
  use serial_test::serial;
  use tempfile::TempDir;

  #[test]
  #[serial(working_directory)]
  fn test_import_working_directory_creates_dir() {
    let temp = TempDir::new().unwrap();
    let new_dir = temp.path().join("new_workdir");
    let cwd = std::env::current_dir().unwrap();

    import_working_directory(&new_dir).unwrap();

    assert!(new_dir.exists());
    assert!(new_dir.is_dir());
    assert_eq!(std::env::current_dir().unwrap(), new_dir);

    std::env::set_current_dir(cwd).unwrap();
  }

  #[test]
  #[serial(working_directory)]
  fn test_import_working_directory_imports_in_new_working_directory() {
    let temp = TempDir::new().unwrap();
    let base_dir = temp.path().join("base");
    let sub_dir = base_dir.join("subdir");
    let source_file = base_dir.join("source.txt");
    let cwd = std::env::current_dir().unwrap();

    std::fs::create_dir_all(&base_dir).unwrap();
    std::fs::create_dir_all(&sub_dir).unwrap();
    std::fs::write(&source_file, "test content").unwrap();

    // Change to sub_dir
    import_working_directory(&sub_dir).unwrap();

    // Now import a file using relative path from parent directory
    let relative_from = "../source.txt";
    let dest_file = Path::new("imported.txt");
    import_copy(Path::new(relative_from), dest_file, false).unwrap();

    // Verify the file was imported to the current working directory
    let expected_dest = sub_dir.join("imported.txt");
    assert!(expected_dest.exists());
    assert_eq!(
      std::fs::read_to_string(&expected_dest).unwrap(),
      "test content"
    );

    std::env::set_current_dir(cwd).unwrap();
  }

  #[test]
  #[serial(working_directory)]
  fn test_import_working_directory_nested() {
    let temp = TempDir::new().unwrap();
    let nested_dir = temp.path().join("a").join("b").join("c");
    let cwd = std::env::current_dir().unwrap();

    import_working_directory(&nested_dir).unwrap();

    assert!(nested_dir.exists());
    assert!(nested_dir.is_dir());
    assert_eq!(std::env::current_dir().unwrap(), nested_dir);

    std::env::set_current_dir(cwd).unwrap();
  }

  #[test]
  #[serial(working_directory)]
  fn test_import_working_directory_already_exists() {
    let temp = TempDir::new().unwrap();
    let existing_dir = temp.path().join("existing");
    let cwd = std::env::current_dir().unwrap();

    // Create the directory first
    std::fs::create_dir(&existing_dir).unwrap();

    // Should not fail if directory already exists
    import_working_directory(&existing_dir).unwrap();

    assert!(existing_dir.exists());
    assert!(existing_dir.is_dir());
    assert_eq!(std::env::current_dir().unwrap(), existing_dir);

    std::env::set_current_dir(cwd).unwrap();
  }

  #[test]
  #[serial(working_directory)]
  fn test_import_working_directory_absolute_path() {
    let temp = TempDir::new().unwrap();
    // Use absolute path
    let abs_dir = temp.path().canonicalize().unwrap().join("absolute_subdir");
    let cwd = std::env::current_dir().unwrap();

    import_working_directory(&abs_dir).unwrap();

    assert!(abs_dir.exists());
    assert!(abs_dir.is_dir());
    assert_eq!(std::env::current_dir().unwrap(), abs_dir);

    std::env::set_current_dir(cwd).unwrap();
  }
}
