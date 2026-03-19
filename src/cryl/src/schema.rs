use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete specification for secret generation pipeline.
/// Contains imports, generations, and exports to execute in order.
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct Specification {
  /// Import operations to run before generation.
  pub imports: Vec<Import>,
  /// Secret generation operations.
  pub generations: Vec<Generation>,
  /// Export operations to run after generation (if not dry-run).
  pub exports: Vec<Export>,
}

/// Import operation - brings existing data into the working directory.
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(tag = "importer", rename_all = "kebab-case")]
pub enum Import {
  /// Import all files from a Vault KV path.
  Vault { arguments: VaultImportArgs },
  /// Import a single file from a Vault KV path.
  VaultFile { arguments: VaultFileImportArgs },
  /// Copy a file from local filesystem.
  Copy { arguments: CopyImportArgs },
}

/// Secret generation operation.
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(tag = "generator", rename_all = "kebab-case")]
pub enum Generation {
  /// Copy file during generation phase.
  Copy { arguments: CopyGenArgs },
  /// Write text file with literal content.
  Text { arguments: TextGenArgs },
  /// Generate JSON file from structured data.
  Json { arguments: DataGenArgs },
  /// Generate YAML file from structured data.
  Yaml { arguments: DataGenArgs },
  /// Generate TOML file from structured data.
  Toml { arguments: DataGenArgs },
  /// Generate random alphanumeric identifier.
  Id { arguments: IdGenArgs },
  /// Generate random alphanumeric key.
  Key { arguments: IdGenArgs },
  /// Split a key into Shamir secret shares.
  KeySplit { arguments: KeySplitArgs },
  /// Combine Shamir shares back into a key.
  KeyCombine { arguments: KeyCombineArgs },
  /// Generate numeric PIN.
  Pin { arguments: PinGenArgs },
  /// Generate password with Argon2 hash.
  Password { arguments: PasswordGenArgs },
  /// Generate password with yescrypt hash.
  #[serde(rename = "password-crypt-3")]
  PasswordCrypt3 { arguments: PasswordGenArgs },
  /// Generate age encryption keypair.
  AgeKey { arguments: AgeKeyArgs },
  /// Generate SSH keypair (ed25519).
  SshKey { arguments: SshKeyArgs },
  /// Generate WireGuard keypair.
  WireguardKey { arguments: WireguardKeyArgs },
  /// Generate TLS Root CA certificate (EC P‑256).
  TlsRoot { arguments: TlsRootArgs },
  /// Generate TLS Intermediate CA certificate (EC P‑256).
  TlsIntermediary { arguments: TlsIntermediaryArgs },
  /// Generate TLS leaf certificate (EC P‑256).
  TlsLeaf { arguments: TlsLeafArgs },
  /// Generate TLS Root CA certificate (RSA 4096).
  TlsRsaRoot { arguments: TlsRootArgs },
  /// Generate TLS Intermediate CA certificate (RSA 4096).
  TlsRsaIntermediary { arguments: TlsIntermediaryArgs },
  /// Generate TLS leaf certificate (RSA 4096).
  TlsRsaLeaf { arguments: TlsLeafArgs },
  /// Generate OpenSSL DH parameters (2048‑bit).
  TlsDhparam { arguments: DhparamArgs },
  /// Generate Nebula CA certificate.
  NebulaCa { arguments: NebulaCaArgs },
  /// Generate Nebula node certificate.
  NebulaCert { arguments: NebulaCertArgs },
  /// Generate CockroachDB CA certificate.
  CockroachCa { arguments: CockroachCaArgs },
  /// Generate CockroachDB node certificate.
  CockroachNodeCert { arguments: CockroachNodeCertArgs },
  /// Generate CockroachDB client certificate.
  CockroachClientCert { arguments: CockroachClientCertArgs },
  /// Generate environment (.env) file from key‑value pairs.
  Env { arguments: EnvArgs },
  /// Generate file from Mustache template.
  Moustache { arguments: MoustacheArgs },
  /// Generate and execute Nushell script (requires --allow-script).
  Script { arguments: ScriptArgs },
  /// Generate SOPS‑encrypted YAML with Age recipients.
  Sops { arguments: SopsArgs },
}

/// Export operation - pushes generated secrets to external systems.
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(tag = "exporter", rename_all = "kebab-case")]
pub enum Export {
  /// Export all files in current directory to Vault KV path.
  Vault { arguments: VaultExportArgs },
  /// Export a single file to Vault KV path.
  VaultFile { arguments: VaultFileExportArgs },
  /// Copy file to local filesystem.
  Copy { arguments: CopyExportArgs },
}

/// Arguments for Vault KV import.
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct VaultImportArgs {
  /// Vault KV path to import from (e.g., "kv/my‑app").
  pub path: String,
  /// If true, missing source does not cause failure.
  pub allow_fail: Option<bool>,
}

