//! CLI argument parsing for cryl

use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

/// cryl - Secret generation tool
///
/// A high-performance, sandboxed CLI tool for generating, encrypting, and
/// managing infrastructure secrets.
#[derive(Parser, Debug)]
#[command(name = "cryl")]
#[command(about = "Secret generation tool")]
#[command(version)]
pub struct Cli {
  #[command(subcommand)]
  pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
  /// Load specification from file path
  #[command(name = "path", visible_alias = "from-path")]
  Path {
    /// Path to specification file
    spec: PathBuf,
    #[command(flatten)]
    common: CommonArgs,
    #[command(flatten)]
    sandbox: SandboxArgs,
  },

  /// Load specification from stdin
  #[command(name = "stdin", visible_alias = "from-stdin")]
  Stdin {
    /// Format of the specification (json, yaml, toml)
    format: String,
    #[command(flatten)]
    common: CommonArgs,
    #[command(flatten)]
    sandbox: SandboxArgs,
  },

  /// Print JSON schema to stdout
  Schema,

  /// Import commands (direct execution, non-sandboxed)
  #[command(subcommand)]
  Import(ImportCommands),

  /// Generate commands (direct execution, non-sandboxed)
  #[command(subcommand)]
  Generate(GenerateCommands),

  /// Export commands (direct execution, non-sandboxed)
  #[command(subcommand)]
  Export(ExportCommands),
}

#[derive(Args, Debug, Clone)]
pub struct CommonArgs {
  /// Don't run exports
  #[arg(long)]
  pub dry_run: bool,

  /// Allow script generator
  #[arg(long)]
  pub allow_script: bool,

  /// Maximum allowed imports
  #[arg(long, default_value = "1024")]
  pub max_imports: usize,

  /// Maximum allowed generations
  #[arg(long, default_value = "1024")]
  pub max_generations: usize,

  /// Maximum allowed exports
  #[arg(long, default_value = "1024")]
  pub max_exports: usize,

  /// Maximum allowed specification size in bytes
  #[arg(long, default_value = "1048576")]
  pub max_specification_size: usize,

  /// Select manifest format (json, yaml, toml)
  #[arg(long, default_value = "json")]
  pub manifest_format: String,

  /// Turn on logging from modules
  #[arg(long)]
  pub verbose: bool,

  /// Turn on logging from tools (implies verbose)
  #[arg(long)]
  pub very_verbose: bool,

  /// Stay in current working directory (non-sandboxed only)
  #[arg(long)]
  pub stay: bool,

  /// Don't remove working directory contents (non-sandboxed only)
  #[arg(long)]
  pub keep: bool,
}

#[derive(Args, Debug, Clone)]
pub struct SandboxArgs {
  /// Don't use sandbox while running
  #[arg(long)]
  pub nosandbox: bool,

  /// Additional read-only bind mounts for bubblewrap
  #[arg(long, value_delimiter = ',')]
  pub ro_binds: Vec<PathBuf>,

  /// Additional bind mounts for bubblewrap
  #[arg(long, value_delimiter = ',')]
  pub binds: Vec<PathBuf>,

  /// Additional tool binaries for bubblewrap PATH
  #[arg(long, value_delimiter = ',')]
  pub tools: Vec<String>,

  /// Allow network while running
  #[arg(long)]
  pub allow_net: bool,
}

#[derive(Subcommand, Debug)]
pub enum ImportCommands {
  /// Copy a file
  Copy {
    /// Source path
    from: PathBuf,
    /// Destination path
    to: PathBuf,
    /// Allow failing to copy if source missing
    #[arg(long)]
    allow_fail: bool,
  },

  /// Import from Vault
  Vault {
    /// Vault path to import from
    path: String,
    /// Allow failing to import if source missing
    #[arg(long)]
    allow_fail: bool,
  },

