//! Basic E2E tests for cryl
//!
//! These tests run the cryl binary with simple specifications
//! to verify the full import -> generate -> export pipeline works.

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// Test a simple generation-only spec (no imports/exports)
#[test]
fn test_simple_id_generation() {
  let temp_dir = TempDir::new().unwrap();
  let spec_path = temp_dir.path().join("spec.toml");
  let work_dir = temp_dir.path().join("work");
  fs::create_dir(&work_dir).unwrap();

  // Simple spec: just generate an ID
  // NOTE: TOML requires exports to be defined before generations table
  let spec_content = r#"
imports = []
exports = []

[[generations]]
generator = "id"
arguments.name = "my-id"
arguments.length = 16
"#;

  fs::write(&spec_path, spec_content).unwrap();

  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd
    .arg("path")
    .arg(&spec_path)
    .arg("--nosandbox")
    .arg("--stay")
    .arg("--keep")
    .current_dir(&work_dir);

  cmd.assert().success();

  // Verify the ID file was created
  let id_path = work_dir.join("my-id");
  assert!(id_path.exists(), "ID file should be created");

  let id_content = fs::read_to_string(&id_path).unwrap();
  assert_eq!(id_content.len(), 16, "ID should be 16 characters");
  assert!(
    id_content.chars().all(|c| c.is_ascii_alphanumeric()),
    "ID should be alphanumeric"
  );
}

/// Test text generation
#[test]
fn test_text_generation() {
  let temp_dir = TempDir::new().unwrap();
  let spec_path = temp_dir.path().join("spec.toml");
  let work_dir = temp_dir.path().join("work");
  fs::create_dir(&work_dir).unwrap();

  // NOTE: TOML requires exports to be defined before generations table
  let spec_content = r#"
imports = []
exports = []

[[generations]]
generator = "text"
arguments.name = "hello.txt"
arguments.text = "Hello, World!"
"#;

  fs::write(&spec_path, spec_content).unwrap();

  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd
    .arg("path")
    .arg(&spec_path)
    .arg("--nosandbox")
    .arg("--stay")
    .arg("--keep")
    .current_dir(&work_dir);

  cmd.assert().success();

  let text_path = work_dir.join("hello.txt");
  assert!(text_path.exists(), "Text file should be created");

  let content = fs::read_to_string(&text_path).unwrap();
  assert_eq!(content, "Hello, World!");
}

/// Test JSON data generation
#[test]
fn test_json_generation() {
  let temp_dir = TempDir::new().unwrap();
  let spec_path = temp_dir.path().join("spec.toml");
  let work_dir = temp_dir.path().join("work");
  fs::create_dir(&work_dir).unwrap();

  // NOTE: TOML requires exports to be defined before generations table
  let spec_content = r#"
imports = []
exports = []

[[generations]]
generator = "json"
arguments.name = "config.json"
arguments.value = { name = "test", version = "1.0" }
"#;

  fs::write(&spec_path, spec_content).unwrap();

  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd
    .arg("path")
    .arg(&spec_path)
    .arg("--nosandbox")
    .arg("--stay")
    .arg("--keep")
    .current_dir(&work_dir);

  cmd.assert().success();

  let json_path = work_dir.join("config.json");
  assert!(json_path.exists(), "JSON file should be created");

  let content = fs::read_to_string(&json_path).unwrap();
  let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
  assert_eq!(parsed["name"], "test");
  assert_eq!(parsed["version"], "1.0");
}

/// Test copy generation
#[test]
fn test_copy_generation() {
  let temp_dir = TempDir::new().unwrap();
  let spec_path = temp_dir.path().join("spec.toml");
  let work_dir = temp_dir.path().join("work");
  fs::create_dir(&work_dir).unwrap();

  // Create a source file to copy
  let source_path = work_dir.join("source.txt");
  fs::write(&source_path, "source content").unwrap();

  // NOTE: TOML requires exports to be defined before generations table
  let spec_content = r#"
imports = []
exports = []

[[generations]]
generator = "copy"
arguments.from = "source.txt"
arguments.to = "destination.txt"
"#;

  fs::write(&spec_path, spec_content).unwrap();

  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd
    .arg("path")
    .arg(&spec_path)
    .arg("--nosandbox")
    .arg("--stay")
    .arg("--keep")
    .current_dir(&work_dir);

  cmd.assert().success();

  let dest_path = work_dir.join("destination.txt");
  assert!(dest_path.exists(), "Destination file should be created");

  let content = fs::read_to_string(&dest_path).unwrap();
  assert_eq!(content, "source content");
}

/// Test full pipeline with copy import and export
#[test]
fn test_full_pipeline_with_copy() {
  let temp_dir = TempDir::new().unwrap();
  let spec_path = temp_dir.path().join("spec.toml");
  let work_dir = temp_dir.path().join("work");
  let import_source = temp_dir.path().join("import.txt");
  let export_dest = temp_dir.path().join("export.txt");

  fs::create_dir(&work_dir).unwrap();
  fs::write(&import_source, "imported data").unwrap();

  // Full spec: import -> generate -> export
  // NOTE: TOML requires exports to be defined before generations table
  let spec_content = format!(
    r#"
[[imports]]
importer = "copy"
arguments.from = "{}"
arguments.to = "imported.txt"

exports = []

[[generations]]
generator = "text"
arguments.name = "generated.txt"
arguments.text = "generated data"

[[exports]]
exporter = "copy"
arguments.from = "generated.txt"
arguments.to = "{}"
"#,
    import_source.display(),
    export_dest.display()
  );

  fs::write(&spec_path, spec_content).unwrap();

  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd
    .arg("path")
    .arg(&spec_path)
    .arg("--nosandbox")
    .arg("--stay")
    .arg("--keep")
    .current_dir(&work_dir);

  cmd.assert().success();

  // Verify import worked
  let imported_path = work_dir.join("imported.txt");
  assert!(imported_path.exists(), "Imported file should exist");
  assert_eq!(fs::read_to_string(&imported_path).unwrap(), "imported data");

  // Verify generation worked
  let generated_path = work_dir.join("generated.txt");
  assert!(generated_path.exists(), "Generated file should exist");
  assert_eq!(
    fs::read_to_string(&generated_path).unwrap(),
    "generated data"
  );

  // Verify export worked
  assert!(export_dest.exists(), "Export destination should exist");
  assert_eq!(fs::read_to_string(&export_dest).unwrap(), "generated data");
}
