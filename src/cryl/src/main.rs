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
mod dispatch;
mod exporters;
mod generators;
mod importers;
mod manifest;
mod run;
mod schema;

use clap::Parser;
use cli::*;
use common::CrylResult;
use dispatch::*;
use run::*;

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
    Commands::Import(import_cmd) => run_import_command(&import_cmd),
    Commands::Generate(gen_cmd) => run_generate_command(&gen_cmd),
    Commands::Export(export_cmd) => run_export_command(&export_cmd),
  }
}
