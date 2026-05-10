//! CLI argument parsing for cryl

use crate::common::Format;
use clap::{Args, Parser, Subcommand};
use clap_stdin::FileOrStdin;

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
    spec: String,
    #[command(flatten)]
    common: CommonArgs,
    #[command(flatten)]
    sandbox: SandboxArgs,
  },

  /// Load specification from stdin
  #[command(name = "stdin", visible_alias = "from-stdin")]
  Stdin {
    /// Format of the specification
    format: Format,
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

  /// Select manifest format
  #[arg(long, default_value = "json")]
  pub manifest_format: Format,

  /// Don't create a manifest file
  #[arg(long)]
  pub no_manifest: bool,

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

  /// Run envsubst on the specification before running (only in `${foo}` format)
  #[arg(long)]
  pub envsubst: bool,
}

#[derive(Args, Debug, Clone)]
pub struct SandboxArgs {
  /// Don't use sandbox while running
  #[arg(long)]
  pub nosandbox: bool,

  /// Additional read-only bind mounts for bubblewrap
  /// Format: `<target and source>,<target>:<source>,...`
  #[arg(long, value_delimiter = ',')]
  pub ro_binds: Vec<BindMount>,

  /// Additional bind mounts for bubblewrap
  /// Format: `<target and source>,<target>:<source>,...`
  #[arg(long, value_delimiter = ',')]
  pub binds: Vec<BindMount>,

  /// Additional tool binaries for bubblewrap PATH
  #[arg(long, value_delimiter = ',')]
  pub tools: Vec<String>,

  /// Environment variables to pass through
  #[arg(long, value_delimiter = ',')]
  pub env: Vec<String>,

  /// Allow network while running
  #[arg(long)]
  pub allow_net: bool,
}

#[derive(Subcommand, Debug)]
pub enum ImportCommands {
  /// Copy a file
  Copy {
    /// Source path
    from: String,
    /// Input format for listing file
    format: Format,
    /// Listing of source files
    listing: FileOrStdin,
    /// Allow failing to copy if source missing
    #[arg(long)]
    allow_fail: bool,
  },

  /// Import from Vault
  Vault {
    /// Vault path to import from
    path: String,
    /// Input format for listing file
    format: Format,
    /// Listing for destinations of files
    listing: FileOrStdin,
    /// Allow failing to import if source missing
    #[arg(long)]
    allow_fail: bool,
  },

  /// Change working directory for imports
  #[command(name = "working-directory")]
  WorkingDirectory {
    /// Path to the new working directory
    path: String,
  },
}

#[derive(Subcommand, Debug)]
pub enum GenerateCommands {
  /// Generate random alphanumeric id
  #[command(name = "id")]
  Id {
    /// Destination file name
    name: String,
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
    name: String,
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
    name: String,
    /// Number of digits
    #[arg(long, default_value = "8")]
    length: u32,
    /// Overwrite destination if it exists
    #[arg(long)]
    renew: bool,
  },

  /// Copy a file
  Copy {
    /// Source path
    from: String,
    /// Destination path
    to: String,
    /// Overwrite destination if exists
    #[arg(long)]
    renew: bool,
  },

  /// Generate text file
  Text {
    /// Destination file name
    name: String,
    /// Text content
    text: String,
    /// Overwrite destination if exists
    #[arg(long)]
    renew: bool,
  },

  /// Convert data to JSON
  Json {
    /// Destination file name
    name: String,
    /// Input format
    format: Format,
    /// Source data file
    data: FileOrStdin,
    /// Overwrite destination if exists
    #[arg(long)]
    renew: bool,
  },

  /// Convert data to YAML
  Yaml {
    /// Destination file name
    name: String,
    /// Input format
    format: Format,
    /// Source data file
    data: FileOrStdin,
    /// Overwrite destination if exists
    #[arg(long)]
    renew: bool,
  },

