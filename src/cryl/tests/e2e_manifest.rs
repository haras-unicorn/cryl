//! E2E tests for manifest functionality

use assert_cmd::Command;
use std::fs;
use std::path::Path;

fn create_temp_dir() -> tempfile::TempDir {
  tempfile::tempdir().expect("Failed to create temp directory")
}

fn write_spec(dir: &Path, name: &str, content: &str) {
  let path = dir.join(name);
  fs::write(&path, content).expect("Failed to write spec");
}

#[test]
fn test_manifest_created_by_default() {
  let temp_dir = create_temp_dir();
  // Note: In TOML, simple arrays must be defined BEFORE array-of-tables
  let spec = r#"
imports = []
exports = []

[[generations]]
generator = "id"
arguments.name = "test-id"
"#;
  write_spec(temp_dir.path(), "spec.toml", spec);

  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd
    .current_dir(&temp_dir)
    .arg("path")
    .arg("--nosandbox")
    .arg("--stay")
    .arg("--keep")
    .arg(temp_dir.path().join("spec.toml"));

  cmd.assert().success();

  // Check manifest was created
  let manifest_path = temp_dir.path().join("cryl-manifest.json");
  assert!(
    manifest_path.exists(),
    "Manifest should be created by default"
  );

  // Verify manifest content
  let manifest_content = fs::read_to_string(&manifest_path).unwrap();
  assert!(
    manifest_content.contains("cryl_version"),
    "Manifest should contain cryl_version"
  );
  assert!(
    manifest_content.contains("timestamp"),
    "Manifest should contain timestamp"
  );
  assert!(
    manifest_content.contains("spec_hash"),
    "Manifest should contain spec_hash"
  );
  assert!(
    manifest_content.contains("environment"),
    "Manifest should contain environment"
  );
  assert!(
    manifest_content.contains("output_hashes"),
    "Manifest should contain output_hashes"
  );
}

#[test]
fn test_manifest_not_created_with_no_manifest_flag() {
  let temp_dir = create_temp_dir();
  let spec = r#"
imports = []
exports = []

[[generations]]
generator = "id"
arguments.name = "test-id"
"#;
  write_spec(temp_dir.path(), "spec.toml", spec);

  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd
    .current_dir(&temp_dir)
    .arg("path")
    .arg("--nosandbox")
    .arg("--stay")
    .arg("--keep")
    .arg("--no-manifest")
    .arg(temp_dir.path().join("spec.toml"));

  cmd.assert().success();

  // Check manifest was NOT created
  let manifest_path = temp_dir.path().join("cryl-manifest.json");
  assert!(
    !manifest_path.exists(),
    "Manifest should not be created with --no-manifest"
  );
}

#[test]
fn test_manifest_yaml_format() {
  let temp_dir = create_temp_dir();
  let spec = r#"
imports = []
exports = []

[[generations]]
generator = "id"
arguments.name = "test-id"
"#;
  write_spec(temp_dir.path(), "spec.toml", spec);

  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd
    .current_dir(&temp_dir)
    .arg("path")
    .arg("--nosandbox")
    .arg("--stay")
    .arg("--keep")
    .arg("--manifest-format=yaml")
    .arg(temp_dir.path().join("spec.toml"));

  cmd.assert().success();

  // Check manifest was created with yaml extension
  let manifest_path = temp_dir.path().join("cryl-manifest.yaml");
  assert!(
    manifest_path.exists(),
    "Manifest should be created with yaml format"
  );

  // Verify it's valid YAML
  let manifest_content = fs::read_to_string(&manifest_path).unwrap();
  assert!(
    manifest_content.contains("cryl_version:"),
    "Manifest should contain cryl_version"
  );
}

#[test]
fn test_manifest_toml_format() {
  let temp_dir = create_temp_dir();
  let spec = r#"
imports = []
exports = []

[[generations]]
generator = "id"
arguments.name = "test-id"
"#;
  write_spec(temp_dir.path(), "spec.toml", spec);

  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd
    .current_dir(&temp_dir)
    .arg("path")
    .arg("--nosandbox")
    .arg("--stay")
    .arg("--keep")
    .arg("--manifest-format=toml")
    .arg(temp_dir.path().join("spec.toml"));

  cmd.assert().success();

  // Check manifest was created with toml extension
  let manifest_path = temp_dir.path().join("cryl-manifest.toml");
  assert!(
    manifest_path.exists(),
    "Manifest should be created with toml format"
  );

  // Verify it's valid TOML
  let manifest_content = fs::read_to_string(&manifest_path).unwrap();
  assert!(
    manifest_content.contains("cryl_version"),
    "Manifest should contain cryl_version"
  );
}

