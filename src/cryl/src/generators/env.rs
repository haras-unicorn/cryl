use std::path::Path;

use itertools::Itertools;

use crate::common::{
  CrylResult, DirectoryListing, Format, deserialize_from_file, list_directory,
  save_atomic,
};

/// Generate an environment (.env-style) file from key-value pairs
///
/// # Arguments
/// * `name` - Path to save the environment file
/// * `format` - Input format of listing
/// * `listing` - Path to file containing key-value pairs
/// * `renew` - Overwrite destination if it exists
///
/// # Description
/// Reads variables from the specified listing. Escapes backslashes, newlines,
/// and double quotes in values, then outputs as KEY="value" format.
pub fn generate_env(
  name: &Path,
  format: Format,
  listing: &Path,
  renew: bool,
) -> CrylResult<()> {
  // Read and deserialize the listing file
  let listing: DirectoryListing = deserialize_from_file(listing, Some(format))?;

  // Keep in variable here so we can consume it with list_directory
  let is_map = matches!(listing, DirectoryListing::Map(_));

  // Process each variable
  let mut lines: Vec<String> = Vec::new();
  for (key, value) in list_directory(std::env::current_dir()?, listing)? {
    // Convert paths to screaming snake case case
    let key = if !is_map {
      Path::new(&key)
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_uppercase())
        .join("_")
    } else {
      key
    };

    // Check if value is a file path and read it if so
    let raw_value = std::fs::read_to_string(&value)?;

    // Escape special characters: backslash, newline, double quote
    let escaped = raw_value
      .replace('\\', "\\\\")
      .replace('\n', "\\n")
      .replace('\r', "\\r")
      .replace('"', "\\\"");

    lines.push(format!("{}=\"{}\"", key, escaped));
  }

  // Join lines with newlines
  let output = lines.join("\n");

  // Save the file (private permissions since it may contain secrets)
  save_atomic(name, output.as_bytes(), renew, false)?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use serial_test::serial;

  use super::*;
  use crate::common::TempCurrentDir;
  use std::fs;
  use std::os::unix::fs::PermissionsExt;
  use std::path::PathBuf;
  use std::str::FromStr;

  #[test]
  #[serial(working_directory)]
  fn test_generate_env_basic() {
    let _temp = TempCurrentDir::new().unwrap();
    let vars_path = PathBuf::from_str("vars.json").unwrap();
    let env_path = PathBuf::from_str(".env").unwrap();

    // Create variables file
    fs::write("key1", "value1").unwrap();
    fs::write("key2", "value2").unwrap();
    fs::write(
      &vars_path,
      r#"{ "type": "map", "value": { "KEY1": "key1", "KEY2": "key2" } }"#,
    )
    .unwrap();

    generate_env(&env_path, Format::Json, &vars_path, false).unwrap();

    assert!(env_path.exists());
    let content = fs::read_to_string(&env_path).unwrap();
    assert!(content.contains("KEY1=\"value1\""));
    assert!(content.contains("KEY2=\"value2\""));
  }

  #[test]
  #[serial(working_directory)]
  fn test_generate_env_missing_file_fails() {
    let _temp = TempCurrentDir::new().unwrap();
    let vars_path = PathBuf::from_str("vars.json").unwrap();
    let env_path = PathBuf::from_str(".env").unwrap();

    // Create a secret file
    fs::write(
      &vars_path,
      r#"{ "type": "map", "value": { "PASSWORD": "password" } }"#,
    )
    .unwrap();

    let result = generate_env(&env_path, Format::Json, &vars_path, false);
    assert!(matches!(result, Err(_)));
  }

  #[test]
  #[serial(working_directory)]
  fn test_generate_env_escapes_special_chars() {
    let _temp = TempCurrentDir::new().unwrap();
    let vars_path = PathBuf::from_str("vars.json").unwrap();
    let env_path = PathBuf::from_str(".env").unwrap();

    // Create variables with special characters (using \\n for newline in JSON)
    fs::write("path", "/home/user/docs").unwrap();
    fs::write("msg", "Hello\nWorld").unwrap();
    fs::write("quote", "say \"hi\"").unwrap();
    fs::write(
      &vars_path,
      r#"{ "type": "map", "value": { "PATH": "path", "MSG": "msg", "QUOTE": "quote" } }"#,
    )
    .unwrap();

    generate_env(&env_path, Format::Json, &vars_path, false).unwrap();

    let content = fs::read_to_string(&env_path).unwrap();
    assert!(content.contains("PATH=\"/home/user/docs\""));
    assert!(content.contains("MSG=\"Hello\\nWorld\""));
    assert!(content.contains("QUOTE=\"say \\\"hi\\\"\""));
  }

  #[test]
  #[serial(working_directory)]
  fn test_generate_env_does_not_trim_whitespace() {
    let _temp = TempCurrentDir::new().unwrap();
    let vars_path = PathBuf::from_str("vars.json").unwrap();
    let env_path = PathBuf::from_str(".env").unwrap();

    // Create variables with whitespace
    fs::write("key", "  value with spaces  ").unwrap();
    fs::write(
      &vars_path,
      r#"{ "type": "map", "value": { "KEY": "key" } }"#,
    )
    .unwrap();

    generate_env(&env_path, Format::Json, &vars_path, false).unwrap();

    let content = fs::read_to_string(&env_path).unwrap();
    assert!(content.contains("KEY=\"  value with spaces  \""));
  }

  #[test]
  #[serial(working_directory)]
  fn test_generate_env_renew_false_no_overwrite() {
    let _temp = TempCurrentDir::new().unwrap();
    let vars_path = PathBuf::from_str("vars.json").unwrap();
    let env_path = PathBuf::from_str(".env").unwrap();

    fs::write("key", "new").unwrap();
    fs::write(
      &vars_path,
      r#"{ "type": "map", "value": { "KEY": "key" } }"#,
    )
    .unwrap();
    fs::write(&env_path, "KEY=\"old\"").unwrap();

    generate_env(&env_path, Format::Json, &vars_path, false).unwrap();

    let content = fs::read_to_string(&env_path).unwrap();
    assert_eq!(content, "KEY=\"old\"");
  }

  #[test]
  #[serial(working_directory)]
  fn test_generate_env_renew_true_overwrites() {
    let _temp = TempCurrentDir::new().unwrap();
    let vars_path = PathBuf::from_str("vars.json").unwrap();
    let env_path = PathBuf::from_str(".env").unwrap();

    fs::write("key", "new").unwrap();
    fs::write(
      &vars_path,
      r#"{ "type": "map", "value": { "KEY": "key" } }"#,
    )
    .unwrap();
    fs::write(&env_path, "KEY=\"old\"").unwrap();

    generate_env(&env_path, Format::Json, &vars_path, true).unwrap();

    let content = fs::read_to_string(&env_path).unwrap();
    assert_eq!(content, "KEY=\"new\"");
  }

  #[test]
  #[serial(working_directory)]
  fn test_generate_env_private_permissions() {
    let _temp = TempCurrentDir::new().unwrap();
    let vars_path = PathBuf::from_str("vars.json").unwrap();
    let env_path = PathBuf::from_str(".env").unwrap();

    fs::write("key", "value").unwrap();
    fs::write(&vars_path, r#"{ "type": "map", "value": {"KEY": "key"} }"#)
      .unwrap();

    generate_env(&env_path, Format::Json, &vars_path, false).unwrap();

    let metadata = fs::metadata(&env_path).unwrap();
    let perms = metadata.permissions();
    assert_eq!(perms.mode() & 0o777, 0o600);
  }

  #[test]
  #[serial(working_directory)]
  fn test_generate_env_yaml_format() {
    let _temp = TempCurrentDir::new().unwrap();
    let vars_path = PathBuf::from_str("vars.yaml").unwrap();
    let env_path = PathBuf::from_str(".env").unwrap();

    fs::write("key1", "value1").unwrap();
    fs::write("key2", "value2").unwrap();
    fs::write(
      &vars_path,
      r#"
        type: map
        value:
          KEY1: key1
          KEY2: key2
    "#,
    )
    .unwrap();

    generate_env(&env_path, Format::Yaml, &vars_path, false).unwrap();

    let content = fs::read_to_string(&env_path).unwrap();
    assert!(content.contains("KEY1=\"value1\""));
    assert!(content.contains("KEY2=\"value2\""));
  }

  #[test]
  #[serial(working_directory)]
  fn test_generate_env_toml_format() {
    let _temp = TempCurrentDir::new().unwrap();
    let vars_path = PathBuf::from_str("vars.toml").unwrap();
    let env_path = PathBuf::from_str(".env").unwrap();

    fs::write("key1", "value1").unwrap();
    fs::write("key2", "value2").unwrap();
    fs::write(
      &vars_path,
      r#"
        type = "map"
        [value]
        KEY1 = "key1"
        KEY2 = "key2"
    "#,
    )
    .unwrap();

    generate_env(&env_path, Format::Toml, &vars_path, false).unwrap();

    let content = fs::read_to_string(&env_path).unwrap();
    assert!(content.contains("KEY1=\"value1\""));
    assert!(content.contains("KEY2=\"value2\""));
  }

  #[test]
  #[serial(working_directory)]
  fn test_generate_env_empty_variables() {
    let _temp = TempCurrentDir::new().unwrap();
    let vars_path = PathBuf::from_str("vars.json").unwrap();
    let env_path = PathBuf::from_str(".env").unwrap();

    fs::write(&vars_path, r#"{ "type": "map", "value": { } }"#).unwrap();

    generate_env(&env_path, Format::Json, &vars_path, false).unwrap();

    let content = fs::read_to_string(&env_path).unwrap();
    assert_eq!(content, "");
  }

  #[test]
  #[serial(working_directory)]
  fn test_generate_env_handles_carriage_return() {
    let _temp = TempCurrentDir::new().unwrap();
    let vars_path = PathBuf::from_str("vars.json").unwrap();
    let env_path = PathBuf::from_str(".env").unwrap();

    fs::write("text", "line1\r\nline2").unwrap();
    fs::write(
      &vars_path,
      r#"{ "type": "map", "value": { "TEXT": "text" } }"#,
    )
    .unwrap();

    generate_env(&env_path, Format::Json, &vars_path, false).unwrap();

    let content = fs::read_to_string(&env_path).unwrap();
    assert!(content.contains("TEXT=\"line1\\r\\nline2\""));
  }

  #[test]
  #[serial(working_directory)]
  fn test_generate_env_vars_subdir() {
    let _temp = TempCurrentDir::new().unwrap();
    let vars_path = PathBuf::from_str("subdir").unwrap().join("vars.json");
    let env_path = PathBuf::from_str(".env").unwrap();

    // Create variables file
    fs::create_dir_all(vars_path.parent().unwrap()).unwrap();
    fs::write("./subdir/key1", "value1").unwrap();
    fs::write("key2", "value2").unwrap();
    fs::write(&vars_path, r#"{ "type": "map", "value": { "KEY1": "./subdir/key1", "KEY2": "key2" } }"#).unwrap();

    generate_env(&env_path, Format::Json, &vars_path, false).unwrap();

    assert!(env_path.exists());
    let content = fs::read_to_string(&env_path).unwrap();
    assert!(content.contains("KEY1=\"value1\""));
    assert!(content.contains("KEY2=\"value2\""));
  }

  #[test]
  #[serial(working_directory)]
  fn test_generate_env_env_subdir() {
    let _temp = TempCurrentDir::new().unwrap();
    let vars_path = PathBuf::from_str("vars.json").unwrap();
    let env_path = PathBuf::from_str("subdir").unwrap().join(".env");

    // Create variables file
    fs::create_dir_all("subdir2").unwrap();
    fs::write("./subdir2/key1", "value1").unwrap();
    fs::write("key2", "value2").unwrap();
    fs::write(&vars_path, r#"{ "type": "map", "value": { "KEY1": "./subdir2/key1", "KEY2": "key2" } }"#).unwrap();

    generate_env(&env_path, Format::Json, &vars_path, false).unwrap();

    assert!(env_path.exists());
    let content = fs::read_to_string(&env_path).unwrap();
    assert!(content.contains("KEY1=\"value1\""));
    assert!(content.contains("KEY2=\"value2\""));
  }
}