  /// Convert data to TOML
  Toml {
    /// Destination file name
    name: String,
    /// Input format
    format: Format,
    /// Source data file
    data: FileOrStdin,
    /// Overwrite destination if exists
    #[arg(long)]
    renew: bool,
  },

  /// Generate password (argon2)
  #[command(name = "password")]
  Password {
    /// Path for public/hashed password
    public: String,
    /// Path for private/plain password
    private: String,
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
    public: String,
    /// Path for private/plain password
    private: String,
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
    public: String,
    /// Private key path
    private: String,
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
    public: String,
    /// Private key path
    private: String,
    /// Passphrase file (optional)
    #[arg(long)]
    password: Option<String>,
    /// Overwrite if exists
    #[arg(long)]
    renew: bool,
  },

  /// Generate WireGuard keypair
  #[command(name = "wireguard-key")]
  WireguardKey {
    /// Private key path
    private: String,
    /// Public key path
    public: String,
    /// Overwrite if exists
    #[arg(long)]
    renew: bool,
  },

  /// Split key into Shamir shares
  #[command(name = "key-split")]
  KeySplit {
    /// Source key file
    key: String,
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
    key: String,
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
    config: String,
    /// Path to save private key
    private: String,
    /// Path to save self-signed certificate
    public: String,
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
    config: String,
    /// Path to write request config (will be created)
    request_config: String,
    /// Path to save private key
    private: String,
    /// Path to save CSR
    request: String,
    /// Issuer/CA certificate path
    ca_public: String,
    /// Issuer/CA private key path
    ca_private: String,
    /// Serial number tracking file
    serial: String,
    /// Path to save signed certificate
    public: String,
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
    config: String,
    /// Path to write request config (will be created)
    request_config: String,
    /// Path to save private key
    private: String,
    /// Path to save CSR
    request: String,
    /// Issuer CA certificate path
    ca_public: String,
    /// Issuer CA private key path
    ca_private: String,
    /// Serial number tracking file
    serial: String,
    /// Path to save signed certificate
    public: String,
    /// Certificate validity in days
    #[arg(long, default_value = "3650")]
    days: u32,
    /// Overwrite destination if exists
    #[arg(long)]
    renew: bool,
  },

