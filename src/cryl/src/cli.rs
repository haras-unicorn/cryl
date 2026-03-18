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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_cli_parse_path() {
    let args = vec!["cryl", "path", "spec.toml"];
    let cli = Cli::parse_from(args);
    match cli.command {
      Commands::Path { spec, .. } => {
        assert_eq!(spec, PathBuf::from("spec.toml"));
      }
      _ => panic!("Expected Path command"),
    }
  }

  #[test]
  fn test_cli_parse_stdin() {
    let args = vec!["cryl", "stdin", "yaml"];
    let cli = Cli::parse_from(args);
    match cli.command {
      Commands::Stdin { format, .. } => {
        assert_eq!(format, "yaml");
      }
      _ => panic!("Expected Stdin command"),
    }
  }

  #[test]
  fn test_cli_parse_schema() {
    let args = vec!["cryl", "schema"];
    let cli = Cli::parse_from(args);
    match cli.command {
      Commands::Schema => {}
      _ => panic!("Expected Schema command"),
    }
  }

  #[test]
  fn test_cli_parse_import_copy() {
    let args = vec!["cryl", "import", "copy", "/from", "/to"];
    let cli = Cli::parse_from(args);
    match cli.command {
      Commands::Import(ImportCommands::Copy { from, to, .. }) => {
        assert_eq!(from, PathBuf::from("/from"));
        assert_eq!(to, PathBuf::from("/to"));
      }
      _ => panic!("Expected Import Copy command"),
    }
  }

  #[test]
  fn test_cli_parse_generate_id() {
    let args = vec!["cryl", "generate", "id", "my-id", "--length", "32"];
    let cli = Cli::parse_from(args);
    match cli.command {
      Commands::Generate(GenerateCommands::Id { name, length, .. }) => {
        assert_eq!(name, PathBuf::from("my-id"));
        assert_eq!(length, 32);
      }
      _ => panic!("Expected Generate Id command"),
    }
  }
}