  /// Import single file from Vault
  #[command(name = "vault-file")]
  VaultFile {
    /// Vault path to import from
    path: String,
    /// File key to import
    file: String,
    /// Allow failing to import if source missing
    #[arg(long)]
    allow_fail: bool,
  },
}

#[derive(Subcommand, Debug)]
pub enum GenerateCommands {
  /// Generate random alphanumeric id
  #[command(name = "id")]
  Id {
    /// Destination file name
    name: PathBuf,
    /// Number of characters
    #[arg(long, default_value = "16")]
    length: u32,
    /// Overwrite destination if it exists
    #[arg(long)]
    renew: bool,
  },

  /// Generate random key
  #[command(name = "key")]
  Key {
    /// Destination file name
    name: PathBuf,
    /// Number of characters
    #[arg(long, default_value = "32")]
    length: u32,
    /// Overwrite destination if it exists
    #[arg(long)]
    renew: bool,
  },

  /// Generate PIN
  #[command(name = "pin")]
  Pin {
    /// Destination file name
    name: PathBuf,
    /// Number of digits
    #[arg(long, default_value = "8")]
    length: u32,
    /// Overwrite destination if it exists
    #[arg(long)]
    renew: bool,
  },

  /// Generate password
  #[command(name = "password")]
  Password {
    /// Destination file name
    name: PathBuf,
    /// Number of characters
    #[arg(long, default_value = "16")]
    length: usize,
    /// Overwrite destination if it exists
    #[arg(long)]
    renew: bool,
  },

  /// Copy a file
  Copy {
    /// Source path
    from: PathBuf,
    /// Destination path
    to: PathBuf,
    /// Overwrite destination if exists
    #[arg(long)]
    renew: bool,
  },

  /// Generate text file
  Text {
    /// Destination file name
    name: PathBuf,
    /// Text content
    text: String,
    /// Overwrite destination if exists
    #[arg(long)]
    renew: bool,
  },

  /// Convert data between formats
  Data {
    /// Destination file name
    name: PathBuf,
    /// Input format
    in_format: String,
    /// Source data path
    data: PathBuf,
    /// Output format
    out_format: String,
    /// Overwrite destination if exists
    #[arg(long)]
    renew: bool,
  },

  /// Generate password (argon2)
  #[command(name = "password-argon2")]
  PasswordArgon2 {
    /// Path for public/hashed password
    public: PathBuf,
    /// Path for private/plain password
    private: PathBuf,
    /// Password length
    #[arg(long, default_value = "8")]
    length: usize,
    /// Overwrite if exists
    #[arg(long)]
    renew: bool,
  },

  /// Generate password (yescrypt)
  #[command(name = "password-crypt-3")]
  PasswordCrypt3 {
    /// Path for public/hashed password
    public: PathBuf,
    /// Path for private/plain password
    private: PathBuf,
    /// Password length
    #[arg(long, default_value = "8")]
    length: usize,
    /// Overwrite if exists
    #[arg(long)]
    renew: bool,
  },

  /// Generate age keypair
  #[command(name = "age-key")]
  AgeKey {
    /// Public key path
    public: PathBuf,
    /// Private key path
    private: PathBuf,
    /// Overwrite if exists
    #[arg(long)]
    renew: bool,
  },

  /// Generate SSH keypair
  #[command(name = "ssh-key")]
  SshKey {
    /// Key comment (e.g., email/host)
    name: String,
    /// Public key path
    public: PathBuf,
    /// Private key path
    private: PathBuf,
    /// Passphrase file (optional)
    #[arg(long)]
    password: Option<PathBuf>,
    /// Overwrite if exists
    #[arg(long)]
    renew: bool,
  },

  /// Generate WireGuard keypair
  #[command(name = "wireguard-key")]
  WireguardKey {
    /// Private key path
    private: PathBuf,
    /// Public key path
    public: PathBuf,
    /// Overwrite if exists
    #[arg(long)]
    renew: bool,
  },