  /// TLS Root CA (EC)
  #[command(name = "tls-root")]
  TlsRoot {
    /// Common Name for the Root CA
    common_name: String,
    /// Organization name
    organization: String,
    /// Path to write OpenSSL config
    config: String,
    /// Path to save private key
    private: String,
    /// Path to save self-signed certificate
    public: String,
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

  /// TLS Intermediate CA (EC)
  #[command(name = "tls-intermediary")]
  TlsIntermediary {
    /// Common Name for the Intermediate CA
    common_name: String,
    /// Organization name
    organization: String,
    /// Path to write merged OpenSSL config (extensions + request)
    config: String,
    /// Path to write request config (will be created)
    request_config: String,
    /// Path to save private key
    private: String,
    /// Path to save CSR
    request: String,
    /// Issuer/CA certificate path
    ca_public: String,
    /// Issuer/CA private key path
    ca_private: String,
    /// Serial number tracking file
    serial: String,
    /// Path to save signed certificate
    public: String,
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

  /// TLS Leaf certificate (EC)
  #[command(name = "tls-leaf")]
  TlsLeaf {
    /// Common Name for certificate
    common_name: String,
    /// Organization name
    organization: String,
    /// Comma-separated Subject Alternative Names
    sans: String,
    /// Path to write merged OpenSSL config (extensions + request)
    config: String,
    /// Path to write request config (will be created)
    request_config: String,
    /// Path to save private key
    private: String,
    /// Path to save CSR
    request: String,
    /// Issuer CA certificate path
    ca_public: String,
    /// Issuer CA private key path
    ca_private: String,
    /// Serial number tracking file
    serial: String,
    /// Path to save signed certificate
    public: String,
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
    name: String,
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
    public: String,
    /// Path to save CA private key
    private: String,
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
    ca_public: String,
    /// Path to Nebula CA private key
    ca_private: String,
    /// Common Name for node certificate
    name: String,
    /// Node IP in CIDR or plain IP form
    ip: String,
    /// Path to save node certificate
    public: String,
    /// Path to save node private key
    private: String,
    /// Overwrite destination if exists
    #[arg(long)]
    renew: bool,
  },

  /// Generate CockroachDB CA
  #[command(name = "cockroach-ca")]
  CockroachCa {
    /// Path to save CA certificate
    public: String,
    /// Path to save CA private key
    private: String,
    /// Overwrite destination if exists
    #[arg(long)]
    renew: bool,
  },

  /// Generate CockroachDB node certificate
  #[command(name = "cockroach-node-cert")]
  CockroachNodeCert {
    /// Path to CockroachDB CA certificate
    ca_public: String,
    /// Path to CockroachDB CA private key
    ca_private: String,
    /// Path to save node certificate
    public: String,
    /// Path to save node private key
    private: String,
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
    ca_public: String,
    /// Path to CockroachDB CA private key
    ca_private: String,
    /// Path to save client certificate
    public: String,
    /// Path to save client private key
    private: String,
    /// CockroachDB username
    user: String,
    /// Overwrite destination if exists
    #[arg(long)]
    renew: bool,
  },

  /// Generate environment (.env) file
  Env {
    /// Destination file path
    name: String,
    /// Input format of variables
    format: Format,
    /// Variables file
    variables: FileOrStdin,
    /// Overwrite destination if exists
    #[arg(long)]
    renew: bool,
  },

  /// Generate from Mustache template
  #[command(name = "mustache")]
  Mustache {
    /// Base name for output files (adds -variables, -template suffixes)
    name: String,
    /// Input format of template and variables file
    format: Format,
    /// File with object of template string and variables as directory listing
    listing_and_template: FileOrStdin,
    /// Overwrite destination if exists
    #[arg(long)]
    renew: bool,
  },

  /// Generate SOPS-encrypted secrets
  Sops {
    /// Path to Age recipient(s) file
    age: String,
    /// Path to save encrypted YAML
    public: String,
    /// Path to save plaintext YAML
    private: String,
    /// Input format for secrets
    format: Format,
    /// Secrets file containing value for directory listing
    values: FileOrStdin,
    /// Overwrite destination if exists
    #[arg(long)]
    renew: bool,
  },

  /// Generate and run Nushell script
  Script {
    /// Path to save script
    name: String,
    /// Script content
    text: String,
    /// Overwrite destination if exists
    #[arg(long)]
    renew: bool,
  },

  /// Change working directory
  #[command(name = "working-directory")]
  WorkingDirectory {
    /// Path to the new working directory
    path: String,
  },
}

#[derive(Subcommand, Debug)]
pub enum ExportCommands {
  /// Copy a file
  Copy {
    /// Input format for listing file
    format: Format,
    /// Listing of source files
    listing: FileOrStdin,
    /// Destination path
    to: String,
  },

  /// Export to Vault
  Vault {
    /// Base vault path
    path: String,
    /// Input format for listing file
    format: Format,
    /// Listing of source files
    listing: FileOrStdin,
  },

  /// Change working directory for exports
  #[command(name = "working-directory")]
  WorkingDirectory {
    /// Path to the new working directory
    path: String,
  },
}

#[derive(Clone, Debug)]
pub struct BindMount {
  pub source: String,
  pub target: Option<String>,
}

impl std::str::FromStr for BindMount {
  type Err = &'static str;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let parts = s.split(':').collect::<Vec<_>>();
    match parts.len() {
      1 => Ok(BindMount {
        source: String::from(parts[0]),
        target: None,
      }),
      2 => Ok(BindMount {
        source: String::from(parts[0]),
        target: Some(String::from(parts[1])),
      }),
      _ => Err("Invalid bind mount format. Use 'path' or 'source:target'"),
    }
  }
}
