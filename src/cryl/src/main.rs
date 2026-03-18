#![deny(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![deny(clippy::arithmetic_side_effects)]
#![deny(clippy::todo)]
#![deny(clippy::unreachable)]
#![deny(clippy::allow_attributes_without_reason)]
#![allow(dead_code, reason = "Added for now until migration is done")]

mod cli;
mod common;
mod exporters;
mod generators;
mod importers;
mod manifest;
mod schema;

use crate::common::{deserialize, Format};
use crate::schema::Specification;
use clap::Parser;
use cli::{Cli, Commands, ExportCommands, GenerateCommands, ImportCommands};
use schemars::schema_for;
use std::io::{self, Read};
use std::path::Path;

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
      importers::import_copy(&from, &to, allow_fail)?;
    }
    ImportCommands::Vault { path, allow_fail } => {
      println!(
        "Import vault: {} (allow_fail: {}) - not yet implemented",
        path, allow_fail
      );
    }
    ImportCommands::VaultFile {
      path,
      file,
      allow_fail,
    } => {
      println!(
        "Import vault-file: {}/{} (allow_fail: {}) - not yet implemented",
        path, file, allow_fail
      );
    }
  }
  Ok(())
}

fn run_generate(
  cmd: GenerateCommands,
) -> Result<(), Box<dyn std::error::Error>> {
  use crate::common::{
    generate_random_alphanumeric, generate_random_digits, save_atomic,
  };

  match cmd {
    GenerateCommands::Id {
      name,
      length,
      renew,
    } => {
      let id = generate_random_alphanumeric(length as usize)?;
      save_atomic(&name, id.as_bytes(), renew, false)?;
      println!("Generated id: {:?}", name);
    }
    GenerateCommands::Key {
      name,
      length,
      renew,
    } => {
      let key = generate_random_alphanumeric(length as usize)?;
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
    GenerateCommands::Password {
      name,
      length,
      renew,
    } => {
      let password = generators::generate_password(length)?;
      save_atomic(&name, password.as_bytes(), renew, false)?;
      print!("Generated password: {:?}", name);
    }
  }
  Ok(())
}

fn run_export(cmd: ExportCommands) -> Result<(), Box<dyn std::error::Error>> {
  match cmd {
    ExportCommands::Copy { from, to } => {
      exporters::export_copy(&from, &to)?;
    }
    ExportCommands::Vault { path } => {
      println!("Export vault: {} - not yet implemented", path);
    }
    ExportCommands::VaultFile { path, file } => {
      println!("Export vault-file: {}/{} - not yet implemented", path, file);
    }
  }
  Ok(())
}
