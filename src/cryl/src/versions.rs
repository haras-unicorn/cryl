//! Tool version information
//!
//! This module contains version information for all external tools used by cryl.
//! During the Nix build process, this file is automatically patched to include
//! the actual versions of tools from the build environment.

use std::collections::HashMap;

/// Get the version of cryl itself
pub fn cryl_version() -> &'static str {
  env!("CARGO_PKG_VERSION")
}

/// Get version information for all tools as a HashMap
pub fn tool_versions() -> HashMap<&'static str, &'static str> {
  let mut versions = HashMap::new();

  // These are development defaults - they get patched during Nix build
  versions.insert("age", "dev");
  versions.insert("sops", "dev");
  versions.insert("nebula", "dev");
  versions.insert("openssl", "dev");
  versions.insert("mkpasswd", "dev");
  versions.insert("openssh", "dev");
  versions.insert("wireguard-tools", "dev");
  versions.insert("vault", "dev");
  versions.insert("vault-medusa", "dev");
  versions.insert("libargon2", "dev");
  versions.insert("ssss", "dev");
  versions.insert("cockroachdb", "dev");
  versions.insert("bubblewrap", "dev");
  versions.insert("nushell", "dev");

  versions
}

/// Get version for a specific tool
pub fn tool_version(tool: &str) -> Option<&'static str> {
  tool_versions().get(tool).copied()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_cryl_version() {
    let version = cryl_version();
    assert!(!version.is_empty());
    // Should be a semver-like string
    assert!(version.contains('.'));
  }

  #[test]
  fn test_tool_versions() {
    let versions = tool_versions();
    assert!(!versions.is_empty());
    assert!(versions.contains_key("openssl"));
    assert!(versions.contains_key("age"));
  }

  #[test]
  fn test_tool_version() {
    assert!(tool_version("openssl").is_some());
    assert!(tool_version("nonexistent").is_none());
  }
}
