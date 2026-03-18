use crate::common::CrylResult;
use std::{collections::HashMap, path::Path};

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
  pub fn compute_file_hash(path: &Path) -> CrylResult<String> {
    let content = std::fs::read(path)?;
    Ok(Self::compute_hash(&content))
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
}
