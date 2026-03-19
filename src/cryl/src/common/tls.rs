//! Common TLS generation utilities

use std::path::Path;
use std::process::Command;

use crate::common::{CrylError, CrylResult, save_atomic};

/// TLS key algorithm types
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TlsAlgorithm {
  /// EC (Elliptic Curve) with prime256v1 curve
  Ec,
  /// RSA with 4096 bits
  Rsa,
}

impl TlsAlgorithm {
  /// Get the algorithm name for OpenSSL
  pub fn name(&self) -> &'static str {
    match self {
      TlsAlgorithm::Ec => "EC",
      TlsAlgorithm::Rsa => "RSA",
    }
  }

  /// Get the pkeyopt option for OpenSSL
  pub fn pkeyopt(&self) -> &'static str {
    match self {
      TlsAlgorithm::Ec => "ec_paramgen_curve:prime256v1",
      TlsAlgorithm::Rsa => "rsa_keygen_bits:4096",
    }
  }

  /// Get key usage for leaf certificates
  pub fn leaf_key_usage(&self) -> &'static str {
    match self {
      TlsAlgorithm::Ec => "critical,digitalSignature",
      TlsAlgorithm::Rsa => "critical,digitalSignature,keyEncipherment",
    }
  }
}

/// Build basicConstraints string for CA certificates
pub fn build_basic_constraints(pathlen: i32) -> String {
  if pathlen < 0 {
    "critical,CA:true".to_string()
  } else {
    format!("critical,CA:true,pathlen:{}", pathlen)
  }
}

