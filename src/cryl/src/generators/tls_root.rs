use std::path::Path;
use std::process::Command;

use crate::common::{save_atomic, CrylError, CrylResult};

/// Generate a TLS Root CA (private key + self-signed certificate)
///
/// # Arguments
/// * `common_name` - Common Name for the Root CA (e.g., "My Root CA")
/// * `organization` - Organization (e.g., "My Company")
/// * `config` - Path to save the OpenSSL config file (public permissions 644)
/// * `private` - Path to save the private key (private permissions 600)
/// * `public` - Path to save the root certificate (public permissions 644)
/// * `pathlen` - Allowed intermediate depth (use -1 for unlimited)
/// * `days` - Certificate validity in days
/// * `renew` - Overwrite destinations if they exist
pub fn generate_tls_root(
  common_name: &str,
  organization: &str,
  config: &Path,
  private: &Path,
  public: &Path,
  pathlen: i32,
  days: u32,
  renew: bool,
) -> CrylResult<()> {
  // If public cert exists and we're not renewing, skip everything
  if !renew && public.exists() {
    return Ok(());
  }

  // Build basicConstraints based on pathlen
  let basic_constraints = if pathlen < 0 {
    "critical,CA:true".to_string()
  } else {
    format!("critical,CA:true,pathlen:{}", pathlen)
  };

  // Create OpenSSL config
  let config_content = format!(
    r#"[req]
default_md = sha256
distinguished_name = dn
x509_extensions = ext
prompt = no

[dn]
CN = {}
O = {}

[ext]
basicConstraints = {}
keyUsage = critical,keyCertSign,cRLSign
subjectKeyIdentifier = hash
"#,
    common_name, organization, basic_constraints
  );

  // Save config file with public permissions
  save_atomic(config, config_content.as_bytes(), renew, true)?;

  // Generate private key using EC algorithm with prime256v1 curve
  let private_output = Command::new("openssl")
    .arg("genpkey")
    .arg("-algorithm")
    .arg("EC")
    .arg("-pkeyopt")
    .arg("ec_paramgen_curve:prime256v1")
    .arg("-quiet")
    .output()?;

  if !private_output.status.success() {
    return Err(CrylError::ToolExecution {
      tool: "openssl genpkey".to_string(),
      exit_code: private_output.status.code().unwrap_or(-1),
      stderr: String::from_utf8_lossy(&private_output.stderr).to_string(),
    });
  }

  let private_content = String::from_utf8_lossy(&private_output.stdout);

  // Save private key with private permissions (600)
  save_atomic(private, private_content.as_bytes(), renew, false)?;

  // Generate self-signed certificate using the config
  let cert_output = Command::new("openssl")
    .arg("req")
    .arg("-x509")
    .arg("-key")
    .arg(private)
    .arg("-config")
    .arg(config)
    .arg("-days")
    .arg(days.to_string())
    .output()?;

  if !cert_output.status.success() {
    return Err(CrylError::ToolExecution {
      tool: "openssl req -x509".to_string(),
      exit_code: cert_output.status.code().unwrap_or(-1),
      stderr: String::from_utf8_lossy(&cert_output.stderr).to_string(),
    });
  }

  let cert_content = String::from_utf8_lossy(&cert_output.stdout);

  // Save certificate with public permissions (644)
  save_atomic(public, cert_content.as_bytes(), renew, true)?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::os::unix::fs::PermissionsExt;
  use tempfile::TempDir;

  #[test]
  fn test_generate_tls_root_success() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let config_path = temp.path().join("ca.conf");
    let private_path = temp.path().join("ca.key");
    let public_path = temp.path().join("ca.crt");

    generate_tls_root(
      "Test Root CA",
      "Test Org",
      &config_path,
      &private_path,
      &public_path,
      1,
      3650,
      true,
    )?;

    // Check that all files exist
    assert!(config_path.exists());
    assert!(private_path.exists());
    assert!(public_path.exists());

    // Check config content
    let config_content = std::fs::read_to_string(&config_path)?;
    assert!(config_content.contains("[req]"));
    assert!(config_content.contains("CN = Test Root CA"));
    assert!(config_content.contains("O = Test Org"));
    assert!(
      config_content.contains("basicConstraints = critical,CA:true,pathlen:1")
    );
    assert!(config_content.contains("keyUsage = critical,keyCertSign,cRLSign"));

    // Check private key content
    let private_content = std::fs::read_to_string(&private_path)?;
    assert!(private_content.contains("BEGIN PRIVATE KEY"));
    assert!(private_content.contains("END PRIVATE KEY"));

    // Check certificate content
    let cert_content = std::fs::read_to_string(&public_path)?;
    assert!(cert_content.contains("BEGIN CERTIFICATE"));
    assert!(cert_content.contains("END CERTIFICATE"));

    // Check config permissions (644)
    let config_metadata = std::fs::metadata(&config_path)?;
    assert_eq!(config_metadata.permissions().mode() & 0o777, 0o644);

    // Check private key permissions (600)
    let private_metadata = std::fs::metadata(&private_path)?;
    assert_eq!(private_metadata.permissions().mode() & 0o777, 0o600);

    // Check certificate permissions (644)
    let public_metadata = std::fs::metadata(&public_path)?;
    assert_eq!(public_metadata.permissions().mode() & 0o777, 0o644);

    Ok(())
  }

  #[test]
  fn test_generate_tls_root_unlimited_pathlen() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let config_path = temp.path().join("ca.conf");
    let private_path = temp.path().join("ca.key");
    let public_path = temp.path().join("ca.crt");

    generate_tls_root(
      "Test Root CA",
      "Test Org",
      &config_path,
      &private_path,
      &public_path,
      -1, // Unlimited pathlen
      3650,
      true,
    )?;

    // Check config content for unlimited pathlen
    let config_content = std::fs::read_to_string(&config_path)?;
    assert!(config_content.contains("basicConstraints = critical,CA:true"));
    assert!(!config_content.contains("pathlen:"));

    Ok(())
  }

  #[test]
  fn test_generate_tls_root_no_renew() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let config_path = temp.path().join("ca.conf");
    let private_path = temp.path().join("ca.key");
    let public_path = temp.path().join("ca.crt");

    // Pre-create files
    std::fs::write(&config_path, "existing_config")?;
    std::fs::write(&private_path, "existing_private")?;
    std::fs::write(&public_path, "existing_public")?;

    // Generate with renew=false should not overwrite
    generate_tls_root(
      "Test Root CA",
      "Test Org",
      &config_path,
      &private_path,
      &public_path,
      1,
      3650,
      false,
    )?;

    assert_eq!(std::fs::read_to_string(&config_path)?, "existing_config");
    assert_eq!(std::fs::read_to_string(&private_path)?, "existing_private");
    assert_eq!(std::fs::read_to_string(&public_path)?, "existing_public");

    Ok(())
  }

  #[test]
  fn test_generate_tls_root_renew() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let config_path = temp.path().join("ca.conf");
    let private_path = temp.path().join("ca.key");
    let public_path = temp.path().join("ca.crt");

    // Pre-create files
    std::fs::write(&config_path, "existing_config")?;
    std::fs::write(&private_path, "existing_private")?;
    std::fs::write(&public_path, "existing_public")?;

    // Generate with renew=true should overwrite
    generate_tls_root(
      "Test Root CA",
      "Test Org",
      &config_path,
      &private_path,
      &public_path,
      1,
      3650,
      true,
    )?;

    let config_content = std::fs::read_to_string(&config_path)?;
    assert!(config_content.contains("[req]"));

    let private_content = std::fs::read_to_string(&private_path)?;
    assert!(private_content.contains("BEGIN PRIVATE KEY"));

    let cert_content = std::fs::read_to_string(&public_path)?;
    assert!(cert_content.contains("BEGIN CERTIFICATE"));

    Ok(())
  }
}
