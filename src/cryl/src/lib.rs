//! cryl - Secret generation tool
//!
//! A high-performance, sandboxed CLI tool for generating, encrypting, and
//! managing infrastructure secrets.

#![deny(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![deny(clippy::arithmetic_side_effects)]
#![deny(clippy::todo)]
#![deny(clippy::unreachable)]
#![deny(clippy::allow_attributes_without_reason)]

use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

pub mod format;
pub mod schema;

/// Errors that can occur during cryl operations
#[derive(Error, Debug)]
pub enum CrylError {
  #[error("IO error: {0}")]
  Io(#[from] std::io::Error),

  #[error("Serialization error: {0}")]
  Serialization(#[from] serde_json::Error),

  #[error("YAML serialization error: {0}")]
  YamlSerialization(#[from] serde_yaml::Error),

  #[error("TOML serialization error: {0}")]
  TomlSerialization(#[from] toml::ser::Error),

  #[error("TOML deserialization error: {0}")]
  TomlDeserialization(#[from] toml::de::Error),

  #[error("Invalid specification: {message}")]
  InvalidSpec { message: String },

  #[error("Tool execution failed: {tool} exited with {exit_code}")]
  ToolExecution {
    tool: String,
    exit_code: i32,
    stderr: String,
  },

  #[error("Sandbox error: {0}")]
  Sandbox(String),

  #[error("Import failed: {importer} - {message}")]
  Import { importer: String, message: String },

  #[error("Generation failed: {generator} - {message}")]
  Generation { generator: String, message: String },

  #[error("Export failed: {exporter} - {message}")]
  Export { exporter: String, message: String },

  #[error("Tool not found: {0}")]
  ToolNotFound(String),

  #[error("Invalid format: {0}")]
  InvalidFormat(String),

  #[error("Validation failed: {0}")]
  Validation(String),
}

/// Result type alias for cryl operations
pub type Result<T> = std::result::Result<T, CrylError>;

/// Manifest containing execution metadata and output hashes
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Manifest {
  pub version: String,
  pub environment: HashMap<String, String>,
  pub spec_hash: String,
  pub output_hashes: HashMap<String, String>,
}

impl Manifest {
  /// Compute SHA256 hash of content
  pub fn compute_hash(content: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(content);
    hex::encode(hasher.finalize())
  }

  /// Compute hash of a file
  pub fn compute_file_hash(path: &Path) -> Result<String> {
    let content = std::fs::read(path)?;
    Ok(Self::compute_hash(&content))
  }
}

/// Trait for building and executing external commands
pub trait CommandBuilder {
  /// Get the program name
  fn program(&self) -> &str;

  /// Get the arguments
  fn args(&self) -> &[String];

  /// Execute the command and return output
  fn execute(&self) -> Result<std::process::Output> {
    let output = std::process::Command::new(self.program())
      .args(self.args())
      .output()?;

    if !output.status.success() {
      return Err(CrylError::ToolExecution {
        tool: self.program().to_string(),
        exit_code: output.status.code().unwrap_or(-1),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
      });
    }

    Ok(output)
  }
}

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
) -> Result<()> {
  use std::fs;
  use std::os::unix::fs::PermissionsExt;

  let path = path.as_ref();

  // Check if file exists and we shouldn't renew
  if !renew && path.exists() {
    return Ok(());
  }

  let tmp_path = path.with_extension("tmp");

  // Write to temp file
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
pub fn read_file_if_exists<P: AsRef<Path>>(path: P) -> Result<Option<String>> {
  match std::fs::read_to_string(path) {
    Ok(content) => Ok(Some(content)),
    Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
    Err(e) => Err(e.into()),
  }
}

/// Generate random alphanumeric string using OpenSSL
pub fn generate_random_alnum(length: usize) -> Result<String> {
  let mut result = String::new();
  let alphabet: Vec<char> =
    ('a'..='z').chain('A'..='Z').chain('0'..='9').collect();

  while result.len() < length {
    let needed = length.saturating_sub(result.len());
    let batch_size = std::cmp::max(needed.saturating_mul(2), 32);

    // Use OpenSSL for randomness
    let output = std::process::Command::new("openssl")
      .args(["rand", "-base64", &batch_size.to_string()])
      .output()?;

    if !output.status.success() {
      return Err(CrylError::ToolExecution {
        tool: "openssl".to_string(),
        exit_code: output.status.code().unwrap_or(-1),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
      });
    }

    let base64 = String::from_utf8_lossy(&output.stdout);
    for c in base64.chars() {
      if result.len() >= length {
        break;
      }
      if alphabet.contains(&c) {
        result.push(c);
      }
    }
  }

  Ok(result)
}

/// Generate random numeric string (digits only)
pub fn generate_random_digits(length: usize) -> Result<String> {
  let mut result = String::new();

  while result.len() < length {
    let needed = length.saturating_sub(result.len());
    let batch_size = std::cmp::max(needed.saturating_mul(2), 32);

    let output = std::process::Command::new("openssl")
      .args(["rand", "-base64", &batch_size.to_string()])
      .output()?;

    if !output.status.success() {
      return Err(CrylError::ToolExecution {
        tool: "openssl".to_string(),
        exit_code: output.status.code().unwrap_or(-1),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
      });
    }

    let base64 = String::from_utf8_lossy(&output.stdout);
    for c in base64.chars() {
      if result.len() >= length {
        break;
      }
      if c.is_ascii_digit() {
        result.push(c);
      }
    }
  }

  Ok(result)
}

#[cfg(test)]
mod tests {
  use super::*;
  use tempfile::TempDir;

  #[test]
  fn test_compute_hash() {
    let data = b"test data";
    let hash = Manifest::compute_hash(data);
    assert_eq!(hash.len(), 64); // SHA256 hex is 64 chars
  }

  #[test]
  fn test_generate_random_alnum() {
    let result = generate_random_alnum(16).unwrap();
    assert_eq!(result.len(), 16);
    assert!(result.chars().all(|c| c.is_ascii_alphanumeric()));
  }

  #[test]
  fn test_generate_random_digits() {
    let result = generate_random_digits(8).unwrap();
    assert_eq!(result.len(), 8);
    assert!(result.chars().all(|c| c.is_ascii_digit()));
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
}