/// Generate a private key using the specified algorithm
pub fn generate_private_key(algorithm: TlsAlgorithm) -> CrylResult<String> {
  let output = Command::new("openssl")
    .arg("genpkey")
    .arg("-algorithm")
    .arg(algorithm.name())
    .arg("-pkeyopt")
    .arg(algorithm.pkeyopt())
    .arg("-quiet")
    .output()?;

  if !output.status.success() {
    return Err(CrylError::ToolExecution {
      tool: "openssl genpkey".to_string(),
      exit_code: output.status.code().unwrap_or(-1),
      stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    });
  }

  Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Save a private key to file
pub fn save_private_key(
  path: &Path,
  content: &str,
  renew: bool,
) -> CrylResult<()> {
  save_atomic(path, content.as_bytes(), renew, false)
}

/// Save a public certificate or config to file
pub fn save_public_file(
  path: &Path,
  content: &str,
  renew: bool,
) -> CrylResult<()> {
  save_atomic(path, content.as_bytes(), renew, true)
}

/// Generate a self-signed certificate (for Root CA)
pub fn generate_self_signed_cert(
  private_key_path: &Path,
  config_path: &Path,
  days: u32,
) -> CrylResult<String> {
  let output = Command::new("openssl")
    .arg("req")
    .arg("-x509")
    .arg("-key")
    .arg(private_key_path)
    .arg("-config")
    .arg(config_path)
    .arg("-days")
    .arg(days.to_string())
    .output()?;

  if !output.status.success() {
    return Err(CrylError::ToolExecution {
      tool: "openssl req -x509".to_string(),
      exit_code: output.status.code().unwrap_or(-1),
      stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    });
  }

  Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Generate a Certificate Signing Request (CSR)
pub fn generate_csr(
  private_key_path: &Path,
  config_path: &Path,
) -> CrylResult<String> {
  let output = Command::new("openssl")
    .arg("req")
    .arg("-new")
    .arg("-key")
    .arg(private_key_path)
    .arg("-config")
    .arg(config_path)
    .arg("-quiet")
    .output()?;

  if !output.status.success() {
    return Err(CrylError::ToolExecution {
      tool: "openssl req -new".to_string(),
      exit_code: output.status.code().unwrap_or(-1),
      stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    });
  }

  Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Sign a certificate with a CA
pub fn sign_certificate(
  csr_path: &Path,
  ca_cert_path: &Path,
  ca_key_path: &Path,
  serial_path: &Path,
  config_path: &Path,
  days: u32,
) -> CrylResult<(String, String)> {
  let _tmp_suffix = ".tmp";
  let tmp_serial_path = serial_path.with_extension("tmp");

  // Determine serial args
  let serial_args: Vec<String> = if serial_path.exists() {
    // Copy existing serial file
    std::fs::copy(serial_path, &tmp_serial_path)?;
    vec![
      "-CAserial".to_string(),
      tmp_serial_path.to_string_lossy().to_string(),
    ]
  } else {
    vec![
      "-CAcreateserial".to_string(),
      "-CAserial".to_string(),
      tmp_serial_path.to_string_lossy().to_string(),
    ]
  };

  let output = Command::new("openssl")
    .arg("x509")
    .arg("-req")
    .arg("-in")
    .arg(csr_path)
    .arg("-CA")
    .arg(ca_cert_path)
    .arg("-CAkey")
    .arg(ca_key_path)
    .args(&serial_args)
    .arg("-extfile")
    .arg(config_path)
    .arg("-extensions")
    .arg("ext")
    .arg("-days")
    .arg(days.to_string())
    .output()?;

  if !output.status.success() {
    // Clean up temp serial file on failure
    let _ = std::fs::remove_file(&tmp_serial_path);
    return Err(CrylError::ToolExecution {
      tool: "openssl x509 -req".to_string(),
      exit_code: output.status.code().unwrap_or(-1),
      stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    });
  }

  let cert_content = String::from_utf8_lossy(&output.stdout).to_string();

  // Read the serial file content
  let serial_content = std::fs::read_to_string(&tmp_serial_path)?;

  // Clean up temp serial file
  let _ = std::fs::remove_file(&tmp_serial_path);

  Ok((cert_content, serial_content))
}

/// Build root CA config content
pub fn build_root_config(
  common_name: &str,
  organization: &str,
  basic_constraints: &str,
) -> String {
  format!(
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
  )
}

/// Build intermediary CA request config (base config)
pub fn build_intermediary_request_config(
  common_name: &str,
  organization: &str,
) -> String {
  format!(
    r#"[req]
default_md = sha256
distinguished_name = dn
x509_extensions = ext
prompt = no

[dn]
CN = {}
O = {}

[ext]
keyUsage = critical,keyCertSign,cRLSign
subjectKeyIdentifier = hash
"#,
    common_name, organization
  )
}

/// Build intermediary CA final config by appending CA-specific extensions
pub fn build_intermediary_final_config(
  request_config: &str,
  basic_constraints: &str,
) -> String {
  format!(
    r#"{}
basicConstraints = {}
authorityKeyIdentifier = keyid,issuer
"#,
    request_config, basic_constraints
  )
}

/// Check if we should skip generation (files exist and no renew)
pub fn should_skip_generation(public_path: &Path, renew: bool) -> bool {
  !renew && public_path.exists()
}

/// Check if a string is a valid IP address (simple check)
pub fn is_ip_address(s: &str) -> bool {
  s.parse::<std::net::IpAddr>().is_ok()
}

/// Parse SANs (Subject Alternative Names) from comma-separated string
/// Returns (dns_sans, ip_sans)
pub fn parse_sans(sans: &str) -> (Vec<String>, Vec<String>) {
  let sans_list: Vec<&str> = sans.split(',').map(str::trim).collect();

  let mut dns_sans = Vec::new();
  let mut ip_sans = Vec::new();

  for san in sans_list {
    if san.is_empty() {
      continue;
    }
    if is_ip_address(san) {
      ip_sans.push(san.to_string());
    } else {
      dns_sans.push(san.to_string());
    }
  }

  (dns_sans, ip_sans)
}

/// Build leaf certificate request config
pub fn build_leaf_request_config(
  common_name: &str,
  organization: &str,
  dns_sans: &[String],
  ip_sans: &[String],
  key_usage: &str,
) -> String {
  let dns_san_lines: String = dns_sans
    .iter()
    .enumerate()
    .map(|(i, san)| format!("DNS.{} = {}", i.saturating_add(1), san))
    .collect::<Vec<_>>()
    .join("\n");

  let ip_san_lines: String = ip_sans
    .iter()
    .enumerate()
    .map(|(i, ip)| format!("IP.{} = {}", i.saturating_add(1), ip))
    .collect::<Vec<_>>()
    .join("\n");

  format!(
    r#"[req]
default_md = sha256
distinguished_name = dn
req_extensions = ext
prompt = no

[dn]
CN = {}
O = {}

[sans]
{}
{}

[ext]
keyUsage = {}
extendedKeyUsage = serverAuth,clientAuth
subjectAltName = @sans
subjectKeyIdentifier = hash
"#,
    common_name, organization, dns_san_lines, ip_san_lines, key_usage
  )
}

/// Build leaf certificate final config
pub fn build_leaf_final_config(request_config: &str) -> String {
  format!(
    r#"{}
basicConstraints = critical,CA:false
authorityKeyIdentifier = keyid,issuer
"#,
    request_config
  )
}
