use super::{CrylError, CrylResult};
use std::{
  collections::{HashMap, HashSet},
  path::{Path, PathBuf},
};

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
  if let Some(parent) = tmp_path.parent() {
    fs::create_dir_all(parent)?;
  }
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

pub fn strip_current_directory<P: AsRef<Path>, C: AsRef<Path>>(
  path: P,
  current_dir: C,
) -> PathBuf {
  let path = path.as_ref();
  let path = path.strip_prefix(".").unwrap_or(path);
  path
    .strip_prefix(current_dir.as_ref())
    .unwrap_or(path)
    .to_owned()
}

pub fn read_directory_files<P: AsRef<Path>>(
  path: P,
  recurse: bool,
) -> CrylResult<Vec<PathBuf>> {
  fn go(
    paths: &mut Vec<PathBuf>,
    visited: &mut HashSet<PathBuf>,
    current_dir: &Path,
    path: &Path,
    recurse: bool,
    recursed: bool,
  ) -> CrylResult<()> {
    if path.is_file() {
      paths.push(strip_current_directory(path, current_dir));
      return Ok(());
    }

    let canon = path.canonicalize()?;
    if visited.contains(&canon) {
      return Ok(());
    }
    visited.insert(canon);

    if path.is_dir() && (recurse || !recursed) {
      for entry in std::fs::read_dir(path)? {
        go(paths, visited, current_dir, &entry?.path(), recurse, true)?;
      }
      return Ok(());
    }

    Ok(())
  }

  let mut paths = vec![];
  let mut visited = HashSet::new();
  let current_dir = std::env::current_dir()?;

  go(
    &mut paths,
    &mut visited,
    &current_dir,
    path.as_ref(),
    recurse,
    false,
  )?;

  Ok(paths)
}

#[derive(
  Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase", tag = "type", content = "value")]
pub enum DirectoryListing {
  /// List only files in current directory
  Flat,
  /// List files in current directory and subdirectories recursively
  Deep,
  /// Checks that files in the current directory or subdirectories exist
  List(Vec<PathBuf>),
  /// Check that file values in the current directory or subdirectories exist
  Map(HashMap<String, PathBuf>),
}

