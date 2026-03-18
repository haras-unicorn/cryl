use std::path::Path;

use crate::common::{
  build_basic_constraints, build_intermediary_final_config,
  build_intermediary_request_config, generate_csr, generate_private_key,
  save_private_key, save_public_file, should_skip_generation, sign_certificate,
  CrylResult, TlsAlgorithm,
};

/// Generate a TLS Intermediate CA (key + CSR + signed cert) using EC algorithm
///
/// # Arguments
/// * `common_name` - Common Name for the Intermediate CA
/// * `organization` - Organization
/// * `config` - Path to write the merged OpenSSL config (ext + req)
/// * `request_config` - Path to read base request config (will be extended)
/// * `private` - Path to save the intermediate private key
/// * `request` - Path to save the CSR
/// * `ca_public` - Root CA cert (public)
/// * `ca_private` - Root CA key (private)
/// * `serial` - Serial file to track issued cert serials
/// * `public` - Path to save the signed intermediate cert (public)
/// * `pathlen` - Allowed subordinate depth (use -1 for unlimited)
/// * `days` - Certificate validity in days
/// * `renew` - Overwrite destinations if they exist
pub fn generate_tls_intermediary(
  common_name: &str,
  organization: &str,
  config: &Path,
  request_config: &Path,
  private: &Path,
  request: &Path,
  ca_public: &Path,
  ca_private: &Path,
  serial: &Path,
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

  // Create request config
  let request_config_content =
    build_intermediary_request_config(common_name, organization);
  save_public_file(request_config, &request_config_content, renew)?;

  // Create final config by merging request config with CA extensions
  let final_config_content = build_intermediary_final_config(
    &request_config_content,
    &basic_constraints,
  );
  save_public_file(config, &final_config_content, renew)?;

  // Generate private key using EC algorithm
  let private_content = generate_private_key(TlsAlgorithm::Ec)?;
  save_private_key(private, &private_content, renew)?;

  // Generate CSR
  let csr_content = generate_csr(private, config)?;
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
    build_root_config, generate_private_key, generate_self_signed_cert,
    save_private_key, save_public_file, TlsAlgorithm,
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

    let private_content = generate_private_key(TlsAlgorithm::Ec)?;
    save_private_key(&ca_private, &private_content, true)?;

    let cert_content =
      generate_self_signed_cert(&ca_private, &ca_config, 3650)?;
    save_public_file(&ca_public, &cert_content, true)?;

    Ok((ca_public, ca_private))
  }

  #[test]
  fn test_generate_tls_intermediary_success() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let (ca_public, ca_private) = create_test_ca(&temp)?;

    let config_path = temp.path().join("inter.conf");
    let request_config_path = temp.path().join("inter_req.conf");
    let private_path = temp.path().join("inter.key");
    let request_path = temp.path().join("inter.csr");
    let serial_path = temp.path().join("serial");
    let public_path = temp.path().join("inter.crt");

    generate_tls_intermediary(
      "Test Intermediate CA",
      "Test Org",
      &config_path,
      &request_config_path,
      &private_path,
      &request_path,
      &ca_public,
      &ca_private,
      &serial_path,
      &public_path,
      0,
      3650,
      true,
    )?;

    // Check that all files exist
    assert!(config_path.exists());
    assert!(request_config_path.exists());
    assert!(private_path.exists());
    assert!(request_path.exists());
    assert!(serial_path.exists());
    assert!(public_path.exists());

    // Check config content
    let config_content = std::fs::read_to_string(&config_path)?;
    assert!(config_content.contains("CN = Test Intermediate CA"));
    assert!(
      config_content.contains("basicConstraints = critical,CA:true,pathlen:0")
    );
    assert!(config_content.contains("authorityKeyIdentifier"));

    // Check private key content
    let private_content = std::fs::read_to_string(&private_path)?;
    assert!(private_content.contains("BEGIN PRIVATE KEY"));

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
  fn test_generate_tls_intermediary_unlimited_pathlen() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let (ca_public, ca_private) = create_test_ca(&temp)?;

    let config_path = temp.path().join("inter.conf");
    let request_config_path = temp.path().join("inter_req.conf");
    let private_path = temp.path().join("inter.key");
    let request_path = temp.path().join("inter.csr");
    let serial_path = temp.path().join("serial");
    let public_path = temp.path().join("inter.crt");

    generate_tls_intermediary(
      "Test Intermediate CA",
      "Test Org",
      &config_path,
      &request_config_path,
      &private_path,
      &request_path,
      &ca_public,
      &ca_private,
      &serial_path,
      &public_path,
      -1, // Unlimited
      3650,
      true,
    )?;

    let config_content = std::fs::read_to_string(&config_path)?;
    assert!(config_content.contains("basicConstraints = critical,CA:true"));
    assert!(!config_content.contains("pathlen:"));

    Ok(())
  }

  #[test]
  fn test_generate_tls_intermediary_no_renew() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let (ca_public, ca_private) = create_test_ca(&temp)?;

    let config_path = temp.path().join("inter.conf");
    let request_config_path = temp.path().join("inter_req.conf");
    let private_path = temp.path().join("inter.key");
    let request_path = temp.path().join("inter.csr");
    let serial_path = temp.path().join("serial");
    let public_path = temp.path().join("inter.crt");

    // Pre-create files
    std::fs::write(&config_path, "existing")?;
    std::fs::write(&public_path, "existing")?;

    generate_tls_intermediary(
      "Test Intermediate CA",
      "Test Org",
      &config_path,
      &request_config_path,
      &private_path,
      &request_path,
      &ca_public,
      &ca_private,
      &serial_path,
      &public_path,
      0,
      3650,
      false,
    )?;

    assert_eq!(std::fs::read_to_string(&config_path)?, "existing");

    Ok(())
  }

  #[test]
  fn test_generate_tls_intermediary_renew() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let (ca_public, ca_private) = create_test_ca(&temp)?;

    let config_path = temp.path().join("inter.conf");
    let request_config_path = temp.path().join("inter_req.conf");
    let private_path = temp.path().join("inter.key");
    let request_path = temp.path().join("inter.csr");
    let serial_path = temp.path().join("serial");
    let public_path = temp.path().join("inter.crt");

    // Pre-create files
    std::fs::write(&config_path, "existing")?;
    std::fs::write(&public_path, "existing")?;

    generate_tls_intermediary(
      "Test Intermediate CA",
      "Test Org",
      &config_path,
      &request_config_path,
      &private_path,
      &request_path,
      &ca_public,
      &ca_private,
      &serial_path,
      &public_path,
      0,
      3650,
      true,
    )?;

    let cert_content = std::fs::read_to_string(&public_path)?;
    assert!(cert_content.contains("BEGIN CERTIFICATE"));

    Ok(())
  }
}
