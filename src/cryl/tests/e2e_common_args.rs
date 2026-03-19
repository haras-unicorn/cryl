//! Common arguments E2E tests for cryl
//!
//! These tests verify the behavior of common CLI arguments like
//! --dry-run, --allow-script, and --max-* limits.

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// Test dry-run flag prevents exports
#[test]
fn test_dry_run_prevents_exports() {
  let temp_dir = TempDir::new().unwrap();
  let spec_path = temp_dir.path().join("spec.toml");
  let work_dir = temp_dir.path().join("work");
  let export_dest = temp_dir.path().join("export.txt");

  fs::create_dir(&work_dir).unwrap();

  // NOTE: TOML requires exports to be defined before generations table
  let spec_content = format!(
    r#"
imports = []

[[exports]]
exporter = "copy"
arguments.from = "generated.txt"
arguments.to = "{}"

[[generations]]
generator = "text"
arguments.name = "generated.txt"
arguments.text = "generated data"
"#,
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
    .arg("--dry-run")
    .current_dir(&work_dir);

  cmd.assert().success();

  // Generation should still happen
  let generated_path = work_dir.join("generated.txt");
  assert!(generated_path.exists(), "Generated file should exist");

  // But export should NOT happen
  assert!(
    !export_dest.exists(),
    "Export file should NOT exist in dry-run"
  );
}

/// Test --max-imports limit enforcement
#[test]
fn test_max_imports_limit() {
  let temp_dir = TempDir::new().unwrap();
  let spec_path = temp_dir.path().join("spec.toml");
  let work_dir = temp_dir.path().join("work");

  fs::create_dir(&work_dir).unwrap();

  // Create a spec with 3 imports
  let spec_content = r#"
exports = []

[[imports]]
importer = "copy"
arguments.from = "dummy1.txt"
arguments.to = "import1.txt"
arguments.allow_fail = true

[[imports]]
importer = "copy"
arguments.from = "dummy2.txt"
arguments.to = "import2.txt"
arguments.allow_fail = true

[[imports]]
importer = "copy"
arguments.from = "dummy3.txt"
arguments.to = "import3.txt"
arguments.allow_fail = true

[[generations]]
generator = "text"
arguments.name = "test.txt"
arguments.text = "test"
"#;

  fs::write(&spec_path, spec_content).unwrap();

  // Run with --max-imports=2 (should fail)
  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd
    .arg("path")
    .arg(&spec_path)
    .arg("--nosandbox")
    .arg("--stay")
    .arg("--keep")
    .arg("--max-imports=2")
    .current_dir(&work_dir);

  cmd.assert().failure().stderr(predicates::str::contains(
    "Import count (3) exceeds maximum allowed (2)",
  ));

  // Run with --max-imports=3 (should succeed)
  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd
    .arg("path")
    .arg(&spec_path)
    .arg("--nosandbox")
    .arg("--stay")
    .arg("--keep")
    .arg("--max-imports=3")
    .current_dir(&work_dir);

  cmd.assert().success();
}

/// Test --max-generations limit enforcement
#[test]
fn test_max_generations_limit() {
  let temp_dir = TempDir::new().unwrap();
  let spec_path = temp_dir.path().join("spec.toml");
  let work_dir = temp_dir.path().join("work");

  fs::create_dir(&work_dir).unwrap();

  // Create a spec with 3 generations
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
generator = "text"
arguments.name = "file3.txt"
arguments.text = "content3"
"#;

  fs::write(&spec_path, spec_content).unwrap();

  // Run with --max-generations=2 (should fail)
  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd
    .arg("path")
    .arg(&spec_path)
    .arg("--nosandbox")
    .arg("--stay")
    .arg("--keep")
    .arg("--max-generations=2")
    .current_dir(&work_dir);

  cmd.assert().failure().stderr(predicates::str::contains(
    "Generation count (3) exceeds maximum allowed (2)",
  ));

  // Run with --max-generations=3 (should succeed)
  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd
    .arg("path")
    .arg(&spec_path)
    .arg("--nosandbox")
    .arg("--stay")
    .arg("--keep")
    .arg("--max-generations=3")
    .current_dir(&work_dir);

  cmd.assert().success();
}

/// Test --max-exports limit enforcement
#[test]
fn test_max_exports_limit() {
  let temp_dir = TempDir::new().unwrap();
  let spec_path = temp_dir.path().join("spec.toml");
  let work_dir = temp_dir.path().join("work");

  fs::create_dir(&work_dir).unwrap();

  // Create a spec with 3 exports
  let spec_content = r#"
imports = []

[[exports]]
exporter = "copy"
arguments.from = "file1.txt"
arguments.to = "/dev/null"

[[exports]]
exporter = "copy"
arguments.from = "file2.txt"
arguments.to = "/dev/null"

[[exports]]
exporter = "copy"
arguments.from = "file3.txt"
arguments.to = "/dev/null"

[[generations]]
generator = "text"
arguments.name = "file1.txt"
arguments.text = "content1"

[[generations]]
generator = "text"
arguments.name = "file2.txt"
arguments.text = "content2"

[[generations]]
generator = "text"
arguments.name = "file3.txt"
arguments.text = "content3"
"#;

  fs::write(&spec_path, spec_content).unwrap();

  // Run with --max-exports=2 (should fail)
  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd
    .arg("path")
    .arg(&spec_path)
    .arg("--nosandbox")
    .arg("--stay")
    .arg("--keep")
    .arg("--max-exports=2")
    .current_dir(&work_dir);

  cmd.assert().failure().stderr(predicates::str::contains(
    "Export count (3) exceeds maximum allowed (2)",
  ));

  // Run with --max-exports=3 (should succeed)
  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd
    .arg("path")
    .arg(&spec_path)
    .arg("--nosandbox")
    .arg("--stay")
    .arg("--keep")
    .arg("--max-exports=3")
    .current_dir(&work_dir);

  cmd.assert().success();
}

/// Test --max-specification-size limit enforcement
#[test]
fn test_max_specification_size_limit() {
  let temp_dir = TempDir::new().unwrap();
  let spec_path = temp_dir.path().join("spec.toml");
  let work_dir = temp_dir.path().join("work");

  fs::create_dir(&work_dir).unwrap();

  // Create a spec with some content
  let spec_content = r#"
imports = []
exports = []

[[generations]]
generator = "text"
arguments.name = "test.txt"
arguments.text = "some content here that makes the file bigger than 50 bytes"
"#;

  fs::write(&spec_path, spec_content).unwrap();

  // Run with small max-specification-size (should fail)
  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd
    .arg("path")
    .arg(&spec_path)
    .arg("--nosandbox")
    .arg("--stay")
    .arg("--keep")
    .arg("--max-specification-size=50")
    .current_dir(&work_dir);

  cmd
    .assert()
    .failure()
    .stderr(predicates::str::contains("exceeds maximum allowed"));

  // Run with larger limit (should succeed)
  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd
    .arg("path")
    .arg(&spec_path)
    .arg("--nosandbox")
    .arg("--stay")
    .arg("--keep")
    .arg("--max-specification-size=500")
    .current_dir(&work_dir);

  cmd.assert().success();
}

/// Test that script generator is blocked without --allow-script
#[test]
fn test_script_generator_blocked_without_flag() {
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

  // Run without --allow-script (should fail)
  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd
    .arg("path")
    .arg(&spec_path)
    .arg("--nosandbox")
    .arg("--stay")
    .arg("--keep")
    .current_dir(&work_dir);

  cmd
    .assert()
    .failure()
    .stderr(predicates::str::contains("Script generator not allowed"));
}

/// Test that script generator works with --allow-script
#[test]
fn test_script_generator_allowed_with_flag() {
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

  // Run with --allow-script (should succeed)
  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd
    .arg("path")
    .arg(&spec_path)
    .arg("--nosandbox")
    .arg("--stay")
    .arg("--keep")
    .arg("--allow-script")
    .current_dir(&work_dir);

  cmd.assert().success();

  // Verify the script file was created
  assert!(work_dir.join("test.nu").exists());
}

/// Test combination of dry-run and allow-script
#[test]
fn test_dry_run_with_allow_script() {
  let temp_dir = TempDir::new().unwrap();
  let spec_path = temp_dir.path().join("spec.toml");
  let work_dir = temp_dir.path().join("work");

  fs::create_dir(&work_dir).unwrap();

  let spec_content = r#"
imports = []

[[exports]]
exporter = "copy"
arguments.from = "test.nu"
arguments.to = "/dev/null"

[[generations]]
generator = "script"
arguments.name = "test.nu"
arguments.text = "echo 'hello'"
"#;

  fs::write(&spec_path, spec_content).unwrap();

  // Run with both --allow-script and --dry-run
  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd
    .arg("path")
    .arg(&spec_path)
    .arg("--nosandbox")
    .arg("--stay")
    .arg("--keep")
    .arg("--allow-script")
    .arg("--dry-run")
    .current_dir(&work_dir);

  cmd.assert().success();

  // Script generation should happen
  assert!(
    work_dir.join("test.nu").exists(),
    "Script file should be created"
  );
  // But export should be skipped due to dry-run
}

/// Test default max limits are high enough for normal specs
#[test]
fn test_default_max_limits_sufficient() {
  let temp_dir = TempDir::new().unwrap();
  let spec_path = temp_dir.path().join("spec.toml");
  let work_dir = temp_dir.path().join("work");

  fs::create_dir(&work_dir).unwrap();

  // Spec with multiple imports, generations, exports
  let spec_content = r#"
[[exports]]
exporter = "copy"
arguments.from = "gen1.txt"
arguments.to = "/dev/null"

[[exports]]
exporter = "copy"
arguments.from = "gen2.txt"
arguments.to = "/dev/null"

[[imports]]
importer = "copy"
arguments.from = "dummy1.txt"
arguments.to = "import1.txt"
arguments.allow_fail = true

[[imports]]
importer = "copy"
arguments.from = "dummy2.txt"
arguments.to = "import2.txt"
arguments.allow_fail = true

[[generations]]
generator = "text"
arguments.name = "gen1.txt"
arguments.text = "content1"

[[generations]]
generator = "text"
arguments.name = "gen2.txt"
arguments.text = "content2"
"#;

  fs::write(&spec_path, spec_content).unwrap();

  // Run with default limits (should succeed with defaults of 1024)
  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd
    .arg("path")
    .arg(&spec_path)
    .arg("--nosandbox")
    .arg("--stay")
    .arg("--keep")
    .current_dir(&work_dir);

  cmd.assert().success();
}
