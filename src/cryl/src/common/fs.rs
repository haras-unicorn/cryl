use itertools::{Either, Itertools};

use super::{CrylError, CrylResult};
use std::{
  any::Any,
  collections::HashMap,
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

#[derive(Debug, Clone)]
pub enum InMemoryFilesystem {
  File(Vec<u8>),
  Dir(HashMap<String, InMemoryFilesystem>),
}

impl TryFrom<&serde_yaml::Value> for InMemoryFilesystem {
  type Error = CrylError;

  fn try_from(value: &serde_yaml::Value) -> Result<Self, Self::Error> {
    if let Some(bytes) = value.as_str().map(|value| value.bytes()) {
      return Ok(Self::File(bytes.collect()));
    }

    if let Some(map) = value.as_mapping() {
      let mut result = HashMap::new();
      for (key, value) in map {
        let Some(key) = key.as_str() else {
          continue;
        };
        let value = Self::try_from(value)?;
        result.insert(key.to_owned(), value);
      }

      return Ok(Self::Dir(result));
    }

    Err(CrylError::InMemoryFilesystem(format!(
      "Failed parsing YAML value of type '{:?}'",
      value.type_id(),
    )))
  }
}

impl TryFrom<&serde_json::Value> for InMemoryFilesystem {
  type Error = CrylError;

  fn try_from(value: &serde_json::Value) -> Result<Self, Self::Error> {
    if let Some(bytes) = value.as_str().map(|value| value.bytes()) {
      return Ok(InMemoryFilesystem::File(bytes.collect()));
    }

    if let Some(map) = value.as_object() {
      let mut result = HashMap::new();
      for (key, value) in map {
        let value = Self::try_from(value)?;
        result.insert(key.to_owned(), value);
      }
      return Ok(Self::Dir(result));
    }

    Err(CrylError::InMemoryFilesystem(format!(
      "Failed parsing JSON value of type '{:?}'",
      value.type_id(),
    )))
  }
}

impl TryFrom<&toml::Value> for InMemoryFilesystem {
  type Error = CrylError;

  fn try_from(value: &toml::Value) -> Result<Self, Self::Error> {
    if let Some(bytes) = value.as_str().map(|value| value.bytes()) {
      return Ok(InMemoryFilesystem::File(bytes.collect()));
    }

    if let Some(map) = value.as_table() {
      let mut result = HashMap::new();
      for (key, value) in map {
        let value = Self::try_from(value)?;
        result.insert(key.to_owned(), value);
      }
      return Ok(Self::Dir(result));
    }

    Err(CrylError::InMemoryFilesystem(format!(
      "Failed parsing TOML value of type '{:?}'",
      value.type_id(),
    )))
  }
}

impl TryFrom<&Path> for InMemoryFilesystem {
  type Error = CrylError;

  fn try_from(value: &Path) -> Result<Self, Self::Error> {
    if value.is_file() {
      return Ok(Self::File(std::fs::read(value)?));
    }

    let mut result = HashMap::new();
    for entry_result in value.read_dir()? {
      let entry = entry_result?;

      let key = entry
        .path()
        .strip_prefix(value)
        .unwrap_or(&entry.path())
        .as_os_str()
        .to_string_lossy()
        .to_string();
      let value = Self::try_from(entry.path())?;
      result.insert(key, value);
    }
    Ok(Self::Dir(result))
  }
}

impl TryFrom<PathBuf> for InMemoryFilesystem {
  type Error = CrylError;

  fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
    Self::try_from(value.as_path())
  }
}

impl From<InMemoryFilesystem> for serde_yaml::Value {
  fn from(val: InMemoryFilesystem) -> Self {
    match val {
      InMemoryFilesystem::File(content) => serde_yaml::Value::String(
        String::from_utf8_lossy(content.as_slice()).to_string(),
      ),
      InMemoryFilesystem::Dir(map) => {
        let mut result = serde_yaml::Mapping::new();
        for (key, value) in map {
          result.insert(
            serde_yaml::Value::String(key),
            InMemoryFilesystem::into(value),
          );
        }

        serde_yaml::Value::Mapping(result)
      }
    }
  }
}

