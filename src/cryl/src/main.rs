#![deny(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![deny(clippy::arithmetic_side_effects)]
#![deny(clippy::todo)]
#![deny(clippy::unreachable)]
#![deny(clippy::allow_attributes_without_reason)]

use clap::Parser;
use cryl::format::{deserialize, serialize, Format};
use cryl::schema::Specification;
use schemars::schema_for;
use std::io::{self, Read};
use std::path::Path;

mod cli;

use cli::{Cli, Commands, ExportCommands, GenerateCommands, ImportCommands};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let cli = Cli::parse();

  match cli.command {
    Commands::Schema => print_schema(),
    Commands::Path {
      spec,
      common,
      sandbox,
    } => run_from_path(&spec, &common, &sandbox),
    Commands::Stdin {
      format,
      common,
      sandbox,
    } => run_from_stdin(&format, &common, &sandbox),
    Commands::Import(import_cmd) => run_import(import_cmd),
    Commands::Generate(gen_cmd) => run_generate(gen_cmd),
    Commands::Export(export_cmd) => run_export(export_cmd),
  }
}

fn print_schema() -> Result<(), Box<dyn std::error::Error>> {
  let schema = schema_for!(Specification);
  println!("{}", serde_json::to_string_pretty(&schema)?);
  Ok(())
}

fn run_from_path(
  _spec_path: &Path,
  _common: &cli::CommonArgs,
  _sandbox: &cli::SandboxArgs,
) -> Result<(), Box<dyn std::error::Error>> {
  // TODO: Implement full execution pipeline
  println!("TODO: Run from path");
  Ok(())
}

fn run_from_stdin(
  format: &str,
  _common: &cli::CommonArgs,
  _sandbox: &cli::SandboxArgs,
) -> Result<(), Box<dyn std::error::Error>> {
  let format = Format::parse(format)
    .ok_or_else(|| format!("Unknown format: {}", format))?;

  let mut content = String::new();
  io::stdin().read_to_string(&mut content)?;

  let _spec: Specification = deserialize(&content, format)?;

  // TODO: Implement full execution pipeline
  println!("TODO: Run from stdin");
  Ok(())
}

fn run_import(cmd: ImportCommands) -> Result<(), Box<dyn std::error::Error>> {
  match cmd {
    ImportCommands::Copy {
      from,
      to,
      allow_fail,
    } => {
      println!(
        "Import copy: {:?} -> {:?} (allow_fail: {})",
        from, to, allow_fail
      );
      // TODO: Implement copy importer
    }
    ImportCommands::Vault { path, allow_fail } => {
      println!("Import vault: {} (allow_fail: {})", path, allow_fail);
      // TODO: Implement vault importer
    }
    ImportCommands::VaultFile {
      path,
      file,
      allow_fail,
    } => {
      println!(
        "Import vault-file: {}/{} (allow_fail: {})",
        path, file, allow_fail
      );
      // TODO: Implement vault-file importer
    }
  }
  Ok(())
}

fn run_generate(
  cmd: GenerateCommands,
) -> Result<(), Box<dyn std::error::Error>> {
  use cryl::{generate_random_alnum, generate_random_digits, save_atomic};

  match cmd {
    GenerateCommands::Id {
      name,
      length,
      renew,
    } => {
      let id = generate_random_alnum(length as usize)?;
      save_atomic(&name, id.as_bytes(), renew, false)?;
      println!("Generated id: {:?}", name);
    }
    GenerateCommands::Key {
      name,
      length,
      renew,
    } => {
      let key = generate_random_alnum(length as usize)?;
      save_atomic(&name, key.as_bytes(), renew, false)?;
      println!("Generated key: {:?}", name);
    }
    GenerateCommands::Pin {
      name,
      length,
      renew,
    } => {
      let pin = generate_random_digits(length as usize)?;
      save_atomic(&name, pin.as_bytes(), renew, false)?;
      println!("Generated pin: {:?}", name);
    }
  }
  Ok(())
}

fn run_export(cmd: ExportCommands) -> Result<(), Box<dyn std::error::Error>> {
  match cmd {
    ExportCommands::Copy { from, to } => {
      println!("Export copy: {:?} -> {:?}", from, to);
      // TODO: Implement copy exporter
    }
    ExportCommands::Vault { path } => {
      println!("Export vault: {}", path);
      // TODO: Implement vault exporter
    }
    ExportCommands::VaultFile { path, file } => {
      println!("Export vault-file: {}/{}", path, file);
      // TODO: Implement vault-file exporter
    }
  }
  Ok(())
}