  /// Split key into Shamir shares
  #[command(name = "key-split")]
  KeySplit {
    /// Source key file
    key: PathBuf,
    /// Share filename prefix
    prefix: String,
    /// Minimum shares to reconstruct
    threshold: usize,
    /// Total shares to generate
    shares: usize,
    /// Overwrite if exists
    #[arg(long)]
    renew: bool,
  },

  /// Combine Shamir shares
  #[command(name = "key-combine")]
  KeyCombine {
    /// Comma-separated share files
    shares: String,
    /// Output key file
    key: PathBuf,
    /// Required shares (must match split threshold)
    threshold: usize,
    /// Overwrite if exists
    #[arg(long)]
    renew: bool,
  },

  /// TLS Root CA (RSA)
  #[command(name = "tls-rsa-root")]
  TlsRsaRoot {
    /// Common Name for the Root CA
    common_name: String,
    /// Organization name
    organization: String,
    /// Path to write OpenSSL config
    config: PathBuf,
    /// Path to save private key
    private: PathBuf,
    /// Path to save self-signed certificate
    public: PathBuf,
    /// Certificate path length constraint (-1 for unlimited)
    #[arg(long, default_value = "1")]
    pathlen: i32,
    /// Certificate validity in days
    #[arg(long, default_value = "3650")]
    days: u32,
    /// Overwrite destination if exists
    #[arg(long)]
    renew: bool,
  },

  /// TLS Intermediate CA (RSA)
  #[command(name = "tls-rsa-intermediary")]
  TlsRsaIntermediary {
    /// Common Name for the Intermediate CA
    common_name: String,
    /// Organization name
    organization: String,
    /// Path to write merged OpenSSL config (extensions + request)
    config: PathBuf,
    /// Path to write request config (will be created)
    request_config: PathBuf,
    /// Path to save private key
    private: PathBuf,
    /// Path to save CSR
    request: PathBuf,
    /// Issuer/CA certificate path
    ca_public: PathBuf,
    /// Issuer/CA private key path
    ca_private: PathBuf,
    /// Serial number tracking file
    serial: PathBuf,
    /// Path to save signed certificate
    public: PathBuf,
    /// Certificate path length constraint (-1 for unlimited)
    #[arg(long, default_value = "0")]
    pathlen: i32,
    /// Certificate validity in days
    #[arg(long, default_value = "3650")]
    days: u32,
    /// Overwrite destination if exists
    #[arg(long)]
    renew: bool,
  },

  /// TLS Leaf certificate (RSA)
  #[command(name = "tls-rsa-leaf")]
  TlsRsaLeaf {
    /// Common Name for certificate
    common_name: String,
    /// Organization name
    organization: String,
    /// Comma-separated Subject Alternative Names
    sans: String,
    /// Path to write merged OpenSSL config (extensions + request)
    config: PathBuf,
    /// Path to write request config (will be created)
    request_config: PathBuf,
    /// Path to save private key
    private: PathBuf,
    /// Path to save CSR
    request: PathBuf,
    /// Issuer CA certificate path
    ca_public: PathBuf,
    /// Issuer CA private key path
    ca_private: PathBuf,
    /// Serial number tracking file
    serial: PathBuf,
    /// Path to save signed certificate
    public: PathBuf,
    /// Certificate validity in days
    #[arg(long, default_value = "3650")]
    days: u32,
    /// Overwrite destination if exists
    #[arg(long)]
    renew: bool,
  },

  /// Generate OpenSSL DH parameters
  #[command(name = "tls-dhparam")]
  TlsDhparam {
    /// Path to save DH parameters file
    name: PathBuf,
    /// Overwrite destination if exists
    #[arg(long)]
    renew: bool,
  },

  /// Generate Nebula CA
  #[command(name = "nebula-ca")]
  NebulaCa {
    /// Common Name for the CA
    name: String,
    /// Path to save CA certificate
    public: PathBuf,
    /// Path to save CA private key
    private: PathBuf,
    /// Certificate validity in days
    #[arg(long, default_value = "3650")]
    days: u32,
    /// Overwrite destination if exists
    #[arg(long)]
    renew: bool,
  },

