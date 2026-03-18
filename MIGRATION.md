# cryl Migration Guide: Nushell → Rust

This document describes the migration of the `rumor` secret generation tool from
Nushell to Rust (`cryl`). It serves as both a specification for the
implementation and a reference for understanding architectural decisions and
behavioral changes.

## Overview

The migration maintains full feature parity with the original Nushell
implementation while improving type safety, error handling, and testability. The
three-phase execution model (Import → Generate → Export) remains unchanged.

## Architecture Changes

### Error Handling

Replace Nushell's error propagation with Rust's `thiserror` for structured,
detailed error messages:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CrylError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid specification: {message}")]
    InvalidSpec { message: String },

    #[error("Tool execution failed: {tool} exited with {exit_code}")]
    ToolExecution {
        tool: String,
        exit_code: i32,
        stderr: String
    },

    #[error("Sandbox error: {0}")]
    Sandbox(String),

    #[error("Import failed: {importer} - {message}")]
    Import { importer: String, message: String },

    #[error("Generation failed: {generator} - {message}")]
    Generation { generator: String, message: String },

    #[error("Export failed: {exporter} - {message}")]
    Export { exporter: String, message: String },
}
```

### Command Execution Pattern

Replace Nushell's `exec` with direct `std::process::Command` execution:

```rust
pub trait CommandBuilder {
    fn program(&self) -> &str;
    fn args(&self) -> &[String];

