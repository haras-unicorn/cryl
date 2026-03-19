use std::path::Path;
use std::process::Command;

use crate::common::{CrylError, CrylResult, save_atomic};

/// Generate and run a Nushell script
///
/// # Arguments
/// * `name` - Path to save the script
/// * `text` - Script contents
/// * `renew` - Overwrite destination if it exists
///
/// # Description
/// Saves the script text to the specified path and then executes it
/// using the system `nu` command (Nushell from PATH).
pub fn generate_script(name: &Path, text: &str, renew: bool) -> CrylResult<()> {
  // If renew is false and file exists, skip both saving and execution
  if !renew && name.exists() {
    return Ok(());
  }

  // Save the script file with private permissions (may contain secrets)
  save_atomic(name, text.as_bytes(), renew, false)?;

  // Execute the script using nushell from PATH
  let output = Command::new("nu").arg(name).output()?;

  if !output.status.success() {
    return Err(CrylError::ToolExecution {
      tool: "nu".to_string(),
      exit_code: output.status.code().unwrap_or(-1),
      stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    });
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;
  use std::os::unix::fs::PermissionsExt;
  use tempfile::TempDir;

  #[test]
  fn test_generate_script_creates_and_executes() {
    let temp = TempDir::new().unwrap();
    let script_path = temp.path().join("test_script.nu");
    let output_path = temp.path().join("output.txt");

    // Create a script that writes to a file
    let script_content = format!(
      r#""Hello, from nu script!" | save {}"#,
      output_path.display()
    );

    generate_script(&script_path, &script_content, false).unwrap();

    // Check script file exists
    assert!(script_path.exists());

    // Check the script was executed (output file should exist)
    assert!(output_path.exists());
    let content = fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("Hello, from nu script!"));
  }

  #[test]
  fn test_generate_script_saves_content() {
    let temp = TempDir::new().unwrap();
    let script_path = temp.path().join("script.nu");

    let script_content = r#"# This is a test script
let x = 42
$x"#;

    generate_script(&script_path, script_content, false).unwrap();

    let saved_content = fs::read_to_string(&script_path).unwrap();
    assert_eq!(saved_content, script_content);
  }

  #[test]
  fn test_generate_script_private_permissions() {
    let temp = TempDir::new().unwrap();
    let script_path = temp.path().join("script.nu");

    let script_content = r#""test""#;

    generate_script(&script_path, script_content, false).unwrap();

    let metadata = fs::metadata(&script_path).unwrap();
    let perms = metadata.permissions();
    assert_eq!(perms.mode() & 0o777, 0o600);
  }

  #[test]
  fn test_generate_script_renew_false_no_overwrite() {
    let temp = TempDir::new().unwrap();
    let script_path = temp.path().join("script.nu");

    fs::write(&script_path, "Original script").unwrap();

    generate_script(&script_path, "New script", false).unwrap();

    let content = fs::read_to_string(&script_path).unwrap();
    assert_eq!(content, "Original script");
  }

  #[test]
  fn test_generate_script_renew_true_overwrites() {
    let temp = TempDir::new().unwrap();
    let script_path = temp.path().join("script.nu");
    let output_path = temp.path().join("output.txt");

    fs::write(&script_path, "Original script").unwrap();

    let script_content =
      format!(r#""New output" | save {}"#, output_path.display());

    generate_script(&script_path, &script_content, true).unwrap();

    let content = fs::read_to_string(&script_path).unwrap();
    assert_eq!(content, script_content);

    // Check it was executed
    assert!(output_path.exists());
    let output = fs::read_to_string(&output_path).unwrap();
    assert!(output.contains("New output"));
  }

  #[test]
  fn test_generate_script_unicode_content() {
    let temp = TempDir::new().unwrap();
    let script_path = temp.path().join("unicode.nu");

    // cspell:disable-next-line
    let script_content = r#""Hello 世界! 🌍 émojis""#;

    generate_script(&script_path, script_content, false).unwrap();

    let saved_content = fs::read_to_string(&script_path).unwrap();
    assert_eq!(saved_content, script_content);
  }

  #[test]
  fn test_generate_script_multiline() {
    let temp = TempDir::new().unwrap();
    let script_path = temp.path().join("multiline.nu");

    let script_content = r#"def greet [name] {
  $"Hello, ($name)!"
}

greet "World""#;

    generate_script(&script_path, script_content, false).unwrap();

    let saved_content = fs::read_to_string(&script_path).unwrap();
    assert_eq!(saved_content, script_content);
  }

  #[test]
  fn test_generate_script_empty_content() {
    let temp = TempDir::new().unwrap();
    let script_path = temp.path().join("empty.nu");

    // Empty script should still execute successfully
    generate_script(&script_path, "", false).unwrap();

    assert!(script_path.exists());
    let content = fs::read_to_string(&script_path).unwrap();
    assert_eq!(content, "");
  }

  #[test]
  fn test_generate_script_handles_variables() {
    let temp = TempDir::new().unwrap();
    let script_path = temp.path().join("vars.nu");
    let output_path = temp.path().join("vars_output.txt");

    let script_content = format!(
      r#"let name = "Alice"
let age = 30
$"Name: ($name), Age: ($age)" | save {}"#,
      output_path.display()
    );

    generate_script(&script_path, &script_content, false).unwrap();

    assert!(output_path.exists());
    let content = fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("Name: Alice"));
    assert!(content.contains("Age: 30"));
  }
}
