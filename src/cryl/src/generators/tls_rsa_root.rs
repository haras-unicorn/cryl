use std::path::Path;

use crate::common::{
  CrylResult, TlsAlgorithm, build_basic_constraints, build_root_config,
  generate_private_key, generate_self_signed_cert, save_private_key,
  save_public_file, should_skip_generation,
};

/// Generate a TLS Root CA (private key + self-signed certificate) using RSA
/// algorithm
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
pub fn generate_tls_rsa_root(
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
  if should_skip_generation(public, renew) {
    return Ok(());
  }

  // Build basicConstraints based on pathlen
  let basic_constraints = build_basic_constraints(pathlen);

  // Create and save OpenSSL config
  let config_content =
    build_root_config(common_name, organization, &basic_constraints);
  save_public_file(config, &config_content, renew)?;

  // Generate private key using RSA algorithm
  let private_content = generate_private_key(TlsAlgorithm::Rsa)?;
  save_private_key(private, &private_content, renew)?;

  // Generate self-signed certificate using the config
  let cert_content = generate_self_signed_cert(private, config, days)?;
  save_public_file(public, &cert_content, renew)?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::os::unix::fs::PermissionsExt;
  use tempfile::TempDir;

  #[test]
  fn test_generate_tls_rsa_root_success() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let config_path = temp.path().join("ca.conf");
    let private_path = temp.path().join("ca.key");
    let public_path = temp.path().join("ca.crt");

    generate_tls_rsa_root(
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

    // Check private key content (RSA keys are different format)
    let private_content = std::fs::read_to_string(&private_path)?;
    assert!(
      private_content.contains("BEGIN PRIVATE KEY")
        || private_content.contains("BEGIN RSA PRIVATE KEY")
    );

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
  fn test_generate_tls_rsa_root_unlimited_pathlen() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let config_path = temp.path().join("ca.conf");
    let private_path = temp.path().join("ca.key");
    let public_path = temp.path().join("ca.crt");

    generate_tls_rsa_root(
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
  fn test_generate_tls_rsa_root_no_renew() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let config_path = temp.path().join("ca.conf");
    let private_path = temp.path().join("ca.key");
    let public_path = temp.path().join("ca.crt");

    // Pre-create files
    std::fs::write(&config_path, "existing_config")?;
    std::fs::write(&private_path, "existing_private")?;
    std::fs::write(&public_path, "existing_public")?;

    // Generate with renew=false should not overwrite
    generate_tls_rsa_root(
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
  fn test_generate_tls_rsa_root_renew() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let config_path = temp.path().join("ca.conf");
    let private_path = temp.path().join("ca.key");
    let public_path = temp.path().join("ca.crt");

    // Pre-create files
    std::fs::write(&config_path, "existing_config")?;
    std::fs::write(&private_path, "existing_private")?;
    std::fs::write(&public_path, "existing_public")?;

    // Generate with renew=true should overwrite
    generate_tls_rsa_root(
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
    assert!(
      private_content.contains("BEGIN PRIVATE KEY")
        || private_content.contains("BEGIN RSA PRIVATE KEY")
    );

    let cert_content = std::fs::read_to_string(&public_path)?;
    assert!(cert_content.contains("BEGIN CERTIFICATE"));

    Ok(())
  }
}