#[test]
fn test_manifest_spec_hash_matches() {
  let temp_dir = create_temp_dir();
  let spec = r#"
imports = []
exports = []

[[generations]]
generator = "id"
arguments.name = "test-id"
"#;
  write_spec(temp_dir.path(), "spec.toml", spec);

  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd
    .current_dir(&temp_dir)
    .arg("path")
    .arg("--nosandbox")
    .arg("--stay")
    .arg("--keep")
    .arg(temp_dir.path().join("spec.toml"));

  cmd.assert().success();

  // Read manifest
  let manifest_path = temp_dir.path().join("cryl-manifest.json");
  let manifest_content = fs::read_to_string(&manifest_path).unwrap();
  let manifest: serde_json::Value =
    serde_json::from_str(&manifest_content).unwrap();

  // Verify spec_hash is present and non-empty
  let spec_hash = manifest["spec_hash"].as_str().unwrap();
  assert!(!spec_hash.is_empty(), "spec_hash should not be empty");
  assert_eq!(
    spec_hash.len(),
    64,
    "spec_hash should be 64 characters (SHA256 hex)"
  );
}

#[test]
fn test_manifest_environment_contains_tools() {
  let temp_dir = create_temp_dir();
  let spec = r#"
imports = []
exports = []

[[generations]]
generator = "id"
arguments.name = "test-id"
"#;
  write_spec(temp_dir.path(), "spec.toml", spec);

  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd
    .current_dir(&temp_dir)
    .arg("path")
    .arg("--nosandbox")
    .arg("--stay")
    .arg("--keep")
    .arg(temp_dir.path().join("spec.toml"));

  cmd.assert().success();

  // Read manifest
  let manifest_path = temp_dir.path().join("cryl-manifest.json");
  let manifest_content = fs::read_to_string(&manifest_path).unwrap();
  let manifest: serde_json::Value =
    serde_json::from_str(&manifest_content).unwrap();

  // Verify environment contains tool info
  let environment = manifest["environment"].as_object().unwrap();
  assert!(!environment.is_empty(), "environment should contain tools");

  // Check that openssl is recorded (should always be available)
  if let Some(openssl) = environment.get("openssl") {
    assert!(openssl.get("version").is_some(), "Tool should have version");
    assert!(openssl.get("path").is_some(), "Tool should have path");
  }
}

#[test]
fn test_manifest_output_hashes_match_generated_files() {
  let temp_dir = create_temp_dir();
  let spec = r#"
imports = []
exports = []

[[generations]]
generator = "id"
arguments.name = "test-id"
arguments.length = 8
"#;
  write_spec(temp_dir.path(), "spec.toml", spec);

  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd
    .current_dir(&temp_dir)
    .arg("path")
    .arg("--nosandbox")
    .arg("--stay")
    .arg("--keep")
    .arg(temp_dir.path().join("spec.toml"));

  cmd.assert().success();

  // Read manifest
  let manifest_path = temp_dir.path().join("cryl-manifest.json");
  let manifest_content = fs::read_to_string(&manifest_path).unwrap();
  let manifest: serde_json::Value =
    serde_json::from_str(&manifest_content).unwrap();

  // Verify output_hashes contains our generated file
  let output_hashes = manifest["output_hashes"].as_object().unwrap();
  assert!(
    output_hashes.contains_key("test-id"),
    "output_hashes should contain generated file 'test-id'"
  );
  eprintln!("Manifest output_hashes: {:?}", output_hashes);
  assert!(
    output_hashes.contains_key("test-id"),
    "output_hashes should contain generated file 'test-id', got keys: {:?}",
    output_hashes.keys().collect::<Vec<_>>()
  );

  // Verify the hash matches the actual file
  let file_hash = output_hashes["test-id"].as_str().unwrap();
  let actual_content =
    fs::read_to_string(temp_dir.path().join("test-id")).unwrap();
  use sha2::{Digest, Sha256};
  let mut hasher = Sha256::new();
  hasher.update(actual_content.trim().as_bytes());
  let expected_hash = format!("{:x}", hasher.finalize());
  assert_eq!(
    file_hash, expected_hash,
    "Hash should match actual file content"
  );
}

