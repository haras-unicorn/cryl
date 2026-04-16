use std::path::Path;

use crate::common::CrylResult;

/// Change working directory by creating a subdirectory
///
/// # Arguments
/// * `path` - Path to the new working directory (relative to current)
///
/// # Description
/// Creates the specified directory and all parent directories if they don't exist.
/// This generator doesn't create any files - it only changes the working directory
/// context for subsequent operations.
pub fn generate_working_directory(path: &Path) -> CrylResult<()> {
  // Create the directory and all parent directories
  std::fs::create_dir_all(path)?;

  std::env::set_current_dir(path)?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use crate::generators::generate_text;

  use super::*;
  use std::{fs, path::PathBuf, str::FromStr};
  use tempfile::TempDir;

  #[test]
  fn test_generate_working_directory_creates_dir() {
    let temp = TempDir::new().unwrap();
    let new_dir = temp.path().join("new_workdir");

    generate_working_directory(&new_dir).unwrap();

    assert!(new_dir.exists());
    assert!(new_dir.is_dir());
    assert_eq!(std::env::current_dir().unwrap(), new_dir);
  }

  #[test]
  fn test_generate_working_directory_generates_in_new_working_directory() {
    let temp = TempDir::new().unwrap();
    let new_dir = temp.path().join("new_workdir");
    let text_file_name = "text";
    let text_file_content = "my text";
    let text_file_path = new_dir.join(text_file_name);
    let text_file_relative_path = PathBuf::from_str(text_file_name).unwrap();

    // Check that it isn't somehow the absolute path
    assert_eq!(text_file_relative_path.to_str().unwrap(), text_file_name);

    generate_working_directory(&new_dir).unwrap();
    generate_text(&text_file_relative_path, text_file_content, false).unwrap();

    assert!(new_dir.exists());
    assert!(new_dir.is_dir());
    assert_eq!(std::env::current_dir().unwrap(), new_dir);
    assert!(std::fs::exists(&text_file_path).unwrap());
    assert_eq!(
      std::fs::read_to_string(&text_file_path).unwrap(),
      text_file_content
    );
  }

  #[test]
  fn test_generate_working_directory_nested() {
    let temp = TempDir::new().unwrap();
    let nested_dir = temp.path().join("a").join("b").join("c");

    generate_working_directory(&nested_dir).unwrap();

    assert!(nested_dir.exists());
    assert!(nested_dir.is_dir());
    assert_eq!(std::env::current_dir().unwrap(), nested_dir);
  }

  #[test]
  fn test_generate_working_directory_already_exists() {
    let temp = TempDir::new().unwrap();
    let existing_dir = temp.path().join("existing");

    // Create the directory first
    fs::create_dir(&existing_dir).unwrap();

    // Should not fail if directory already exists
    generate_working_directory(&existing_dir).unwrap();

    assert!(existing_dir.exists());
    assert!(existing_dir.is_dir());
    assert_eq!(std::env::current_dir().unwrap(), existing_dir);
  }

  #[test]
  fn test_generate_working_directory_absolute_path() {
    let temp = TempDir::new().unwrap();
    // Use absolute path instead of relative to avoid changing working directories
    let abs_dir = temp.path().canonicalize().unwrap().join("absolute_subdir");

    generate_working_directory(&abs_dir).unwrap();

    assert!(abs_dir.exists());
    assert!(abs_dir.is_dir());
    assert_eq!(std::env::current_dir().unwrap(), abs_dir);
  }
}
