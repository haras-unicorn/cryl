#![deny(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![deny(clippy::arithmetic_side_effects)]
#![deny(clippy::todo)]
#![deny(clippy::unreachable)]
#![deny(clippy::allow_attributes_without_reason)]
#![allow(dead_code, reason = "Added for now until migration is done")]
#![allow(unused, reason = "Added for now until migration is done")]
#![allow(
  clippy::too_many_arguments,
  reason = "Added for now until migration is done"
)]

mod cli;
mod common;
mod exporters;
mod generators;
mod importers;
mod manifest;
mod schema;

use crate::common::{deserialize, CrylResult, Format};
use crate::schema::Specification;
use clap::Parser;
use cli::{Cli, Commands, ExportCommands, GenerateCommands, ImportCommands};
use schemars::schema_for;
use std::io::{self, Read};
use std::path::Path;

fn main() -> CrylResult<()> {
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

fn print_schema() -> CrylResult<()> {
  let schema = schema_for!(Specification);
  println!("{}", serde_json::to_string_pretty(&schema)?);
  Ok(())
}

fn run_from_path(
  _spec_path: &Path,
  _common: &cli::CommonArgs,
  _sandbox: &cli::SandboxArgs,
) -> CrylResult<()> {
  // TODO: Implement full execution pipeline
  println!("TODO: Run from path");
  Ok(())
}

fn run_from_stdin(
  format: &str,
  _common: &cli::CommonArgs,
  _sandbox: &cli::SandboxArgs,
) -> CrylResult<()> {
  let format = Format::parse(format)?;

  let mut content = String::new();
  io::stdin().read_to_string(&mut content)?;

  let _spec: Specification = deserialize(&content, format)?;

  // TODO: Implement full execution pipeline
  println!("TODO: Run from stdin");
  Ok(())
}

fn run_import(cmd: ImportCommands) -> CrylResult<()> {
  match cmd {
    ImportCommands::Copy {
      from,
      to,
      allow_fail,
    } => importers::import_copy(&from, &to, allow_fail),
    ImportCommands::Vault { path, allow_fail } => {
      importers::import_vault(&path, allow_fail)
    }
    ImportCommands::VaultFile {
      path,
      file,
      allow_fail,
    } => importers::import_vault_file(&path, &file, allow_fail),
  }
}

fn run_generate(cmd: GenerateCommands) -> CrylResult<()> {
  match cmd {
    GenerateCommands::Copy { from, to, renew } => {
      generators::generate_copy(&from, &to, renew)
    }
    GenerateCommands::Text { name, text, renew } => {
      generators::generate_text(&name, &text, renew)
    }
    GenerateCommands::Data {
      name,
      in_format,
      data,
      out_format,
      renew,
    } => {
      generators::generate_data(&name, &in_format, &data, &out_format, renew)
    }
    GenerateCommands::Id {
      name,
      length,
      renew,
    } => generators::generate_id(&name, length, renew),
    GenerateCommands::Key {
      name,
      length,
      renew,
    } => generators::generate_key(&name, length, renew),
    GenerateCommands::Pin {
      name,
      length,
      renew,
    } => generators::generate_pin(&name, length, renew),
    GenerateCommands::Password {
      name,
      length,
      renew,
    } => generators::generate_password(&name, length, renew),
    GenerateCommands::PasswordArgon2 {
      public,
      private,
      length,
      renew,
    } => generators::generate_password_argon2(&public, &private, length, renew),
    GenerateCommands::PasswordCrypt3 {
      public,
      private,
      length,
      renew,
    } => generators::generate_password_crypt3(&public, &private, length, renew),
    GenerateCommands::AgeKey {
      public,
      private,
      renew,
    } => generators::generate_age_key(&public, &private, renew),
    GenerateCommands::SshKey {
      name,
      public,
      private,
      password,
      renew,
    } => generators::generate_ssh_key(
      &name,
      &public,
      &private,
      password.as_deref(),
      renew,
    ),
    GenerateCommands::WireguardKey {
      public,
      private,
      renew,
    } => generators::generate_wireguard_key(&public, &private, renew),
    GenerateCommands::KeySplit {
      key,
      prefix,
      threshold,
      shares,
      renew,
    } => {
      generators::generate_key_split(&key, &prefix, threshold, shares, renew)
    }
    GenerateCommands::KeyCombine {
      shares,
      key,
      threshold,
      renew,
    } => generators::generate_key_combine(&shares, &key, threshold, renew),
    GenerateCommands::TlsRoot {
      common_name,
      organization,
      config,
      private,
      public,
      pathlen,
      days,
      renew,
    } => generators::generate_tls_root(
      &common_name,
      &organization,
      &config,
      &private,
      &public,
      pathlen,
      days,
      renew,
    ),
    GenerateCommands::TlsIntermediary {
      common_name,
      organization,
      config,
      request_config,
      private,
      request,
      ca_public,
      ca_private,
      serial,
      public,
      pathlen,
      days,
      renew,
    } => generators::generate_tls_intermediary(
      &common_name,
      &organization,
      &config,
      &request_config,
      &private,
      &request,
      &ca_public,
      &ca_private,
      &serial,
      &public,
      pathlen,
      days,
      renew,
    ),
    GenerateCommands::TlsLeaf {
      common_name,
      organization,
      sans,
      config,
      request_config,
      private,
      request,
      ca_public,
      ca_private,
      serial,
      public,
      days,
      renew,
    } => generators::generate_tls_leaf(
      &common_name,
      &organization,
      &sans,
      &config,
      &request_config,
      &private,
      &request,
      &ca_public,
      &ca_private,
      &serial,
      &public,
      days,
      renew,
    ),
    GenerateCommands::TlsRsaRoot {
      common_name,
      organization,
      config,
      private,
      public,
      pathlen,
      days,
      renew,
    } => generators::generate_tls_rsa_root(
      &common_name,
      &organization,
      &config,
      &private,
      &public,
      pathlen,
      days,
      renew,
    ),
    GenerateCommands::TlsRsaIntermediary {
      common_name,
      organization,
      config,
      request_config,
      private,
      request,
      ca_public,
      ca_private,
      serial,
      public,
      pathlen,
      days,
      renew,
    } => generators::generate_tls_rsa_intermediary(
      &common_name,
      &organization,
      &config,
      &request_config,
      &private,
      &request,
      &ca_public,
      &ca_private,
      &serial,
      &public,
      pathlen,
      days,
      renew,
    ),
    GenerateCommands::TlsRsaLeaf {
      common_name,
      organization,
      sans,
      config,
      request_config,
      private,
      request,
      ca_public,
      ca_private,
      serial,
      public,
      days,
      renew,
    } => generators::generate_tls_rsa_leaf(
      &common_name,
      &organization,
      &sans,
      &config,
      &request_config,
      &private,
      &request,
      &ca_public,
      &ca_private,
      &serial,
      &public,
      days,
      renew,
    ),
    GenerateCommands::TlsDhparam { name, renew } => {
      generators::generate_tls_dhparam(&name, renew)
    }
    GenerateCommands::NebulaCa {
      name,
      public,
      private,
      days,
      renew,
    } => generators::generate_nebula_ca(&name, &public, &private, days, renew),
    GenerateCommands::NebulaCert {
      ca_public,
      ca_private,
      name,
      ip,
      public,
      private,
      renew,
    } => generators::generate_nebula_cert(
      &ca_public,
      &ca_private,
      &name,
      &ip,
      &public,
      &private,
      renew,
    ),
    GenerateCommands::CockroachCa {
      public,
      private,
      renew,
    } => generators::generate_cockroach_ca(&public, &private, renew),
    GenerateCommands::CockroachNodeCert {
      ca_public,
      ca_private,
      public,
      private,
      hosts,
      renew,
    } => generators::generate_cockroach_node_cert(
      &ca_public,
      &ca_private,
      &public,
      &private,
      &hosts,
      renew,
    ),
    GenerateCommands::CockroachClientCert {
      ca_public,
      ca_private,
      public,
      private,
      user,
      renew,
    } => generators::generate_cockroach_client_cert(
      &ca_public,
      &ca_private,
      &public,
      &private,
      &user,
      renew,
    ),
    GenerateCommands::Env {
      name,
      format,
      vars,
      renew,
    } => generators::generate_env(&name, &format, &vars, renew),
    GenerateCommands::Mustache {
      name,
      format,
      variables_and_template,
      renew,
    } => generators::generate_mustache(
      &name,
      &format,
      &variables_and_template,
      renew,
    ),
    GenerateCommands::Script { name, text, renew } => {
      generators::generate_script(&name, &text, renew)
    }
    GenerateCommands::Sops {
      age,
      public,
      private,
      format,
      values,
      renew,
    } => generators::generate_sops(
      &age, &public, &private, &format, &values, renew,
    ),
  }
}

fn run_export(cmd: ExportCommands) -> CrylResult<()> {
  match cmd {
    ExportCommands::Copy { from, to } => exporters::export_copy(&from, &to),
    ExportCommands::Vault { path } => exporters::export_vault(&path),
    ExportCommands::VaultFile { path, file } => {
      exporters::export_vault_file(&path, &file)
    }
  }
}