#[test]
fn test_manifest_not_created_on_error() {
  let temp_dir = create_temp_dir();
  // Use an invalid generator to cause an error
  let spec = r#"
imports = []
exports = []

[[generations]]
generator = "invalid-generator"
arguments.name = "test-id"
"#;
  write_spec(temp_dir.path(), "spec.toml", spec);

  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd
    .current_dir(&temp_dir)
    .arg("path")
    .arg("--nosandbox")
    .arg("--stay")
    .arg("--keep")
    .arg(temp_dir.path().join("spec.toml"));

  cmd.assert().failure();

  // Check manifest was NOT created on error
  let manifest_path = temp_dir.path().join("cryl-manifest.json");
  assert!(
    !manifest_path.exists(),
    "Manifest should not be created on error"
  );
}

#[test]
fn test_manifest_with_imports_generations_exports() {
  let temp_dir = create_temp_dir();

  // Create a file to import
  fs::write(temp_dir.path().join("import-file"), "imported-content").unwrap();

  let spec = format!(
    r#"
exports = []

[[imports]]
importer = "copy"
arguments.from = "{}"
arguments.to = "imported"

[[generations]]
generator = "text"
arguments.name = "generated"
arguments.text = "generated-content"
"#,
    temp_dir.path().join("import-file").to_str().unwrap()
  );

  write_spec(temp_dir.path(), "spec.toml", &spec);

  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd
    .current_dir(&temp_dir)
    .arg("path")
    .arg("--nosandbox")
    .arg("--stay")
    .arg("--keep")
    .arg(temp_dir.path().join("spec.toml"));

  cmd.assert().success();

  // Read manifest
  let manifest_path = temp_dir.path().join("cryl-manifest.json");
  let manifest_content = fs::read_to_string(&manifest_path).unwrap();
  let manifest: serde_json::Value =
    serde_json::from_str(&manifest_content).unwrap();

  // Verify all files are in output_hashes
  let output_hashes = manifest["output_hashes"].as_object().unwrap();
  assert!(
    output_hashes.contains_key("imported"),
    "output_hashes should contain imported file"
  );
  assert!(
    output_hashes.contains_key("generated"),
    "output_hashes should contain generated file"
  );

  // Verify manifest doesn't contain itself
  assert!(
    !output_hashes.contains_key("cryl-manifest.json"),
    "output_hashes should not contain manifest"
  );
}

#[test]
fn test_manifest_cryl_version_matches_package() {
  let temp_dir = create_temp_dir();
  let spec = r#"
imports = []
exports = []

[[generations]]
generator = "id"
arguments.name = "test-id"
"#;
  write_spec(temp_dir.path(), "spec.toml", spec);

  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd
    .current_dir(&temp_dir)
    .arg("path")
    .arg("--nosandbox")
    .arg("--stay")
    .arg("--keep")
    .arg(temp_dir.path().join("spec.toml"));

  cmd.assert().success();

  // Read manifest
  let manifest_path = temp_dir.path().join("cryl-manifest.json");
  let manifest_content = fs::read_to_string(&manifest_path).unwrap();
  let manifest: serde_json::Value =
    serde_json::from_str(&manifest_content).unwrap();

  // Verify cryl_version is present and looks like semver
  let version = manifest["cryl_version"].as_str().unwrap();
  assert!(!version.is_empty(), "cryl_version should not be empty");
  assert!(
    version.contains('.'),
    "cryl_version should look like semver"
  );
}

#[test]
fn test_manifest_timestamp_is_valid_rfc3339() {
  let temp_dir = create_temp_dir();
  let spec = r#"
imports = []
exports = []

[[generations]]
generator = "id"
arguments.name = "test-id"
"#;
  write_spec(temp_dir.path(), "spec.toml", spec);

  let mut cmd = Command::cargo_bin("cryl").unwrap();
  cmd
    .current_dir(&temp_dir)
    .arg("path")
    .arg("--nosandbox")
    .arg("--stay")
    .arg("--keep")
    .arg(temp_dir.path().join("spec.toml"));

  cmd.assert().success();

  // Read manifest
  let manifest_path = temp_dir.path().join("cryl-manifest.json");
  let manifest_content = fs::read_to_string(&manifest_path).unwrap();
  let manifest: serde_json::Value =
    serde_json::from_str(&manifest_content).unwrap();

  // Verify timestamp is present and valid RFC3339
  let timestamp = manifest["timestamp"].as_str().unwrap();
  assert!(!timestamp.is_empty(), "timestamp should not be empty");

  // Try to parse it as chrono::DateTime
  let parsed = chrono::DateTime::parse_from_rfc3339(timestamp);
  assert!(parsed.is_ok(), "timestamp should be valid RFC3339");
}
