use std::path::Path;

use crate::common::{
  CrylResult, TlsAlgorithm, build_leaf_final_config, build_leaf_request_config,
  generate_csr, generate_private_key, parse_sans, save_private_key,
  save_public_file, should_skip_generation, sign_certificate,
};

/// Generate a TLS Leaf certificate (key + CSR + signed cert) using RSA algorithm
///
/// # Arguments
/// * `common_name` - Common Name for the certificate
/// * `organization` - Organization
/// * `sans` - Comma-separated SANs (e.g., "example.com,www.example.com,10.0.0.1")
/// * `config` - Path to write the merged OpenSSL config
/// * `request_config` - Path to write the request config
/// * `private` - Path to save the private key
/// * `request` - Path to save the CSR
/// * `ca_public` - Issuer cert (Intermediate or Root)
/// * `ca_private` - Issuer key (matching private key)
/// * `serial` - Serial file to track issued cert serials
/// * `public` - Path to save the signed leaf certificate
/// * `days` - Certificate validity in days
/// * `renew` - Overwrite destinations if they exist
pub fn generate_tls_rsa_leaf(
  common_name: &str,
  organization: &str,
  sans: &str,
  config: &Path,
  request_config: &Path,
  private: &Path,
  request: &Path,
  ca_public: &Path,
  ca_private: &Path,
  serial: &Path,
  public: &Path,
  days: u32,
  renew: bool,
) -> CrylResult<()> {
  // If public cert exists and we're not renewing, skip everything
  if should_skip_generation(public, renew) {
    return Ok(());
  }

  // Parse SANs
  let (dns_sans, ip_sans) = parse_sans(sans);

  // Get key usage for RSA algorithm
  let key_usage = TlsAlgorithm::Rsa.leaf_key_usage();

  // Create request config
  let request_config_content = build_leaf_request_config(
    common_name,
    organization,
    &dns_sans,
    &ip_sans,
    key_usage,
  );
  save_public_file(request_config, &request_config_content, renew)?;

  // Create final config
  let final_config_content = build_leaf_final_config(&request_config_content);
  save_public_file(config, &final_config_content, renew)?;

  // Generate private key using RSA algorithm
  let private_content = generate_private_key(TlsAlgorithm::Rsa)?;
  save_private_key(private, &private_content, renew)?;

  // Generate CSR using request_config
  let csr_content = generate_csr(private, request_config)?;
  save_public_file(request, &csr_content, renew)?;

  // Sign certificate with CA
  let (cert_content, serial_content) =
    sign_certificate(request, ca_public, ca_private, serial, config, days)?;

  // Save signed certificate
  save_public_file(public, &cert_content, renew)?;

  // Save serial file
  save_public_file(serial, &serial_content, renew)?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::common::{
    TlsAlgorithm, build_basic_constraints, build_root_config,
    generate_private_key, generate_self_signed_cert, save_private_key,
    save_public_file,
  };
  use std::os::unix::fs::PermissionsExt;
  use tempfile::TempDir;

  fn create_test_ca(
    temp: &TempDir,
  ) -> anyhow::Result<(std::path::PathBuf, std::path::PathBuf)> {
    let ca_config = temp.path().join("ca.conf");
    let ca_private = temp.path().join("ca.key");
    let ca_public = temp.path().join("ca.crt");

    let basic_constraints = build_basic_constraints(1);
    let config_content =
      build_root_config("Test Root CA", "Test Org", &basic_constraints);
    save_public_file(&ca_config, &config_content, true)?;

    let private_content = generate_private_key(TlsAlgorithm::Rsa)?;
    save_private_key(&ca_private, &private_content, true)?;

    let cert_content =
      generate_self_signed_cert(&ca_private, &ca_config, 3650)?;
    save_public_file(&ca_public, &cert_content, true)?;

    Ok((ca_public, ca_private))
  }

  #[test]
  fn test_generate_tls_rsa_leaf_success() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let (ca_public, ca_private) = create_test_ca(&temp)?;

    let config_path = temp.path().join("leaf.conf");
    let request_config_path = temp.path().join("leaf_req.conf");
    let private_path = temp.path().join("leaf.key");
    let request_path = temp.path().join("leaf.csr");
    let serial_path = temp.path().join("serial");
    let public_path = temp.path().join("leaf.crt");

    generate_tls_rsa_leaf(
      "example.com",
      "Test Org",
      "example.com,www.example.com,10.0.0.1",
      &config_path,
      &request_config_path,
      &private_path,
      &request_path,
      &ca_public,
      &ca_private,
      &serial_path,
      &public_path,
      365,
      true,
    )?;

    // Check that all files exist
    assert!(config_path.exists());
    assert!(request_config_path.exists());
    assert!(private_path.exists());
    assert!(request_path.exists());
    assert!(serial_path.exists());
    assert!(public_path.exists());

    // Check request config content
    let request_config_content = std::fs::read_to_string(&request_config_path)?;
    assert!(request_config_content.contains("CN = example.com"));
    assert!(request_config_content.contains("DNS.1 = example.com"));
    assert!(request_config_content.contains("DNS.2 = www.example.com"));
    assert!(request_config_content.contains("IP.1 = 10.0.0.1"));
    assert!(
      request_config_content
        .contains("extendedKeyUsage = serverAuth,clientAuth")
    );
    // RSA has different key usage
    assert!(
      request_config_content
        .contains("critical,digitalSignature,keyEncipherment")
    );

    // Check final config
    let config_content = std::fs::read_to_string(&config_path)?;
    assert!(config_content.contains("basicConstraints = critical,CA:false"));
    assert!(config_content.contains("authorityKeyIdentifier"));

    // Check private key content (RSA)
    let private_content = std::fs::read_to_string(&private_path)?;
    assert!(
      private_content.contains("BEGIN PRIVATE KEY")
        || private_content.contains("BEGIN RSA PRIVATE KEY")
    );

    // Check CSR content
    let csr_content = std::fs::read_to_string(&request_path)?;
    assert!(csr_content.contains("BEGIN CERTIFICATE REQUEST"));

    // Check certificate content
    let cert_content = std::fs::read_to_string(&public_path)?;
    assert!(cert_content.contains("BEGIN CERTIFICATE"));

    // Check permissions
    assert_eq!(
      std::fs::metadata(&private_path)?.permissions().mode() & 0o777,
      0o600
    );
    assert_eq!(
      std::fs::metadata(&public_path)?.permissions().mode() & 0o777,
      0o644
    );

    Ok(())
  }

  #[test]
  fn test_generate_tls_rsa_leaf_dns_only() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let (ca_public, ca_private) = create_test_ca(&temp)?;

    let config_path = temp.path().join("leaf.conf");
    let request_config_path = temp.path().join("leaf_req.conf");
    let private_path = temp.path().join("leaf.key");
    let request_path = temp.path().join("leaf.csr");
    let serial_path = temp.path().join("serial");
    let public_path = temp.path().join("leaf.crt");

    generate_tls_rsa_leaf(
      "example.com",
      "Test Org",
      "example.com,www.example.com",
      &config_path,
      &request_config_path,
      &private_path,
      &request_path,
      &ca_public,
      &ca_private,
      &serial_path,
      &public_path,
      365,
      true,
    )?;

    let request_config_content = std::fs::read_to_string(&request_config_path)?;
    assert!(request_config_content.contains("DNS.1 = example.com"));
    assert!(request_config_content.contains("DNS.2 = www.example.com"));
    assert!(!request_config_content.contains("IP.1"));

    Ok(())
  }

  #[test]
  fn test_generate_tls_rsa_leaf_ip_only() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let (ca_public, ca_private) = create_test_ca(&temp)?;

    let config_path = temp.path().join("leaf.conf");
    let request_config_path = temp.path().join("leaf_req.conf");
    let private_path = temp.path().join("leaf.key");
    let request_path = temp.path().join("leaf.csr");
    let serial_path = temp.path().join("serial");
    let public_path = temp.path().join("leaf.crt");

    generate_tls_rsa_leaf(
      "10.0.0.1",
      "Test Org",
      "10.0.0.1,192.168.1.1",
      &config_path,
      &request_config_path,
      &private_path,
      &request_path,
      &ca_public,
      &ca_private,
      &serial_path,
      &public_path,
      365,
      true,
    )?;

    let request_config_content = std::fs::read_to_string(&request_config_path)?;
    assert!(request_config_content.contains("IP.1 = 10.0.0.1"));
    assert!(request_config_content.contains("IP.2 = 192.168.1.1"));
    assert!(!request_config_content.contains("DNS.1"));

    Ok(())
  }

  #[test]
  fn test_generate_tls_rsa_leaf_no_renew() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let (ca_public, ca_private) = create_test_ca(&temp)?;

    let config_path = temp.path().join("leaf.conf");
    let request_config_path = temp.path().join("leaf_req.conf");
    let private_path = temp.path().join("leaf.key");
    let request_path = temp.path().join("leaf.csr");
    let serial_path = temp.path().join("serial");
    let public_path = temp.path().join("leaf.crt");

    // Pre-create files
    std::fs::write(&config_path, "existing")?;
    std::fs::write(&public_path, "existing")?;

    generate_tls_rsa_leaf(
      "example.com",
      "Test Org",
      "example.com",
      &config_path,
      &request_config_path,
      &private_path,
      &request_path,
      &ca_public,
      &ca_private,
      &serial_path,
      &public_path,
      365,
      false,
    )?;

    assert_eq!(std::fs::read_to_string(&config_path)?, "existing");

    Ok(())
  }

  #[test]
  fn test_generate_tls_rsa_leaf_renew() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let (ca_public, ca_private) = create_test_ca(&temp)?;

    let config_path = temp.path().join("leaf.conf");
    let request_config_path = temp.path().join("leaf_req.conf");
    let private_path = temp.path().join("leaf.key");
    let request_path = temp.path().join("leaf.csr");
    let serial_path = temp.path().join("serial");
    let public_path = temp.path().join("leaf.crt");

    // Pre-create files
    std::fs::write(&config_path, "existing")?;
    std::fs::write(&public_path, "existing")?;

    generate_tls_rsa_leaf(
      "example.com",
      "Test Org",
      "example.com",
      &config_path,
      &request_config_path,
      &private_path,
      &request_path,
      &ca_public,
      &ca_private,
      &serial_path,
      &public_path,
      365,
      true,
    )?;

    let cert_content = std::fs::read_to_string(&public_path)?;
    assert!(cert_content.contains("BEGIN CERTIFICATE"));

    Ok(())
  }
}