pub fn list_directory<P: AsRef<Path>>(
  path: P,
  listing: DirectoryListing,
) -> CrylResult<HashMap<String, PathBuf>> {
  match listing {
    DirectoryListing::Flat | DirectoryListing::Deep => {
      read_directory_files(path, matches!(listing, DirectoryListing::Deep)).map(
        |files| {
          files
            .into_iter()
            .map(|file| (file.to_string_lossy().to_string(), file))
            .collect::<HashMap<_, _>>()
        },
      )
    }
    DirectoryListing::List(list) => {
      let current_dir = std::env::current_dir()?;
      let read = read_directory_files(path, true)?;
      let mut result_map = HashMap::new();
      for file in list {
        let stripped = strip_current_directory(&file, &current_dir);
        if !read.contains(&stripped) {
          return Err(CrylError::DirectoryListing(file));
        }
        result_map.insert(file.to_string_lossy().to_string(), stripped);
      }
      Ok(result_map)
    }
    DirectoryListing::Map(map) => {
      let current_dir = std::env::current_dir()?;
      let read = read_directory_files(path, true)?;
      let mut result_map = HashMap::new();
      for (key, file) in map {
        let stripped = strip_current_directory(&file, &current_dir);
        if !read.contains(&stripped) {
          return Err(CrylError::DirectoryListing(file.clone()));
        }
        result_map.insert(key, stripped);
      }
      Ok(result_map)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use tempfile::TempDir;

  #[cfg(unix)]
  fn symbolic_link<P: AsRef<Path>, Q: AsRef<Path>>(
    original: P,
    link: Q,
  ) -> CrylResult<()> {
    std::os::unix::fs::symlink(original, link)?;
    Ok(())
  }

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
  fn test_save_atomic_subdir() {
    use std::os::unix::fs::PermissionsExt;
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("subdir").join("test");

    save_atomic(&path, b"content", false, true).unwrap();

    assert!(path.exists());
    let content = std::fs::read_to_string(&path).unwrap();
    assert_eq!(content, "content");

    let metadata = std::fs::metadata(&path).unwrap();
    let perms = metadata.permissions();
    assert_eq!(perms.mode() & 0o777, 0o644);
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

  #[test]
  fn test_read_directory_files_recursively_reads_file() {
    let temp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(temp.path(), "content").unwrap();
    assert_eq!(
      read_directory_files(temp.path(), false).unwrap(),
      vec![temp.path()]
    );
  }

  #[test]
  fn test_read_directory_files_recursively_follows_symlink() {
    let original = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(original.path(), "content").unwrap();
    let temp_dir = TempDir::new().unwrap();
    let link = temp_dir.path().join("link");
    symbolic_link(&original, &link).unwrap();
    assert_eq!(read_directory_files(&link, false).unwrap(), vec![link]);
  }

  #[test]
  fn test_read_directory_files_recursively_reads_dir() {
    let temp_dir = TempDir::new().unwrap();
    let file = temp_dir.path().join("file");
    std::fs::write(&file, "content").unwrap();
    assert_eq!(
      read_directory_files(temp_dir.path(), false).unwrap(),
      vec![file]
    );
  }

  #[test]
  fn test_read_directory_files_recursively_reads_dir_recursively() {
    let temp_dir = TempDir::new().unwrap();
    let file = temp_dir.path().join("subdir").join("file");
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, "content").unwrap();
    assert_eq!(
      read_directory_files(temp_dir.path(), true).unwrap(),
      vec![file]
    );
  }

  #[test]
  fn test_read_directory_files_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    assert_eq!(
      read_directory_files(temp_dir.path(), false).unwrap(),
      Vec::<PathBuf>::new()
    );
    assert_eq!(
      read_directory_files(temp_dir.path(), true).unwrap(),
      Vec::<PathBuf>::new()
    );
  }

  #[test]
  fn test_read_directory_files_nested_structure() {
    let temp_dir = TempDir::new().unwrap();

    // Create a complex structure
    let file1 = temp_dir.path().join("root_file.txt");
    let subdir1 = temp_dir.path().join("subdir1");
    let subdir2 = temp_dir.path().join("subdir2");

    std::fs::create_dir(&subdir1).unwrap();
    std::fs::create_dir(&subdir2).unwrap();

    let file2 = subdir1.join("file2.txt");
    let file3 = subdir2.join("file3.txt");
    let file4 = subdir1.join("nested").join("file4.txt");

    std::fs::create_dir_all(file4.parent().unwrap()).unwrap();

    for file in &[&file1, &file2, &file3, &file4] {
      std::fs::write(file, "content").unwrap();
    }

    // Test non-recursive
    let non_recursive = read_directory_files(temp_dir.path(), false).unwrap();
    assert_eq!(non_recursive.len(), 1);
    assert!(non_recursive.contains(&file1));

    // Test recursive
    let recursive = read_directory_files(temp_dir.path(), true).unwrap();
    assert_eq!(recursive.len(), 4);
    for file in &[&file1, &file2, &file3, &file4] {
      assert!(recursive.contains(file));
    }
  }

  #[test]
  fn test_read_directory_files_symlink_loop() {
    let temp_dir = TempDir::new().unwrap();

    // Create symlink that points to parent directory
    let subdir = temp_dir.path().join("subdir");
    std::fs::create_dir(&subdir).unwrap();

    let link_back = subdir.join("link_to_parent");
    symbolic_link(temp_dir.path(), &link_back).unwrap();

    // This should not cause infinite recursion
    let result = read_directory_files(temp_dir.path(), true).unwrap();
    assert!(result.is_empty());
  }

  #[test]
  fn test_read_directory_files_mixed_types() {
    let temp_dir = TempDir::new().unwrap();

    // Create file, directory, and symlink
    let file = temp_dir.path().join("file.txt");
    let subdir = temp_dir.path().join("subdir");
    let link = temp_dir.path().join("link.txt");

    std::fs::write(&file, "content").unwrap();
    std::fs::create_dir(&subdir).unwrap();
    symbolic_link(&file, &link).unwrap();

    let result = read_directory_files(temp_dir.path(), false).unwrap();

    // Should find both file and symlink
    assert_eq!(result.len(), 2);
    assert!(result.contains(&file));
    assert!(result.contains(&link));
    assert!(!result.contains(&subdir)); // Shouldn't include directories
  }

  #[test]
  fn test_read_directory_files_hidden_files() {
    let temp_dir = TempDir::new().unwrap();

    // Create hidden files (starting with dot)
    let hidden = temp_dir.path().join(".hidden_file");
    let normal = temp_dir.path().join("normal_file");

    std::fs::write(&hidden, "content").unwrap();
    std::fs::write(&normal, "content").unwrap();

    let result = read_directory_files(temp_dir.path(), false).unwrap();

    // Should find both hidden and normal files
    assert_eq!(result.len(), 2);
    assert!(result.contains(&hidden));
    assert!(result.contains(&normal));
  }

  #[test]
  fn test_list_directory_flat() {
    let temp = TempDir::new().unwrap();
    std::fs::write(temp.path().join("a.txt"), "a").unwrap();
    std::fs::write(temp.path().join("b.rs"), "b").unwrap();

    let result = list_directory(temp.path(), DirectoryListing::Flat).unwrap();

    assert_eq!(result.len(), 2);
    assert!(result.contains_key(&format!("{}/a.txt", temp.path().display())));
    assert!(result.contains_key(&format!("{}/b.rs", temp.path().display())));
  }

  #[test]
  fn test_list_directory_deep() {
    let temp = TempDir::new().unwrap();
    let sub = temp.path().join("sub");
    std::fs::create_dir(&sub).unwrap();
    std::fs::write(sub.join("nested.txt"), "test").unwrap();

    let result = list_directory(temp.path(), DirectoryListing::Deep).unwrap();

    assert_eq!(result.len(), 1);
    assert!(
      result.contains_key(&format!("{}/sub/nested.txt", temp.path().display()))
    );
  }

  #[test]
  fn test_list_directory_custom_list_ok() {
    let temp = TempDir::new().unwrap();
    let file1 = temp.path().join("file1.txt");
    let file2 = temp.path().join("subdir").join("file2.txt");
    std::fs::create_dir_all(file2.parent().unwrap()).unwrap();
    std::fs::write(&file1, "a").unwrap();
    std::fs::write(&file2, "b").unwrap();

    let list = vec![file1.clone(), file2.clone()];
    let result =
      list_directory(temp.path(), DirectoryListing::List(list)).unwrap();

    assert_eq!(result.len(), 2);
    assert_eq!(result[&file1.to_string_lossy().to_string()], file1);
    assert_eq!(result[&file2.to_string_lossy().to_string()], file2);
  }

  #[test]
  fn test_list_directory_custom_list_missing() {
    let temp = TempDir::new().unwrap();
    let missing = temp.path().join("ghost.txt");
    let list = vec![missing];

    let result = list_directory(temp.path(), DirectoryListing::List(list));
    assert!(result.is_err());
    assert!(matches!(result, Err(CrylError::DirectoryListing(_))));
  }

  #[test]
  fn test_list_directory_custom_map_ok() {
    let temp = TempDir::new().unwrap();
    let file1 = temp.path().join("a.txt");
    let file2 = temp.path().join("b.txt");
    std::fs::write(&file1, "a").unwrap();
    std::fs::write(&file2, "b").unwrap();

    let mut map = HashMap::new();
    map.insert("alias1".to_string(), file1.clone());
    map.insert("alias2".to_string(), file2.clone());

    let result =
      list_directory(temp.path(), DirectoryListing::Map(map.clone())).unwrap();

    assert_eq!(result, map);
  }

  #[test]
  fn test_list_directory_custom_map_missing() {
    let temp = TempDir::new().unwrap();
    let mut map = HashMap::new();
    map.insert("ghost".to_string(), temp.path().join("nonexistent.txt"));

    let result = list_directory(temp.path(), DirectoryListing::Map(map));
    assert!(result.is_err());
    assert!(matches!(result, Err(CrylError::DirectoryListing(_))));
  }

  #[test]
  fn test_list_directory_empty_dir() {
    let temp = TempDir::new().unwrap();

    // Flat on empty dir
    let flat = list_directory(temp.path(), DirectoryListing::Flat).unwrap();
    assert!(flat.is_empty());

    // Deep on empty dir
    let deep = list_directory(temp.path(), DirectoryListing::Deep).unwrap();
    assert!(deep.is_empty());

    // Custom list with no files should error
    let list = DirectoryListing::List(vec![]);
    let result = list_directory(temp.path(), list).unwrap();
    assert!(result.is_empty());

    // Custom map with no files
    let map = DirectoryListing::Map(HashMap::new());
    let result = list_directory(temp.path(), map).unwrap();
    assert!(result.is_empty());
  }

  #[test]
  fn test_list_directory_symlink_in_dir() {
    use std::os::unix::fs::symlink;
    let temp = TempDir::new().unwrap();
    let target = temp.path().join("target.txt");
    let link = temp.path().join("link.txt");
    std::fs::write(&target, "content").unwrap();
    symlink(&target, &link).unwrap();

    let result = list_directory(temp.path(), DirectoryListing::Flat).unwrap();

    // Should include symlink file entry
    assert_eq!(result.len(), 2);
    assert!(result.contains_key(&link.to_string_lossy().to_string()));
  }
}
