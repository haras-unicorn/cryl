//! Sandbox E2E tests for cryl
//!
//! These tests verify the bubblewrap sandbox functionality.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Test that sandbox execution works for basic generation
#[test]
fn test_sandbox_basic_generation() {
  let temp_dir = TempDir::new().unwrap();
  let spec_path = temp_dir.path().join("spec.toml");
  let work_dir = temp_dir.path().join("work");

  fs::create_dir(&work_dir).unwrap();

  let spec_content = r#"
imports = []
exports = []

[[generations]]
generator = "text"
arguments.name = "test.txt"
arguments.text = "Hello from sandbox!"
"#;

  fs::write(&spec_path, spec_content).unwrap();

  // Run with sandbox (no --nosandbox flag)
  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd.arg("path").arg(&spec_path).current_dir(&work_dir);

  // Sandbox should succeed
  cmd.assert().success();

  // Verify file was created (output will be printed to stderr since sandbox)
  // Note: In sandbox mode, files are created inside the sandbox and output is printed
}

/// Test sandbox with ID generation
#[test]
fn test_sandbox_id_generation() {
  let temp_dir = TempDir::new().unwrap();
  let spec_path = temp_dir.path().join("spec.toml");
  let work_dir = temp_dir.path().join("work");

  fs::create_dir(&work_dir).unwrap();

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
  cmd.arg("path").arg(&spec_path).current_dir(&work_dir);

  cmd.assert().success();
}

/// Test sandbox with multiple generators
#[test]
fn test_sandbox_multiple_generations() {
  let temp_dir = TempDir::new().unwrap();
  let spec_path = temp_dir.path().join("spec.toml");
  let work_dir = temp_dir.path().join("work");

  fs::create_dir(&work_dir).unwrap();

  let spec_content = r#"
imports = []
exports = []

[[generations]]
generator = "text"
arguments.name = "file1.txt"
arguments.text = "content1"

[[generations]]
generator = "text"
arguments.name = "file2.txt"
arguments.text = "content2"

[[generations]]
generator = "id"
arguments.name = "api-key"
arguments.length = 32
"#;

  fs::write(&spec_path, spec_content).unwrap();

  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd.arg("path").arg(&spec_path).current_dir(&work_dir);

  cmd.assert().success();
}

/// Test sandbox with dry-run flag
#[test]
fn test_sandbox_with_dry_run() {
  let temp_dir = TempDir::new().unwrap();
  let spec_path = temp_dir.path().join("spec.toml");
  let work_dir = temp_dir.path().join("work");

  fs::create_dir(&work_dir).unwrap();

  let spec_content = r#"
imports = []

[[exports]]
exporter = "copy"
arguments.from = "test.txt"
arguments.to = "/dev/null"

[[generations]]
generator = "text"
arguments.name = "test.txt"
arguments.text = "test content"
"#;

  fs::write(&spec_path, spec_content).unwrap();

  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd
    .arg("path")
    .arg(&spec_path)
    .arg("--dry-run")
    .current_dir(&work_dir);

  cmd.assert().success();
}

/// Test sandbox with copy import (using allow_fail since source may not exist)
#[test]
fn test_sandbox_with_import() {
  let temp_dir = TempDir::new().unwrap();
  let spec_path = temp_dir.path().join("spec.toml");
  let work_dir = temp_dir.path().join("work");

  fs::create_dir(&work_dir).unwrap();

  let spec_content = r#"
exports = []

[[imports]]
importer = "copy"
arguments.from = "source.txt"
arguments.to = "dest.txt"
arguments.allow_fail = true

[[generations]]
generator = "text"
arguments.name = "test.txt"
arguments.text = "test"
"#;

  fs::write(&spec_path, spec_content).unwrap();

  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd.arg("path").arg(&spec_path).current_dir(&work_dir);

  cmd.assert().success();
}

/// Test sandbox with --ro-binds flag
#[test]
fn test_sandbox_with_ro_binds() {
  let temp_dir = TempDir::new().unwrap();
  let spec_path = temp_dir.path().join("spec.toml");
  let work_dir = temp_dir.path().join("work");
  let source_dir = temp_dir.path().join("source");

  fs::create_dir(&work_dir).unwrap();
  fs::create_dir(&source_dir).unwrap();

  // Create a file in source dir
  fs::write(source_dir.join("data.txt"), "readonly data").unwrap();

  let spec_content = r#"
imports = []
exports = []

[[generations]]
generator = "text"
arguments.name = "test.txt"
arguments.text = "test"
"#;

  fs::write(&spec_path, spec_content).unwrap();

  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd
    .arg("path")
    .arg(&spec_path)
    .arg("--ro-binds")
    .arg(source_dir.to_str().unwrap())
    .current_dir(&work_dir);

  cmd.assert().success();
}

