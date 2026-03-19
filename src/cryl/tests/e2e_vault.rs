//! Vault E2E tests for cryl
//!
//! These tests run the cryl binary with Vault-based specifications
//! using testcontainers to spin up a real Vault instance.

use assert_cmd::Command;
use serial_test::serial;
use std::fs;
use std::process::Command as StdCommand;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use testcontainers::{
  ContainerAsync, GenericImage, ImageExt, runners::AsyncRunner,
};
use tokio::net::TcpStream;

/// Helper to start a Vault container for testing
async fn vault_container(
  root_token: &str,
) -> anyhow::Result<ContainerAsync<GenericImage>> {
  let container = GenericImage::new("hashicorp/vault", "1.14")
    .with_env_var("VAULT_DEV_ROOT_TOKEN_ID", root_token)
    .with_exposed_host_port(8200)
    .start()
    .await?;

  let host_port = container.get_host_port_ipv4(8200).await?;
  let addr = format!("127.0.0.1:{}", host_port);

  let start = Instant::now();
  let timeout = Duration::from_secs(30);

  loop {
    match TcpStream::connect(&addr).await {
      Ok(_) => break,
      Err(_) if start.elapsed() < timeout => {
        tokio::time::sleep(Duration::from_millis(100)).await;
        continue;
      }
      Err(e) => {
        return Err(anyhow::anyhow!(
          "Vault port never became reachable: {}",
          e
        ));
      }
    }
  }

  let vault_addr = format!("http://{}", addr);
  let client = reqwest::Client::new();
  let health_timeout = Duration::from_secs(10);

  loop {
    match client
      .get(format!("{}/v1/sys/health", vault_addr))
      .timeout(health_timeout)
      .send()
      .await
    {
      Ok(resp) if resp.status().is_success() => break,
      _ if start.elapsed() < timeout => {
        tokio::time::sleep(Duration::from_millis(500)).await;
        continue;
      }
      _ => return Err(anyhow::anyhow!("Vault health check never passed")),
    }
  }

  #[allow(unsafe_code, reason = "Tested in serial tests")]
  unsafe {
    std::env::set_var("VAULT_ADDR", &vault_addr);
    std::env::set_var("VAULT_TOKEN", root_token);
    std::env::set_var("VAULT_SKIP_VERIFY", "true");
  }

  StdCommand::new("vault")
    .args(["secrets", "enable", "-path=kv", "kv-v2"])
    .output()?;

  Ok(container)
}

/// Test vault export (all files in directory)
#[tokio::test]
#[serial]
async fn test_vault_export() {
  let _container: ContainerAsync<GenericImage> =
    vault_container("cryl-vault-export-e2e").await.unwrap();

  let temp_dir = TempDir::new().unwrap();
  let spec_path = temp_dir.path().join("spec.toml");
  let work_dir = temp_dir.path().join("work");

  fs::create_dir(&work_dir).unwrap();

  // Spec that generates secrets and exports to vault
  let spec_content = r#"
imports = []

[[exports]]
exporter = "vault"
arguments.path = "kv/exported-app"

[[generations]]
generator = "text"
arguments.name = "config.json"
arguments.text = '{"port": 8080, "debug": false}'

[[generations]]
generator = "text"
arguments.name = "secret.key"
arguments.text = "my-secret-key-123"
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

  // Verify files were created locally
  assert!(work_dir.join("config.json").exists());
  assert!(work_dir.join("secret.key").exists());

  // Verify they were exported to vault
  let output = StdCommand::new("vault")
    .args(["kv", "get", "-format=json", "kv/exported-app/current"])
    .output()
    .expect("Failed to get vault data");

  assert!(output.status.success(), "Vault get should succeed");

  let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
  assert_eq!(
    json["data"]["data"]["config.json"],
    "{\"port\": 8080, \"debug\": false}"
  );
  assert_eq!(json["data"]["data"]["secret.key"], "my-secret-key-123");
}

/// Test vault_file exporter for single file
#[tokio::test]
#[serial]
async fn test_vault_file_export() {
  let _container: ContainerAsync<GenericImage> =
    vault_container("cryl-vault-file-export-e2e").await.unwrap();

  let temp_dir = TempDir::new().unwrap();
  let spec_path = temp_dir.path().join("spec.toml");
  let work_dir = temp_dir.path().join("work");

  fs::create_dir(&work_dir).unwrap();

  // Spec that generates a secret and exports single file to vault
  let spec_content = r#"
imports = []

[[exports]]
exporter = "vault-file"
arguments.path = "kv/single-secret"
arguments.file = "master.key"

[[generations]]
generator = "text"
arguments.name = "master.key"
arguments.text = "master-secret-value-xyz789"
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

  // Verify the file was created locally
  assert!(work_dir.join("master.key").exists());

  // Verify it was exported to vault
  let output = StdCommand::new("vault")
    .args(["kv", "get", "-format=json", "kv/single-secret/current"])
    .output()
    .expect("Failed to get vault data");

  assert!(output.status.success(), "Vault get should succeed");

  let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
  assert_eq!(
    json["data"]["data"]["master.key"],
    "master-secret-value-xyz789"
  );
}

