use std::path::PathBuf;

use crate::common::{CrylError, CrylResult};

/// Generate random password with special characters
pub fn generate_password(
  name: &PathBuf,
  length: usize,
  _renew: bool,
) -> CrylResult<()> {
  let mut result = String::new();
  // Include alphanumeric + special characters, excluding confusing ones
  let alphabet: Vec<char> = ('a'..='z')
    .chain('A'..='Z')
    .chain('0'..='9')
    .chain("!@#$%^&*-_+=".chars())
    .collect();

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
      if alphabet.contains(&c) {
        result.push(c);
      }
    }
  }

  // Write to destination
  std::fs::write(name, result)?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_generate_password() {
    let result = generate_password(16).unwrap();
    assert_eq!(result.len(), 16);
    // Password should contain at least alphanumeric characters
    assert!(result
      .chars()
      .all(|c| c.is_ascii_alphanumeric() || "!@#$%^&*-_+=".contains(c)));
  }
}