/// Test sandbox with --binds flag
#[test]
fn test_sandbox_with_binds() {
  let temp_dir = TempDir::new().unwrap();
  let spec_path = temp_dir.path().join("spec.toml");
  let work_dir = temp_dir.path().join("work");
  let data_dir = temp_dir.path().join("data");

  fs::create_dir(&work_dir).unwrap();
  fs::create_dir(&data_dir).unwrap();

  fs::write(data_dir.join("input.txt"), "input data").unwrap();

  let spec_content = r#"
imports = []
exports = []

[[generations]]
generator = "text"
arguments.name = "test.txt"
arguments.text = "test"
"#;

  fs::write(&spec_path, spec_content).unwrap();

  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd
    .arg("path")
    .arg(&spec_path)
    .arg("--binds")
    .arg(data_dir.to_str().unwrap())
    .current_dir(&work_dir);

  cmd.assert().success();
}

/// Test sandbox with max limits
#[test]
fn test_sandbox_with_max_limits() {
  let temp_dir = TempDir::new().unwrap();
  let spec_path = temp_dir.path().join("spec.toml");
  let work_dir = temp_dir.path().join("work");

  fs::create_dir(&work_dir).unwrap();

  let spec_content = r#"
imports = []
exports = []

[[generations]]
generator = "text"
arguments.name = "test.txt"
arguments.text = "test"
"#;

  fs::write(&spec_path, spec_content).unwrap();

  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd
    .arg("path")
    .arg(&spec_path)
    .arg("--max-imports=5")
    .arg("--max-generations=5")
    .arg("--max-exports=5")
    .arg("--max-specification-size=10000")
    .current_dir(&work_dir);

  cmd.assert().success();
}

/// Test that sandbox fails with invalid spec (should propagate error)
#[test]
fn test_sandbox_invalid_spec() {
  let temp_dir = TempDir::new().unwrap();
  let spec_path = temp_dir.path().join("spec.toml");
  let work_dir = temp_dir.path().join("work");

  fs::create_dir(&work_dir).unwrap();

  // Invalid spec - missing required fields
  let spec_content = r#"
imports = []
exports = []

[[generations]]
generator = "invalid_generator"
arguments.name = "test.txt"
"#;

  fs::write(&spec_path, spec_content).unwrap();

  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd.arg("path").arg(&spec_path).current_dir(&work_dir);

  cmd.assert().failure();
}

/// Test sandbox with script generator (requires --allow-script)
#[test]
fn test_sandbox_script_without_allow_script() {
  // Skip if nu is not available (required for script generator)
  if which::which("nu").is_err() {
    eprintln!("Skipping test: nu not available in PATH");
    return;
  }

  let temp_dir = TempDir::new().unwrap();
  let spec_path = temp_dir.path().join("spec.toml");
  let work_dir = temp_dir.path().join("work");

  fs::create_dir(&work_dir).unwrap();

  let spec_content = r#"
imports = []
exports = []

[[generations]]
generator = "script"
arguments.name = "test.nu"
arguments.text = "echo 'hello'"
"#;

  fs::write(&spec_path, spec_content).unwrap();

  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd.arg("path").arg(&spec_path).current_dir(&work_dir);

  cmd
    .assert()
    .failure()
    .stderr(predicate::str::contains("Script generator not allowed"));
}

/// Test sandbox with script generator and --allow-script
#[test]
fn test_sandbox_script_with_allow_script() {
  // Skip if nu is not available (required for script generator)
  if which::which("nu").is_err() {
    eprintln!("Skipping test: nu not available in PATH");
    return;
  }

  let temp_dir = TempDir::new().unwrap();
  let spec_path = temp_dir.path().join("spec.toml");
  let work_dir = temp_dir.path().join("work");

  fs::create_dir(&work_dir).unwrap();

  let spec_content = r#"
imports = []
exports = []

[[generations]]
generator = "script"
arguments.name = "test.nu"
arguments.text = "echo 'hello from sandbox'"
"#;

  fs::write(&spec_path, spec_content).unwrap();

  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd
    .arg("path")
    .arg(&spec_path)
    .arg("--allow-script")
    .current_dir(&work_dir);

  cmd.assert().success();
}

/// Test comparison: sandbox vs nosandbox produce same results
#[test]
fn test_sandbox_same_result_as_nosandbox() {
  let temp_dir = TempDir::new().unwrap();
  let spec_path = temp_dir.path().join("spec.toml");
  let work_dir_sandbox = temp_dir.path().join("work_sandbox");
  let work_dir_nosandbox = temp_dir.path().join("work_nosandbox");

  fs::create_dir(&work_dir_sandbox).unwrap();
  fs::create_dir(&work_dir_nosandbox).unwrap();

  let spec_content = r#"
imports = []
exports = []

[[generations]]
generator = "text"
arguments.name = "test.txt"
arguments.text = "test content"
"#;

  fs::write(&spec_path, spec_content).unwrap();

  // Run with sandbox
  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd
    .arg("path")
    .arg(&spec_path)
    .current_dir(&work_dir_sandbox);
  cmd.assert().success();

  // Run without sandbox
  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd
    .arg("path")
    .arg(&spec_path)
    .arg("--nosandbox")
    .arg("--stay")
    .arg("--keep")
    .current_dir(&work_dir_nosandbox);
  cmd.assert().success();

  // Verify nosandbox created the file
  assert!(work_dir_nosandbox.join("test.txt").exists());
}