    fn execute(&self) -> Result<Output, CrylError> {
        let output = std::process::Command::new(self.program())
            .args(self.args())
            .output()?;

        if !output.status.success() {
            return Err(CrylError::ToolExecution {
                tool: self.program().to_string(),
                exit_code: output.status.code().unwrap_or(-1),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        Ok(output)
    }
}
```

### File Operations

Replace Nushell's `save` with atomic file operations:

```rust
pub fn save_atomic(
    path: &Path,
    content: &[u8],
    renew: bool,
    public: bool,
) -> Result<(), CrylError> {
    let tmp_path = path.with_extension("tmp");

    // Write to temp file
    fs::write(&tmp_path, content)?;

    // Set permissions (600 private, 644 public)
    let perms = if public { 0o644 } else { 0o600 };
    let mut permissions = fs::metadata(&tmp_path)?.permissions();
    permissions.set_mode(perms);
    fs::set_permissions(&tmp_path, permissions)?;

    // Atomic rename
    if renew {
        fs::rename(&tmp_path, path)?;
    } else {
        match fs::rename(&tmp_path, path) {
            Ok(()) => {},
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                fs::remove_file(&tmp_path)?;
            }
            Err(e) => return Err(e.into()),
        }
    }

    Ok(())
}
```

## CLI Structure

### Main Entry Points

Replace Nushell's subcommand structure with clap derive macros:

```rust
#[derive(Parser)]
#[command(name = "cryl", version)]
enum Cli {
    /// Load spec from file
    #[command(visible_alias = "from-path")]
    Path {
        spec: PathBuf,
        #[command(flatten)]
        args: CommonArgs,
        #[command(flatten)]
        sandbox: SandboxArgs,
    },

    /// Load spec from stdin
    #[command(visible_alias = "from-stdin")]
    Stdin {
        format: String,
        #[command(flatten)]
        args: CommonArgs,
        #[command(flatten)]
        sandbox: SandboxArgs,
    },

    /// Print JSON schema
    Schema,

    /// Direct import commands (non-sandboxed only)
    #[command(subcommand)]
    Import(ImportCommands),

    /// Direct generation commands (non-sandboxed only)
    #[command(subcommand)]
    Generate(GenerateCommands),

    /// Direct export commands (non-sandboxed only)
    #[command(subcommand)]
    Export(ExportCommands),
}

#[derive(Args)]
struct CommonArgs {
    #[arg(long)]
    dry_run: bool,

    #[arg(long)]
    allow_script: bool,

    #[arg(long, default_value = "1024")]
    max_imports: usize,

    #[arg(long, default_value = "1024")]
    max_generations: usize,

    #[arg(long, default_value = "1024")]
    max_exports: usize,

    #[arg(long, default_value = "1048576")]
    max_specification_size: usize,

    #[arg(long, default_value = "json")]
    manifest_format: String,

    #[arg(long)]
    verbose: bool,

    #[arg(long)]
    very_verbose: bool,
}

#[derive(Args)]
struct SandboxArgs {
    #[arg(long)]
    nosandbox: bool,

    #[arg(long, value_delimiter = ',')]
    ro_binds: Vec<PathBuf>,

    #[arg(long, value_delimiter = ',')]
    binds: Vec<PathBuf>,

    #[arg(long, value_delimiter = ',')]
    tools: Vec<String>,

    #[arg(long)]
    allow_net: bool,
}
```

### Direct Commands

Individual importers/generators/exporters are available as subcommands when
running non-sandboxed:

```rust
#[derive(Subcommand)]
enum ImportCommands {
    /// Copy a file
    Copy {
        from: PathBuf,
        to: PathBuf,
        #[arg(long)]
        allow_fail: bool,
    },
    /// Import from Vault
    Vault {
        path: String,
        #[arg(long)]
        allow_fail: bool,
    },
    /// Import single file from Vault
    VaultFile {
        path: String,
        file: String,
        #[arg(long)]
        allow_fail: bool,
    },
}

#[derive(Subcommand)]
enum GenerateCommands {
    /// Generate random alphanumeric id
    Id {
        name: PathBuf,
        #[arg(long, default_value = "16")]
        length: u32,
        #[arg(long)]
        renew: bool,
    },
    /// Generate random key
    Key {
        name: PathBuf,
        #[arg(long, default_value = "32")]
        length: u32,
        #[arg(long)]
        renew: bool,
    },
    /// Generate PIN
    Pin {
        name: PathBuf,
        #[arg(long, default_value = "8")]
        length: u32,
        #[arg(long)]
        renew: bool,
    },
    // ... additional generators
}

#[derive(Subcommand)]
enum ExportCommands {
    /// Copy file to destination
    Copy {
        from: PathBuf,
        to: PathBuf,
    },
    /// Export to Vault
    Vault {
        path: String,
    },
    /// Export single file to Vault
    VaultFile {
        path: String,
        file: String,
    },
}
```

## Core Types

### Simplified Manifest

Replace the full capture manifest with a hash-focused version:

```rust
#[derive(Serialize, Deserialize, Debug)]
pub struct Manifest {
    pub version: String,
    pub environment: HashMap<String, String>, // tool_name -> version_or_hash
    pub spec_hash: String, // SHA256 of specification file
    pub output_hashes: HashMap<String, String>, // file_path -> SHA256
}

impl Manifest {
    pub fn compute_spec_hash(spec_content: &[u8]) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(spec_content);
        hex::encode(hasher.finalize())
    }

    pub fn compute_file_hash(path: &Path) -> Result<String, CrylError> {
        use sha2::{Sha256, Digest};
        let content = fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        Ok(hex::encode(hasher.finalize()))
    }
}
```

### Specification Types

Reuse existing schema types with minor adjustments:

```rust
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct Specification {
    pub imports: Vec<Import>,
    pub generations: Vec<Generation>,
    pub exports: Vec<Export>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(tag = "importer", rename_all = "snake_case")]
pub enum Import {
    Vault { arguments: VaultImportArgs },
    VaultFile { arguments: VaultFileImportArgs },
    Copy { arguments: CopyImportArgs },
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(tag = "generator", rename_all = "kebab-case")]
pub enum Generation {
    Copy { arguments: CopyGenArgs },
    Text { arguments: TextGenArgs },
    Json { arguments: DataGenArgs },
    Yaml { arguments: DataGenArgs },
    Toml { arguments: DataGenArgs },
    Id { arguments: IdGenArgs },
    Key { arguments: IdGenArgs },
    Pin { arguments: PinGenArgs },
    Password { arguments: PasswordGenArgs },
    #[serde(rename = "password-crypt-3")]
    PasswordCrypt3 { arguments: PasswordGenArgs },
    #[serde(rename = "age-key")]
    AgeKey { arguments: AgeKeyArgs },
    #[serde(rename = "ssh-key")]
    SshKey { arguments: SshKeyArgs },
    #[serde(rename = "wireguard-key")]
    WireguardKey { arguments: WireguardKeyArgs },
    #[serde(rename = "key-split")]
    KeySplit { arguments: KeySplitArgs },
    #[serde(rename = "key-combine")]
    KeyCombine { arguments: KeyCombineArgs },
    #[serde(rename = "tls-root")]
    TlsRoot { arguments: TlsRootArgs },
    #[serde(rename = "tls-intermediary")]
    TlsIntermediary { arguments: TlsIntermediaryArgs },
    #[serde(rename = "tls-leaf")]
    TlsLeaf { arguments: TlsLeafArgs },
    #[serde(rename = "tls-rsa-root")]
    TlsRsaRoot { arguments: TlsRootArgs },
    #[serde(rename = "tls-rsa-intermediary")]
    TlsRsaIntermediary { arguments: TlsIntermediaryArgs },
    #[serde(rename = "tls-rsa-leaf")]
    TlsRsaLeaf { arguments: TlsLeafArgs },
    #[serde(rename = "tls-dhparam")]
    TlsDhparam { arguments: DhparamArgs },
    #[serde(rename = "nebula-ca")]
    NebulaCa { arguments: NebulaCaArgs },
    #[serde(rename = "nebula-cert")]
    NebulaCert { arguments: NebulaCertArgs },
    #[serde(rename = "cockroach-ca")]
    CockroachCa { arguments: CockroachCaArgs },
    #[serde(rename = "cockroach-node-cert")]
    CockroachNodeCert { arguments: CockroachNodeCertArgs },
    #[serde(rename = "cockroach-client-cert")]
    CockroachClientCert { arguments: CockroachClientCertArgs },
    Env { arguments: EnvArgs },
    Moustache { arguments: MoustacheArgs },
    Script { arguments: ScriptArgs },
    Sops { arguments: SopsArgs },
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(tag = "exporter", rename_all = "snake_case")]
pub enum Export {
    Vault { arguments: VaultExportArgs },
    VaultFile { arguments: VaultFileExportArgs },
    Copy { arguments: CopyExportArgs },
}
```

## Sandbox Implementation

### Manager Mode (CRYL_SANDBOX unset)

When not in sandbox mode, cryl acts as a manager that sets up bubblewrap:

```rust
pub fn run_in_sandbox(
    spec_path: &Path,
    parsed_args: &ParsedArgs,
) -> Result<(), CrylError> {
    let mut bwrap = std::process::Command::new("bwrap");

    // Clear environment and set sandbox flag
    bwrap.arg("--clearenv");
    bwrap.env("CRYL_SANDBOX", "1");

    if parsed_args.verbose {
        bwrap.env("CRYL_VERBOSE", "1");
    }
    if parsed_args.very_verbose {
        bwrap.env("CRYL_VERY_VERBOSE", "1");
    }

    // Create tool directory with symlinks
    let tool_dir = create_tool_symlinks(&parsed_args.tools)?;
    bwrap.arg("--ro-bind").arg(&tool_dir).arg("/tools");
    bwrap.env("PATH", "/tools");

    // Add read-only binds for imports
    for bind in &parsed_args.ro_binds {
        let abs = fs::canonicalize(bind)?;
        bwrap.arg("--ro-bind").arg(&abs).arg(&abs);
    }

    // Add read-write binds for exports
    for bind in &parsed_args.binds {
        let abs = fs::canonicalize(bind)?;
        bwrap.arg("--bind").arg(&abs).arg(&abs);
    }

    // Bind system directories if not Nix
    if !Path::new("/nix/store").exists() {
        if Path::new("/usr").exists() {
            bwrap.arg("--ro-bind").arg("/usr").arg("/usr");
        }
        if Path::new("/bin").exists() {
            bwrap.arg("--ro-bind").arg("/bin").arg("/bin");
        }
        if Path::new("/lib").exists() {
            bwrap.arg("--ro-bind").arg("/lib").arg("/lib");
        }
        if Path::new("/lib64").exists() {
            bwrap.arg("--ro-bind").arg("/lib64").arg("/lib64");
        }
    } else {
        bwrap.arg("--ro-bind").arg("/nix").arg("/nix");
    }

    if Path::new("/etc").exists() {
        bwrap.arg("--ro-bind").arg("/etc").arg("/etc");
    }

    // Working directory setup
    bwrap.arg("--tmpfs").arg("/work");
    bwrap.arg("--chdir").arg("/work");

    // Home directory
    bwrap.arg("--dir").arg("/home");
    bwrap.env("HOME", "/home");

    // Temporary directory
    bwrap.arg("--tmpfs").arg("/tmp");
    bwrap.env("TMPDIR", "/tmp");

    // Locale
    bwrap.env("LC_ALL", "C.UTF-8");
    bwrap.env("LANG", "C.UTF-8");

    // Network restriction
    if !parsed_args.allow_net {
        bwrap.arg("--unshare-net");
    }

    // Security options
    bwrap.arg("--die-with-parent");
    bwrap.arg("--unshare-user");
    bwrap.arg("--uid").arg("0");
    bwrap.arg("--gid").arg("0");
    bwrap.arg("--unshare-pid");
    bwrap.arg("--unshare-uts");
    bwrap.arg("--unshare-ipc");
    bwrap.arg("--proc").arg("/proc");
    bwrap.arg("--dev-bind").arg("/dev").arg("/dev");

    // Execute cryl in sandbox mode
    bwrap.arg("/proc/self/exe");
    bwrap.arg("from-manifest");
    bwrap.arg(spec_path);

    let status = bwrap.status()?;
    if !status.success() {
        return Err(CrylError::Sandbox(format!(
            "Sandbox exited with code {}",
            status.code().unwrap_or(-1)
        )));
    }

    Ok(())
}

fn create_tool_symlinks(tools: &[String]) -> Result<PathBuf, CrylError> {
    let tool_dir = tempfile::tempdir()?.into_path();

    for tool in tools {
        let tool_path = which::which(tool)
            .map_err(|_| CrylError::Sandbox(format!("Tool not found: {}", tool)))?;
        let link = tool_dir.join(tool);
        std::os::unix::fs::symlink(&tool_path, &link)?;
    }

    Ok(tool_dir)
}
```

### Worker Mode (CRYL_SANDBOX=1)

When `CRYL_SANDBOX` is set, cryl executes operations directly without additional
sandboxing:

```rust
pub fn run_worker(spec_path: &Path) -> Result<(), CrylError> {
    // Read and parse specification
    let spec_content = fs::read_to_string(spec_path)?;
    let format = detect_format(spec_path)?;
    let spec: Specification = parse_spec(&spec_content, format)?;

    // Validate against limits
    validate_limits(&spec)?;

    // Execute phases
    let manifest = execute_phases(spec)?;

    // Output manifest
    println!("{}", serde_json::to_string_pretty(&manifest)?);

    Ok(())
}
```

## Module Structure

```
src/
├── main.rs              # CLI entry point, command routing
├── lib.rs               # Core types, errors, traits, file operations
├── schema.rs            # Specification types (from existing)
├── format.rs            # JSON/YAML/TOML parsing and serialization
├── sandbox.rs           # Manager/worker sandbox logic
├── importers/
│   ├── mod.rs           # Import trait and registry
│   ├── copy.rs          # Copy importer
│   └── vault.rs         # Vault and vault-file importers
├── generators/
│   ├── mod.rs           # Generator trait and registry
│   ├── basic.rs         # id, key, pin, password generators
│   ├── keys.rs          # age-key, ssh-key, wireguard-key
│   ├── tls.rs           # TLS certificate generators (EC + RSA)
│   ├── pki.rs           # Nebula and CockroachDB generators
│   └── aggregators.rs   # sops, env, moustache, script generators
├── exporters/
│   ├── mod.rs           # Export trait and registry
│   ├── copy.rs          # Copy exporter
│   └── vault.rs         # Vault and vault-file exporters
└── tests/
    ├── unit/            # Unit tests for each module
    └── e2e/             # End-to-end tests
```

## Dependencies

### Required

```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"
toml = "0.8"
schemars = "1.2"
thiserror = "1.0"
sha2 = "0.10"
hex = "0.4"
tempfile = "3.0"
which = "6.0"
mustache = "2.0"

# Nushell integration for script generator
nu-protocol = "0.90"
nu-engine = "0.90"
nu-parser = "0.90"
nu-command = "0.90"

[dev-dependencies]
mockall = "0.12"
assert_cmd = "2.0"
predicates = "3.0"
tempfile = "3.0"
```

## Behavioral Changes

### Command Names

- `rumor from-path <spec>` → `cryl <spec>` or `cryl path <spec>`
- `rumor from-stdin <format>` → `cryl stdin <format>`
- Direct commands (e.g., `rumor import copy ...`) → `cryl import copy ...`
  (non-sandboxed only)

### Manifest Changes

- Removed: Full script text capture, commandline capture
- Simplified to: version, environment map, spec hash, output file hashes
- Format: JSON, YAML, or TOML (configurable via `--manifest-format`)

### Error Handling

- Detailed error messages via `thiserror`
- Tool execution failures include stderr output
- Each operation phase (import/generation/export) has specific error variants

### Sandbox

- Manager/Worker pattern via `CRYL_SANDBOX` environment variable
- Cleaner implementation using `/proc/self/exe` for binary path
- Environment variables `CRYL_VERBOSE` and `CRYL_VERY_VERBOSE` control logging

## Implementation Priority

1. **Foundation**: Error types, CLI structure, file operations
2. **Core**: Format parsing, sandbox manager/worker
3. **Basic Features**: Copy import/export, id/key/pin generators
4. **PKI Features**: TLS certificates (EC + RSA), Nebula, CockroachDB
5. **Key Management**: age-key, ssh-key, wireguard-key
6. **Aggregators**: SOPS, environment files, templates
7. **Advanced**: Shamir secret sharing, script execution
8. **Testing**: Unit tests, E2E tests (sandbox and native modes)

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;

    mock! {
        CommandRunner {
            fn run(&self, cmd: &str, args: &[String]) -> Result<Output, CrylError>;
        }
    }

    #[test]
    fn test_id_generation() {
        let mut mock = MockCommandRunner::new();
        mock.expect_run()
            .with(eq("openssl"), eq(vec!["rand", "-base64", "32"]))
            .returning(|_, _| Ok(Output {
                stdout: b"abc123XYZ789!@#".to_vec(),
                stderr: vec![],
                status: std::process::ExitStatus::default(),
            }));

        let generator = IdGenerator::with_runner(mock);
        let result = generator.generate(16).unwrap();
        assert_eq!(result.len(), 16);
        assert!(result.chars().all(|c| c.is_ascii_alphanumeric()));
    }
}
```

### E2E Tests

```rust
#[test]
fn test_sandbox_execution() {
    let spec = r#"
[[generations]]
generator = "id"
arguments.name = "test-id"
arguments.length = 16
"#;

    let temp = tempfile::tempdir().unwrap();
    let spec_path = temp.path().join("spec.toml");
    fs::write(&spec_path, spec).unwrap();

    let output = Command::new("cryl")
        .arg(&spec_path)
        .current_dir(&temp)
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
    assert!(temp.path().join("test-id").exists());

    let content = fs::read_to_string(temp.path().join("test-id")).unwrap();
    assert_eq!(content.len(), 16);
}
```

## External Tool Dependencies

The following tools must be available in the sandbox (or host for non-sandboxed
mode):

- `openssl` - Random generation, TLS certificates
- `age-keygen` - Age key generation
- `ssh-keygen` - SSH key generation
- `wg` (wireguard-tools) - WireGuard keys
- `argon2` - Password hashing
- `mkpasswd` - yescrypt password hashing
- `sops` - Secret encryption
- `medusa` - Vault bulk operations
- `vault` - Vault CLI
- `nebula-cert` - Nebula certificates
- `cockroach` - CockroachDB certificates
- `ssss-split`, `ssss-combine` - Shamir secret sharing
- `mo` - Mustache templates (optional if using Rust crate)
- `nu` - Nushell for script execution
- `bwrap` (bubblewrap) - Sandbox execution

## Notes

- All file operations use atomic writes (tmp file + rename) to prevent
  corruption
- Permissions are set strictly: 600 for secrets, 644 for public files
- The specification schema remains unchanged from the original
- Tool execution uses `Vec<String>` for arguments to prevent shell injection
- Sandbox mode restricts network access unless `--allow-net` is specified