impl From<InMemoryFilesystem> for serde_json::Value {
  fn from(val: InMemoryFilesystem) -> Self {
    match val {
      InMemoryFilesystem::File(content) => serde_json::Value::String(
        String::from_utf8_lossy(content.as_slice()).to_string(),
      ),
      InMemoryFilesystem::Dir(map) => {
        let mut result = serde_json::Map::<String, serde_json::Value>::new();
        for (key, value) in map {
          result.insert(key, InMemoryFilesystem::into(value));
        }

        serde_json::Value::Object(result)
      }
    }
  }
}

impl From<InMemoryFilesystem> for toml::Value {
  fn from(val: InMemoryFilesystem) -> Self {
    match val {
      InMemoryFilesystem::File(content) => toml::Value::String(
        String::from_utf8_lossy(content.as_slice()).to_string(),
      ),
      InMemoryFilesystem::Dir(map) => {
        let mut result = toml::map::Map::<String, toml::Value>::new();
        for (key, value) in map {
          result.insert(key, InMemoryFilesystem::into(value));
        }

        toml::Value::Table(result)
      }
    }
  }
}

impl InMemoryFilesystem {
  pub fn trim_dir(
    dir: &HashMap<String, InMemoryFilesystem>,
  ) -> HashMap<String, Vec<u8>> {
    let mut result = HashMap::new();
    for (key, value) in dir {
      if let InMemoryFilesystem::File(content) = value {
        result.insert(key.clone(), content.clone());
      }
    }
    result
  }

  pub fn flatten(
    &self,
    separator: &str,
  ) -> Either<Vec<u8>, HashMap<String, Vec<u8>>> {
    match self {
      InMemoryFilesystem::File(content) => Either::Left(content.clone()),
      InMemoryFilesystem::Dir(dir) => {
        Either::Right(Self::flatten_dir(dir, separator))
      }
    }
  }

  pub fn flatten_dir(
    map: &HashMap<String, InMemoryFilesystem>,
    separator: &str,
  ) -> HashMap<String, Vec<u8>> {
    let mut result = HashMap::new();
    for (key, value) in map {
      match value.flatten(separator) {
        Either::Left(content) => {
          result.insert(key.clone(), content.clone());
        }
        Either::Right(inner) => {
          for (sub_key, content) in inner {
            result
              .insert(key.to_owned() + separator + &sub_key, content.clone());
          }
        }
      };
    }
    result
  }

  pub fn populate_list(
    dir: &HashMap<String, InMemoryFilesystem>,
    list: &Vec<PathBuf>,
    allow_fail: bool,
    separator: &str,
  ) -> CrylResult<HashMap<String, Vec<u8>>> {
    let flattened = Self::flatten_dir(dir, separator);
    let mut result = HashMap::new();
    for path in list {
      let key = path
        .strip_prefix(".")
        .unwrap_or(path)
        .iter()
        .map(|component| component.to_string_lossy())
        .join(separator);
      let content = flattened.get(&key);
      if let Some(content) = content {
        result.insert(key, content.clone());
      } else if !allow_fail {
        return Err(CrylError::DirectoryListing(path.clone()));
      }
    }
    Ok(result)
  }

  pub fn populate_map(
    dir: &HashMap<String, InMemoryFilesystem>,
    map: &HashMap<String, PathBuf>,
    allow_fail: bool,
    separator: &str,
  ) -> CrylResult<HashMap<String, Vec<u8>>> {
    let flattened = Self::flatten_dir(dir, separator);
    let mut result = HashMap::new();
    for (key, path) in map {
      let path_key = path
        .strip_prefix(".")
        .unwrap_or(path)
        .iter()
        .map(|component| component.to_string_lossy())
        .join(separator);
      let content = flattened.get(&path_key);
      if let Some(content) = content {
        result.insert(key.clone(), content.clone());
      } else if !allow_fail {
        return Err(CrylError::DirectoryListing(path.clone()));
      }
    }
    Ok(result)
  }

  pub fn from_listing(
    listing: HashMap<String, Vec<u8>>,
    separator: &str,
  ) -> Self {
    let mut result = HashMap::new();
    for (key, content) in listing.into_iter() {
      let components = key.split(separator).collect::<Vec<_>>();
      let mut map = &mut result;
      for (last, current) in
        components.iter().enumerate().map(|(index, current)| {
          (index == components.len().saturating_sub(1), *current)
        })
      {
        if last {
          map.insert(current.to_owned(), InMemoryFilesystem::File(content));
          break;
        } else {
          map = if let InMemoryFilesystem::Dir(dir) = map
            .entry(current.to_owned())
            .or_insert(InMemoryFilesystem::Dir(HashMap::new()))
          {
            dir
          } else {
            #[allow(clippy::unreachable, reason = "i just put it in")]
            {
              unreachable!()
            }
          };
        }
      }
    }
    Self::Dir(result)
  }
}

