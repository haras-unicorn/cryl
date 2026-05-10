use crate::common::{
  CrylError, CrylResult, DirectoryListing, Format, deserialize_from_file,
  list_directory, save_atomic,
};
use std::path::{Path, PathBuf};

/// Copy importer - copies specified files from source to working directory
pub fn import_copy(
  from: &Path,
  format: Format,
  listing: &Path,
  allow_fail: bool,
) -> CrylResult<()> {
  // Check if source exists
  if !from.exists() || !from.is_dir() {
    if allow_fail {
      return Ok(());
    }
    return Err(CrylError::Import {
      importer: "copy".to_string(),
      message: format!("Source directory not found: {:?}", from),
    });
  }

  // Get listing first to exit early
  let listing: DirectoryListing = deserialize_from_file(listing, Some(format))?;

  // List directory
  for (key, content) in list_directory(from, &listing, allow_fail, "/")? {
    // Read source content
    let path = PathBuf::from_iter(key.split("/"));

    // Write to destination
    save_atomic(path, content.as_slice(), true, false)?;
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::common::TempCurrentDir;
  use serial_test::serial;
  use std::collections::HashMap;

  #[test]
  #[serial(working_directory)]
  fn test_import_copy_success() {
    let temp = TempCurrentDir::new().unwrap();
    let from_dir = temp.path().join("source_dir");
    std::fs::create_dir_all(&from_dir).unwrap();
    std::fs::write(from_dir.join("source.txt"), "test content").unwrap();

    let mut listing = HashMap::new();
    listing.insert("output.txt".to_owned(), PathBuf::from("source.txt"));
    let listing_path = temp.path().join("listing.json");
    serde_json::to_writer(
      std::fs::File::create(&listing_path).unwrap(),
      &DirectoryListing::Map(listing),
    )
    .unwrap();

    import_copy(&from_dir, Format::Json, &listing_path, false).unwrap();

    let output = temp.path().join("output.txt");
    assert!(output.exists());
    assert_eq!(std::fs::read_to_string(output).unwrap(), "test content");
  }

  #[test]
  #[serial(working_directory)]
  fn test_import_copy_missing_allow_fail() {
    let temp = TempCurrentDir::new().unwrap();
    let from_dir = temp.path().join("nonexistent_dir");

    let mut listing = HashMap::new();
    listing.insert("out.txt".to_owned(), PathBuf::from("source.txt"));
    let listing_path = temp.path().join("listing.json");
    serde_json::to_writer(
      std::fs::File::create(&listing_path).unwrap(),
      &DirectoryListing::Map(listing),
    )
    .unwrap();

    import_copy(&from_dir, Format::Json, &listing_path, true).unwrap();
  }

  #[test]
  #[serial(working_directory)]
  fn test_import_copy_missing_no_allow_fail() {
    let temp = TempCurrentDir::new().unwrap();
    let from_dir = temp.path().join("nonexistent_dir");

    let mut listing = HashMap::new();
    listing.insert("out.txt".to_owned(), PathBuf::from("source.txt"));
    let listing_path = temp.path().join("listing.json");
    serde_json::to_writer(
      std::fs::File::create(&listing_path).unwrap(),
      &DirectoryListing::Map(listing),
    )
    .unwrap();

    let result = import_copy(&from_dir, Format::Json, &listing_path, false);
    assert!(result.is_err());
  }

  #[test]
  #[serial(working_directory)]
  fn test_import_copy_from_subdir() {
    let temp = TempCurrentDir::new().unwrap();
    let from_dir = temp.path().join("source_dir");
    std::fs::create_dir_all(from_dir.join("subdir")).unwrap();
    std::fs::write(from_dir.join("subdir/source.txt"), "test content").unwrap();

    let mut listing = HashMap::new();
    listing.insert("output.txt".to_owned(), PathBuf::from("subdir/source.txt"));
    let listing_path = temp.path().join("listing.json");
    serde_json::to_writer(
      std::fs::File::create(&listing_path).unwrap(),
      &DirectoryListing::Map(listing),
    )
    .unwrap();

    import_copy(&from_dir, Format::Json, &listing_path, false).unwrap();

    let output = temp.path().join("output.txt");
    assert!(output.exists());
    assert_eq!(std::fs::read_to_string(output).unwrap(), "test content");
  }

  #[test]
  #[serial(working_directory)]
  fn test_import_copy_to_subdir() {
    let temp = TempCurrentDir::new().unwrap();
    let from_dir = temp.path().join("source_dir");
    std::fs::create_dir_all(&from_dir).unwrap();
    std::fs::write(from_dir.join("source.txt"), "test content").unwrap();

    let mut listing = HashMap::new();
    listing.insert("sub/output.txt".to_owned(), PathBuf::from("source.txt"));
    let listing_path = temp.path().join("listing.json");
    serde_json::to_writer(
      std::fs::File::create(&listing_path).unwrap(),
      &DirectoryListing::Map(listing),
    )
    .unwrap();

    import_copy(&from_dir, Format::Json, &listing_path, false).unwrap();

    let output = temp.path().join("sub").join("output.txt");
    assert!(output.exists());
    assert_eq!(std::fs::read_to_string(output).unwrap(), "test content");
  }
}