/// Test vault export with multiple generations
#[tokio::test]
#[serial]
async fn test_vault_export_multiple_files() {
  let _container: ContainerAsync<GenericImage> =
    vault_container("cryl-vault-multi-e2e").await.unwrap();

  let temp_dir = TempDir::new().unwrap();
  let spec_path = temp_dir.path().join("spec.toml");
  let work_dir = temp_dir.path().join("work");

  fs::create_dir(&work_dir).unwrap();

  // Spec that generates multiple files and exports all to vault
  let spec_content = r##"
imports = []

[[exports]]
exporter = "vault"
arguments.path = "kv/multi-export"

[[generations]]
generator = "id"
arguments.name = "api-key"
arguments.length = 32

[[generations]]
generator = "text"
arguments.name = "README.md"
arguments.text = '''# API Configuration
Use the api-key file for authentication.'''

[[generations]]
generator = "json"
arguments.name = "settings.json"
arguments.value = { timeout = 30, retries = 3 }
"##;

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

  // Verify all files were created locally
  assert!(work_dir.join("api-key").exists());
  assert!(work_dir.join("README.md").exists());
  assert!(work_dir.join("settings.json").exists());

  // Verify they were exported to vault
  let output = StdCommand::new("vault")
    .args(["kv", "get", "-format=json", "kv/multi-export/current"])
    .output()
    .expect("Failed to get vault data");

  assert!(output.status.success(), "Vault get should succeed");

  let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

  // Check api-key is 32 alphanumeric chars
  let api_key = json["data"]["data"]["api-key"].as_str().unwrap();
  assert_eq!(api_key.len(), 32);
  assert!(api_key.chars().all(|c| c.is_ascii_alphanumeric()));

  // Check README
  assert_eq!(
    json["data"]["data"]["README.md"],
    "# API Configuration\nUse the api-key file for authentication."
  );

  // Check JSON settings
  let settings = json["data"]["data"]["settings.json"].as_str().unwrap();
  assert!(settings.contains("timeout"));
  assert!(settings.contains("30"));
}

/// Test vault export with dry-run (should not export)
#[tokio::test]
#[serial]
async fn test_vault_export_dry_run() {
  let _container: ContainerAsync<GenericImage> =
    vault_container("cryl-vault-dry-run-e2e").await.unwrap();

  let temp_dir = TempDir::new().unwrap();
  let spec_path = temp_dir.path().join("spec.toml");
  let work_dir = temp_dir.path().join("work");

  fs::create_dir(&work_dir).unwrap();

  let spec_content = r#"
imports = []

[[exports]]
exporter = "vault"
arguments.path = "kv/dry-run-app"

[[generations]]
generator = "text"
arguments.name = "secret.txt"
arguments.text = "should-not-be-exported"
"#;

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

  // File should be created locally
  assert!(work_dir.join("secret.txt").exists());

  // But should NOT be exported to vault
  let output = StdCommand::new("vault")
    .args(["kv", "get", "-format=json", "kv/dry-run-app/current"])
    .output()
    .expect("Failed to get vault data");

  // This should fail because the path doesn't exist
  assert!(
    !output.status.success(),
    "Vault path should not exist in dry-run"
  );
}

/// Test vault_file export with JSON data
#[tokio::test]
#[serial]
async fn test_vault_file_export_json() {
  let _container: ContainerAsync<GenericImage> =
    vault_container("cryl-vault-json-e2e").await.unwrap();

  let temp_dir = TempDir::new().unwrap();
  let spec_path = temp_dir.path().join("spec.toml");
  let work_dir = temp_dir.path().join("work");

  fs::create_dir(&work_dir).unwrap();

  let spec_content = r#"
imports = []

[[exports]]
exporter = "vault-file"
arguments.path = "kv/json-export"
arguments.file = "data.json"

[[generations]]
generator = "json"
arguments.name = "data.json"
arguments.value = { database = { host = "localhost", port = 5432 }, cache = { enabled = true } }
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

  // Verify the JSON file was created locally
  assert!(work_dir.join("data.json").exists());

  // Verify it was exported to vault
  let output = StdCommand::new("vault")
    .args(["kv", "get", "-format=json", "kv/json-export/current"])
    .output()
    .expect("Failed to get vault data");

  assert!(output.status.success(), "Vault get should succeed");

  let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
  let data = json["data"]["data"]["data.json"].as_str().unwrap();

  // Verify the JSON structure is preserved
  assert!(data.contains("database"));
  assert!(data.contains("localhost"));
  assert!(data.contains("5432"));
  assert!(data.contains("cache"));
  assert!(data.contains("enabled"));
}
