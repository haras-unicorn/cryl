use std::path::Path;

use crate::common::{generate_random_alphanumeric, CrylResult};

/// Generate random alphanumeric password
pub fn generate_password(
  name: &Path,
  length: usize,
  renew: bool,
) -> CrylResult<()> {
  let password = generate_random_alphanumeric(length)?;
  crate::common::save_atomic(name, password.as_bytes(), renew, false)?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use tempfile::TempDir;

  use super::*;

  #[test]
  fn test_generate_password() -> anyhow::Result<()> {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("pass");
    generate_password(&path, 16, true)?;
    let result = std::fs::read_to_string(&path)?;
    assert_eq!(result.len(), 16);
    // Password should contain only alphanumeric characters
    assert!(result.chars().all(|c| c.is_ascii_alphanumeric()));
    Ok(())
  }

  #[test]
  fn test_generate_password_no_overwrite_without_renew() -> anyhow::Result<()> {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("pass");
    std::fs::write(&path, "existing")?;
    generate_password(&path, 16, false)?;
    let result = std::fs::read_to_string(&path)?;
    assert_eq!(result, "existing");
    Ok(())
  }
}