/// Arguments for single‑file Vault KV import.
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct VaultFileImportArgs {
  /// Vault KV path to import from.
  pub path: String,
  /// Key name of the file within the Vault secret.
  pub file: String,
  /// If true, missing source does not cause failure.
  pub allow_fail: Option<bool>,
}

/// Arguments for file‑copy import.
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct CopyImportArgs {
  /// Source file path.
  pub from: String,
  /// Destination file path.
  pub to: String,
  /// If true, missing source does not cause failure.
  pub allow_fail: Option<bool>,
}

/// Arguments for file‑copy generation.
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct CopyGenArgs {
  /// Source file path.
  pub from: String,
  /// Destination file path.
  pub to: String,
  /// Overwrite destination if it already exists.
  pub renew: Option<bool>,
}

/// Arguments for text‑file generation.
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct TextGenArgs {
  /// Destination file name.
  pub name: String,
  /// Literal text content to write.
  pub text: String,
  /// Overwrite destination if it already exists.
  pub renew: Option<bool>,
}

/// Arguments for structured‑data generation (JSON/YAML/TOML).
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct DataGenArgs {
  /// Destination file name.
  pub name: String,
  /// Structured data to serialize.
  pub value: serde_json::Value,
  /// Overwrite destination if it already exists.
  pub renew: Option<bool>,
}

/// Arguments for random identifier/key generation.
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct IdGenArgs {
  /// Destination file name.
  pub name: String,
  /// Length of alphanumeric string (default: id=16, key=32).
  pub length: Option<u32>,
  /// Overwrite destination if it already exists.
  pub renew: Option<bool>,
}

/// Arguments for Shamir secret‑sharing split.
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct KeySplitArgs {
  /// Path to source key file (raw content).
  pub key: String,
  /// Filename prefix for each share (e.g., "share‑0", "share‑1").
  pub prefix: String,
  /// Total number of shares to generate.
  pub shares: u32,
  /// Minimum shares required to reconstruct.
  pub threshold: u32,
  /// Overwrite existing share files.
  pub renew: Option<bool>,
}

/// Arguments for Shamir secret‑sharing combine.
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct KeyCombineArgs {
  /// Path to save reconstructed key.
  pub key: String,
  /// List of share file paths to combine.
  pub shares: Vec<String>,
  /// Required shares (must match original split threshold).
  pub threshold: u32,
  /// Overwrite destination if it already exists.
  pub renew: Option<bool>,
}

/// Arguments for numeric PIN generation.
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct PinGenArgs {
  /// Destination file name.
  pub name: String,
  /// Number of digits (default: 8).
  pub length: Option<u32>,
  /// Overwrite destination if it already exists.
  pub renew: Option<bool>,
}

/// Arguments for password generation (Argon2 or yescrypt).
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct PasswordGenArgs {
  /// Path to save the hashed (public) password.
  pub public: String,
  /// Path to save the plaintext (private) password.
  pub private: String,
  /// Password length in characters (default: 8).
  pub length: Option<u32>,
  /// Overwrite both public and private files if they exist.
  pub renew: Option<bool>,
}

/// Arguments for Age encryption keypair generation.
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct AgeKeyArgs {
  /// Path to save the public key.
  pub public: String,
  /// Path to save the private key.
  pub private: String,
  /// Overwrite both key files if they exist.
  pub renew: Option<bool>,
}

/// Arguments for SSH keypair generation (ed25519).
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct SshKeyArgs {
  /// Key comment (e.g., email or hostname).
  pub name: String,
  /// Path to save the public key.
  pub public: String,
  /// Path to save the private key.
  pub private: String,
  /// Overwrite both key files if they exist.
  pub renew: Option<bool>,
}

/// Arguments for WireGuard keypair generation.
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct WireguardKeyArgs {
  /// Path to save the public key.
  pub public: String,
  /// Path to save the private key.
  pub private: String,
  /// Overwrite both key files if they exist.
  pub renew: Option<bool>,
}

/// Common arguments for TLS Root CA generation (EC or RSA).
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct TlsRootArgs {
  /// Common Name for the CA (e.g., "Sarah Root CA").
  pub common_name: String,
  /// Organization name (e.g., "Green Energy Devs").
  pub organization: String,
  /// Path to write OpenSSL configuration file.
  pub config: String,
  /// Path to save the private key.
  pub private: String,
  /// Path to save the self‑signed certificate.
  pub public: String,
  /// Path length constraint (-1 for unlimited, default: 1).
  pub pathlen: Option<i32>,
  /// Certificate validity in days (default: 3650 ≈ 10 years).
  pub days: Option<u32>,
  /// Overwrite config, key, and certificate files.
  pub renew: Option<bool>,
}

/// Arguments for TLS Intermediate CA generation.
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct TlsIntermediaryArgs {
  #[serde(flatten)]
  /// Common Root CA arguments.
  pub root: TlsRootArgs,
  /// Path to the issuing CA certificate.
  pub ca_public: String,
  /// Path to the issuing CA private key.
  pub ca_private: String,
  /// Path to save the Certificate Signing Request.
  pub request: String,
  /// Path to write the CSR‑only OpenSSL config.
  pub request_config: String,
  /// Path to the serial number tracking file.
  pub serial: String,
}