pub fn list_directory<
  P: TryInto<InMemoryFilesystem, Error = CrylError> + Clone + std::fmt::Debug,
>(
  source: P,
  listing: &DirectoryListing,
  allow_fail: bool,
  separator: &str,
) -> CrylResult<HashMap<String, Vec<u8>>> {
  let InMemoryFilesystem::Dir(dir) = source.clone().try_into()? else {
    return Err(CrylError::InMemoryFilesystem(format!(
      "A directory was expected when a file '{source:?}' was provided"
    )));
  };

  match listing {
    DirectoryListing::Flat => Ok(InMemoryFilesystem::trim_dir(&dir)),
    DirectoryListing::Deep => {
      Ok(InMemoryFilesystem::flatten_dir(&dir, separator))
    }
    DirectoryListing::List(list) => Ok(InMemoryFilesystem::populate_list(
      &dir, list, allow_fail, separator,
    )?),
    DirectoryListing::Map(map) => Ok(InMemoryFilesystem::populate_map(
      &dir, map, allow_fail, separator,
    )?),
  }
}

#[cfg(test)]
mod tests {
  use std::str::FromStr;

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
  fn test_list_directory_flat() {
    let temp = TempDir::new().unwrap();
    std::fs::write(temp.path().join("a.txt"), "a").unwrap();
    std::fs::write(temp.path().join("b.rs"), "b").unwrap();

    let result =
      list_directory(temp.path(), &DirectoryListing::Flat, false, "/").unwrap();

    assert_eq!(result.len(), 2);
    assert!(result.contains_key("a.txt"));
    assert!(result.contains_key("b.rs"));
  }

  #[test]
  fn test_list_directory_deep() {
    let temp = TempDir::new().unwrap();
    let sub = temp.path().join("sub");
    std::fs::create_dir(&sub).unwrap();
    std::fs::write(sub.join("nested.txt"), "test").unwrap();

    let result =
      list_directory(temp.path(), &DirectoryListing::Deep, false, "/").unwrap();

    assert_eq!(result.len(), 1);
    assert!(result.contains_key("sub/nested.txt"));
  }

  #[test]
  fn test_list_directory_custom_list_ok() {
    let temp = TempDir::new().unwrap();
    let file1 = temp.path().join("file1.txt");
    let file2 = temp.path().join("subdir").join("file2.txt");
    std::fs::create_dir_all(file2.parent().unwrap()).unwrap();
    std::fs::write(&file1, "a").unwrap();
    std::fs::write(&file2, "b").unwrap();

    let list = vec![
      file1.strip_prefix(temp.path()).unwrap().to_owned(),
      file2.strip_prefix(temp.path()).unwrap().to_owned(),
    ];
    let result =
      list_directory(temp.path(), &DirectoryListing::List(list), false, "/")
        .unwrap();

    assert_eq!(result.len(), 2);
    assert_eq!(
      result[&file1
        .strip_prefix(temp.path())
        .unwrap()
        .to_string_lossy()
        .to_string()],
      std::fs::read(file1).unwrap()
    );
    assert_eq!(
      result[&file2
        .strip_prefix(temp.path())
        .unwrap()
        .to_string_lossy()
        .to_string()],
      std::fs::read(file2).unwrap()
    );
  }

  #[test]
  fn test_list_directory_custom_list_missing() {
    let temp = TempDir::new().unwrap();
    let missing = temp.path().join("ghost.txt");
    let list = vec![missing];

    let result =
      list_directory(temp.path(), &DirectoryListing::List(list), false, "/");
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
    map.insert(
      "alias1".to_string(),
      file1.strip_prefix(temp.path()).unwrap().to_owned(),
    );
    map.insert(
      "alias2".to_string(),
      file2.strip_prefix(temp.path()).unwrap().to_owned(),
    );

    let result = list_directory(
      temp.path(),
      &DirectoryListing::Map(map.clone()),
      false,
      "/",
    )
    .unwrap();

    assert_eq!(
      result,
      HashMap::from([
        ("alias1".to_string(), std::fs::read(file1).unwrap()),
        ("alias2".to_string(), std::fs::read(file2).unwrap())
      ])
    );
  }

