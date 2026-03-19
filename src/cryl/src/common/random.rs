use super::{CrylError, CrylResult};

/// Generate random alphanumeric string using OpenSSL
pub fn generate_random_alphanumeric(length: usize) -> CrylResult<String> {
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
pub fn generate_random_digits(length: usize) -> CrylResult<String> {
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

  #[test]
  fn test_generate_random_alphanumeric() {
    let result = generate_random_alphanumeric(16).unwrap();
    assert_eq!(result.len(), 16);
    assert!(result.chars().all(|c| c.is_ascii_alphanumeric()));
  }

  #[test]
  fn test_generate_random_digits() {
    let result = generate_random_digits(8).unwrap();
    assert_eq!(result.len(), 8);
    assert!(result.chars().all(|c| c.is_ascii_digit()));
  }
}