/// Arguments for TLS leaf certificate generation.
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct TlsLeafArgs {
  #[serde(flatten)]
  /// Intermediate CA arguments plus SANs.
  pub inter: TlsIntermediaryArgs,
  /// Subject Alternative Names (SANs) as comma‑separated list.
  pub sans: Vec<String>,
}

/// Arguments for OpenSSL DH parameters generation.
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct DhparamArgs {
  /// Path to save the DH parameters file.
  pub name: String,
  /// Overwrite destination if it exists.
  pub renew: Option<bool>,
}

/// Arguments for Nebula CA certificate generation.
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct NebulaCaArgs {
  /// Common Name for the Nebula CA.
  pub name: String,
  /// Path to save the CA certificate.
  pub public: String,
  /// Path to save the CA private key.
  pub private: String,
  /// Certificate validity in days (default: 3650).
  pub days: Option<u32>,
  /// Overwrite both CA files.
  pub renew: Option<bool>,
}

/// Arguments for Nebula node certificate generation.
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct NebulaCertArgs {
  /// Path to the Nebula CA certificate.
  pub ca_public: String,
  /// Path to the Nebula CA private key.
  pub ca_private: String,
  /// Common Name for the node certificate.
  pub name: String,
  /// Node IP in CIDR or plain IP form (e.g., "10.1.1.5/24").
  pub ip: String,
  /// Path to save the node certificate.
  pub public: String,
  /// Path to save the node private key.
  pub private: String,
  /// Overwrite both certificate and key files.
  pub renew: Option<bool>,
}

/// Arguments for CockroachDB CA certificate generation.
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct CockroachCaArgs {
  /// Path to save the CA certificate.
  pub public: String,
  /// Path to save the CA private key.
  pub private: String,
  /// Overwrite both CA files.
  pub renew: Option<bool>,
}

/// Arguments for CockroachDB node certificate generation.
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct CockroachNodeCertArgs {
  /// Path to the CockroachDB CA certificate.
  pub ca_public: String,
  /// Path to the CockroachDB CA private key.
  pub ca_private: String,
  /// Comma‑separated hostnames/IPs for Subject Alternative Names.
  pub hosts: Vec<String>,
  /// Path to save the node certificate.
  pub public: String,
  /// Path to save the node private key.
  pub private: String,
  /// Overwrite both certificate and key files.
  pub renew: Option<bool>,
}

/// Arguments for CockroachDB client certificate generation.
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct CockroachClientCertArgs {
  /// Path to the CockroachDB CA certificate.
  pub ca_public: String,
  /// Path to the CockroachDB CA private key.
  pub ca_private: String,
  /// CockroachDB username for the client certificate.
  pub user: String,
  /// Path to save the client certificate.
  pub public: String,
  /// Path to save the client private key.
  pub private: String,
  /// Overwrite both certificate and key files.
  pub renew: Option<bool>,
}

/// Arguments for environment (.env) file generation.
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct EnvArgs {
  /// Destination file path.
  pub name: String,
  /// Key‑value pairs; values can be strings or file paths.
  pub variables: HashMap<String, String>,
  /// Overwrite destination if it exists.
  pub renew: Option<bool>,
}

/// Arguments for Mustache template generation.
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct MoustacheArgs {
  /// Base name for output files (‑variables, ‑template suffixes added).
  pub name: String,
  /// Mustache template content.
  pub template: String,
  /// Variables to substitute; values can be strings or file paths.
  pub variables: HashMap<String, String>,
  /// Overwrite output file if it exists.
  pub renew: Option<bool>,
}

/// Arguments for Nushell script generation and execution.
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct ScriptArgs {
  /// Path to save the script.
  pub name: String,
  /// Nushell script content.
  pub text: String,
  /// Overwrite script file if it exists.
  pub renew: Option<bool>,
}

/// Arguments for SOPS‑encrypted secret generation.
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct SopsArgs {
  /// Path to file containing Age recipient(s).
  pub age: String,
  /// Path to save encrypted YAML (public).
  pub public: String,
  /// Path to save plaintext YAML (private).
  pub private: String,
  /// Secret key‑value pairs; values can be strings or file paths.
  pub secrets: serde_json::Value,
  /// Overwrite both public and private files.
  pub renew: Option<bool>,
}

/// Arguments for Vault KV export.
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct VaultExportArgs {
  /// Base Vault KV path (e.g., "kv/my‑app").
  pub path: String,
}

/// Arguments for single‑file Vault KV export.
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct VaultFileExportArgs {
  /// Base Vault KV path.
  pub path: String,
  /// Local file to export (key becomes filename).
  pub file: String,
}

/// Arguments for file‑copy export.
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct CopyExportArgs {
  /// Source file path.
  pub from: String,
  /// Destination file path.
  pub to: String,
}
