use crate::common::{
  CrylError, CrylResult, DirectoryListing, Format, deserialize_from_file,
  list_directory,
};
use std::path::{Path, PathBuf};

/// Copy exporter - copies specified files from working directory to destination
pub fn export_copy(
  format: Format,
  listing: &Path,
  to: &Path,
) -> CrylResult<()> {
  // Check if destination exists
  if !to.exists() || !to.is_dir() {
    return Err(CrylError::Export {
      exporter: "copy".to_string(),
      message: format!("Destination invalid: {:?}", to),
    });
  }

  // Get listing first to exit early
  let listing: DirectoryListing = deserialize_from_file(listing, Some(format))?;

  // List directory
  for (key, content) in
    list_directory(std::env::current_dir()?, &listing, false, "/")?
  {
    let path = to.join(PathBuf::from_iter(key.split("/")));
    if let Some(parent) = path.parent() {
      std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, content)?;
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
  fn test_export_copy_success() {
    let temp = TempCurrentDir::new().unwrap();
    let dest = temp.path().join("dest");
    std::fs::create_dir_all(&dest).unwrap();

    std::fs::write("source.txt", "test content").unwrap();
    let mut listing = HashMap::new();
    listing.insert("output.txt".to_owned(), PathBuf::from("source.txt"));
    let listing_path = temp.path().join("listing.json");
    serde_json::to_writer(
      std::fs::File::create(&listing_path).unwrap(),
      &DirectoryListing::Map(listing),
    )
    .unwrap();

    export_copy(Format::Json, &listing_path, &dest).unwrap();

    let output = dest.join("output.txt");
    assert!(output.exists());
    assert_eq!(std::fs::read_to_string(output).unwrap(), "test content");
  }

  #[test]
  #[serial(working_directory)]
  fn test_export_copy_missing() {
    let temp = TempCurrentDir::new().unwrap();
    let dest = temp.path().join("dest");
    std::fs::create_dir_all(&dest).unwrap();

    let mut listing = HashMap::new();
    listing.insert("out.txt".to_owned(), PathBuf::from("nonexistent.txt"));
    let listing_path = temp.path().join("listing.json");
    serde_json::to_writer(
      std::fs::File::create(&listing_path).unwrap(),
      &DirectoryListing::Map(listing),
    )
    .unwrap();

    let result = export_copy(Format::Json, &listing_path, &dest);
    assert!(result.is_err());
  }

  #[test]
  #[serial(working_directory)]
  fn test_export_copy_from_subdir() {
    let temp = TempCurrentDir::new().unwrap();
    let dest = temp.path().join("dest");
    std::fs::create_dir_all(&dest).unwrap();

    std::fs::create_dir_all("subdir").unwrap();
    std::fs::write("subdir/source.txt", "test content").unwrap();
    let mut listing = HashMap::new();
    listing.insert("output.txt".to_owned(), PathBuf::from("subdir/source.txt"));
    let listing_path = temp.path().join("listing.json");
    serde_json::to_writer(
      std::fs::File::create(&listing_path).unwrap(),
      &DirectoryListing::Map(listing),
    )
    .unwrap();

    export_copy(Format::Json, &listing_path, &dest).unwrap();

    let output = dest.join("output.txt");
    assert!(output.exists());
    assert_eq!(std::fs::read_to_string(output).unwrap(), "test content");
  }

  #[test]
  #[serial(working_directory)]
  fn test_export_copy_to_subdir() {
    let temp = TempCurrentDir::new().unwrap();
    let dest = temp.path().join("subdir").join("dest");
    std::fs::create_dir_all(&dest).unwrap();

    std::fs::write("source.txt", "test content").unwrap();
    let mut listing = HashMap::new();
    listing.insert("sub/output.txt".to_owned(), PathBuf::from("source.txt"));
    let listing_path = temp.path().join("listing.json");
    serde_json::to_writer(
      std::fs::File::create(&listing_path).unwrap(),
      &DirectoryListing::Map(listing),
    )
    .unwrap();

    export_copy(Format::Json, &listing_path, &dest).unwrap();

    let output = dest.join("sub").join("output.txt");
    assert!(output.exists());
    assert_eq!(std::fs::read_to_string(output).unwrap(), "test content");
  }
}