  /// Generate Nebula node certificate
  #[command(name = "nebula-cert")]
  NebulaCert {
    /// Path to Nebula CA certificate
    ca_public: PathBuf,
    /// Path to Nebula CA private key
    ca_private: PathBuf,
    /// Common Name for node certificate
    name: String,
    /// Node IP in CIDR or plain IP form
    ip: String,
    /// Path to save node certificate
    public: PathBuf,
    /// Path to save node private key
    private: PathBuf,
    /// Overwrite destination if exists
    #[arg(long)]
    renew: bool,
  },

  /// Generate CockroachDB CA
  #[command(name = "cockroach-ca")]
  CockroachCa {
    /// Path to save CA certificate
    public: PathBuf,
    /// Path to save CA private key
    private: PathBuf,
    /// Overwrite destination if exists
    #[arg(long)]
    renew: bool,
  },

  /// Generate CockroachDB node certificate
  #[command(name = "cockroach-node-cert")]
  CockroachNodeCert {
    /// Path to CockroachDB CA certificate
    ca_public: PathBuf,
    /// Path to CockroachDB CA private key
    ca_private: PathBuf,
    /// Path to save node certificate
    public: PathBuf,
    /// Path to save node private key
    private: PathBuf,
    /// Comma-separated hostnames/IPs for SANs
    hosts: String,
    /// Overwrite destination if exists
    #[arg(long)]
    renew: bool,
  },

  /// Generate CockroachDB client certificate
  #[command(name = "cockroach-client-cert")]
  CockroachClientCert {
    /// Path to CockroachDB CA certificate
    ca_public: PathBuf,
    /// Path to CockroachDB CA private key
    ca_private: PathBuf,
    /// Path to save client certificate
    public: PathBuf,
    /// Path to save client private key
    private: PathBuf,
    /// CockroachDB username
    user: String,
    /// Overwrite destination if exists
    #[arg(long)]
    renew: bool,
  },

  /// Generate environment (.env) file
  Env {
    /// Destination file path
    name: PathBuf,
    /// Input format of variables (json, yaml, toml)
    format: String,
    /// Path to variables file
    vars: PathBuf,
    /// Overwrite destination if exists
    #[arg(long)]
    renew: bool,
  },

  /// Generate from Mustache template
  #[command(name = "mustache")]
  Mustache {
    /// Base name for output files (adds -variables, -template suffixes)
    name: PathBuf,
    /// Input format of combined file (json, yaml, toml)
    format: String,
    /// Path to {template, variables} file
    variables_and_template: PathBuf,
    /// Overwrite destination if exists
    #[arg(long)]
    renew: bool,
  },

  /// Generate and run Nushell script
  Script {
    /// Path to save script
    name: PathBuf,
    /// Script content
    text: String,
    /// Overwrite destination if exists
    #[arg(long)]
    renew: bool,
  },

  /// Generate SOPS-encrypted secrets
  Sops {
    /// Path to Age recipient(s) file
    age: PathBuf,
    /// Path to save encrypted YAML
    public: PathBuf,
    /// Path to save plaintext YAML
    private: PathBuf,
    /// Input format for secrets (json, yaml, toml)
    format: String,
    /// Path to secrets file
    values: PathBuf,
    /// Overwrite destination if exists
    #[arg(long)]
    renew: bool,
  },
}

#[derive(Subcommand, Debug)]
pub enum ExportCommands {
  /// Copy a file
  Copy {
    /// Source path
    from: PathBuf,
    /// Destination path
    to: PathBuf,
  },

  /// Export to Vault
  Vault {
    /// Base vault path
    path: String,
  },

  /// Export single file to Vault
  #[command(name = "vault-file")]
  VaultFile {
    /// Base vault path
    path: String,
    /// Local file to export
    file: String,
  },
}
