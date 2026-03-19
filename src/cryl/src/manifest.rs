use crate::common::{CrylError, CrylResult, Format, serialize};
use crate::versions::{cryl_version, tool_version};
use std::collections::HashMap;
use std::path::Path;

/// Information about a tool used during execution
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolInfo {
  /// Version of the tool
  pub version: String,
  /// Canonicalized path to the tool binary
  pub path: String,
}

/// Manifest containing execution metadata and output hashes
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Manifest {
  /// Version of cryl itself
  pub cryl_version: String,
  /// Timestamp of when the manifest was created (ISO 8601 format)
  pub timestamp: String,
  /// SHA256 hash of the input specification content
  pub spec_hash: String,
  /// Format of the specification (json, yaml, toml)
  pub spec_format: String,
  /// Map of tool names to their version information
  pub environment: HashMap<String, ToolInfo>,
  /// Map of output file paths to their SHA256 hashes
  pub output_hashes: HashMap<String, String>,
}

impl Manifest {
  /// Create a new manifest for the given specification
  pub fn new(spec_content: &str, spec_format: Format) -> Self {
    Self {
      cryl_version: cryl_version().to_string(),
      timestamp: chrono::Utc::now().to_rfc3339(),
      spec_hash: Self::compute_hash(spec_content.as_bytes()),
      spec_format: format!("{:?}", spec_format).to_lowercase(),
      environment: HashMap::new(),
      output_hashes: HashMap::new(),
    }
  }

  /// Compute SHA256 hash of content
  pub fn compute_hash(content: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(content);
    hex::encode(hasher.finalize())
  }

  /// Compute hash of a file
  pub fn compute_file_hash(path: &Path) -> CrylResult<String> {
    let content = std::fs::read(path)?;
    Ok(Self::compute_hash(&content))
  }

  /// Record a tool that was used during execution
  pub fn record_tool(&mut self, tool: &str) {
    if let Ok(tool_path) = which::which(tool)
      && let Ok(canonical) = std::fs::canonicalize(&tool_path)
    {
      let version = tool_version(tool)
        .map(|v| v.to_string())
        .unwrap_or_else(|| "unknown".to_string());

      self.environment.insert(
        tool.to_string(),
        ToolInfo {
          version,
          path: canonical.to_string_lossy().to_string(),
        },
      );
    }
  }

  /// Record an output file and its hash
  pub fn record_output(&mut self, path: &Path) -> CrylResult<()> {
    let hash = Self::compute_file_hash(path)?;
    let path_str = path.to_string_lossy().to_string();
    // Strip leading "./" if present for cleaner paths
    let clean_path = if let Some(stripped) = path_str.strip_prefix("./") {
      stripped.to_string()
    } else {
      path_str
    };
    self.output_hashes.insert(clean_path, hash);
    Ok(())
  }

  /// Record all files in the current directory as outputs
  pub fn record_all_outputs(&mut self) -> CrylResult<()> {
    for entry in std::fs::read_dir(".")? {
      let entry = entry?;
      let path = entry.path();

      // Skip the manifest file itself
      if path
        .file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with("cryl-manifest"))
        .unwrap_or(false)
      {
        continue;
      }

      if path.is_file() {
        self.record_output(&path)?;
      }
    }
    Ok(())
  }

  /// Save the manifest to a file
  pub fn save(&self, format: Format) -> CrylResult<()> {
    let filename = format!("cryl-manifest.{}", format.extension());
    let content = serialize(self, format)?;
    std::fs::write(&filename, content)?;
    Ok(())
  }
}

/// Extension trait for Format to get file extension
trait FormatExt {
  fn extension(&self) -> &'static str;
}

impl FormatExt for Format {
  fn extension(&self) -> &'static str {
    match self {
      Format::Json => "json",
      Format::Yaml => "yaml",
      Format::Toml => "toml",
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_compute_hash() {
    let data = b"test data";
    let hash = Manifest::compute_hash(data);
    assert_eq!(hash.len(), 64); // SHA256 hex is 64 chars
  }

  #[test]
  fn test_manifest_new() {
    let manifest = Manifest::new("test spec", Format::Json);
    assert!(!manifest.cryl_version.is_empty());
    assert!(!manifest.timestamp.is_empty());
    assert!(!manifest.spec_hash.is_empty());
    assert_eq!(manifest.spec_format, "json");
    assert!(manifest.environment.is_empty());
    assert!(manifest.output_hashes.is_empty());
  }

  #[test]
  fn test_record_tool() {
    let mut manifest = Manifest::new("test", Format::Json);
    manifest.record_tool("openssl");
    // In dev environment, version will be "dev" or "unknown"
    assert!(manifest.environment.contains_key("openssl"));
  }
}