  #[test]
  fn test_list_directory_custom_map_missing() {
    let temp = TempDir::new().unwrap();
    let mut map = HashMap::new();
    map.insert("ghost".to_string(), temp.path().join("nonexistent.txt"));

    let result =
      list_directory(temp.path(), &DirectoryListing::Map(map), false, "/");
    assert!(result.is_err());
    assert!(matches!(result, Err(CrylError::DirectoryListing(_))));
  }

  #[test]
  fn test_list_directory_empty_dir() {
    let temp = TempDir::new().unwrap();

    // Flat on empty dir
    let flat =
      list_directory(temp.path(), &DirectoryListing::Flat, false, "/").unwrap();
    assert!(flat.is_empty());

    // Deep on empty dir
    let deep =
      list_directory(temp.path(), &DirectoryListing::Deep, false, "/").unwrap();
    assert!(deep.is_empty());

    // Custom list with no files should error
    let list = &DirectoryListing::List(vec![]);
    let result = list_directory(temp.path(), list, false, "/").unwrap();
    assert!(result.is_empty());

    // Custom map with no files
    let map = &DirectoryListing::Map(HashMap::new());
    let result = list_directory(temp.path(), map, false, "/").unwrap();
    assert!(result.is_empty());
  }

  #[test]
  fn test_list_directory_symlink_in_dir() {
    let temp = TempDir::new().unwrap();
    let target = temp.path().join("target.txt");
    let link = temp.path().join("link.txt");
    std::fs::write(&target, "content").unwrap();
    symbolic_link(&target, &link).unwrap();

    let result =
      list_directory(temp.path(), &DirectoryListing::Flat, false, "/").unwrap();

    // Should include symlink file entry
    assert_eq!(result.len(), 2);
    assert!(
      result.contains_key(
        &link
          .strip_prefix(temp.path())
          .unwrap()
          .to_string_lossy()
          .to_string()
      )
    );
  }

  #[test]
  fn test_list_directory_list_allow_fail() {
    let temp = TempDir::new().unwrap();

    let result = list_directory(
      temp.path(),
      &DirectoryListing::List(vec![PathBuf::from_str("some-file").unwrap()]),
      true,
      "/",
    )
    .unwrap();

    // Should be empty
    assert_eq!(result.len(), 0);
  }

  #[test]
  fn test_list_directory_map_allow_fail() {
    let temp = TempDir::new().unwrap();

    let mut map = HashMap::new();
    map.insert(
      "some-file".to_owned(),
      PathBuf::from_str("some-file").unwrap(),
    );
    let result =
      list_directory(temp.path(), &DirectoryListing::Map(map), true, "/")
        .unwrap();

    // Should be empty
    assert_eq!(result.len(), 0);
  }

  #[test]
  fn test_in_memory_filesystem_yaml() {
    let input: serde_yaml::Value = serde_yaml::from_str(
      r#"
     file1: "file1-bla"
     file2: "file2-bla"
     nested:
       file3: "bla-bla"
   "#,
    )
    .unwrap();

    let filesystem: InMemoryFilesystem = (&input).try_into().unwrap();
    let output: serde_yaml::Value = filesystem.try_into().unwrap();

    assert_eq!(output, input);
  }

  #[test]
  fn test_in_memory_filesystem_json() {
    let input: serde_json::Value = serde_json::from_str(
      r#"
      {
        "file1": "file1-bla",
        "file2": "file2-bla",
        "nested": {
          "file3": "bla-bla"
        }
      }
   "#,
    )
    .unwrap();

    let filesystem: InMemoryFilesystem = (&input).try_into().unwrap();
    let output: serde_json::Value = filesystem.try_into().unwrap();

    assert_eq!(output, input);
  }

  #[test]
  fn test_in_memory_filesystem_toml() {
    let input: toml::Value = toml::from_str(
      r#"
      file1 = "file1-bla"
      file2 = "file2-bla"

      [nested]
      file3 = "bla-bla"
     "#,
    )
    .unwrap();

    let filesystem: InMemoryFilesystem = (&input).try_into().unwrap();
    let output: toml::Value = filesystem.try_into().unwrap();

    assert_eq!(output, input);
  }
}
