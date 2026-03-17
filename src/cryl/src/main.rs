#![deny(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![deny(clippy::arithmetic_side_effects)]
#![deny(clippy::todo)]
#![deny(clippy::unreachable)]
#![deny(clippy::allow_attributes_without_reason)]

use clap::{Parser, Subcommand};
use schema::Specification;
use schemars::schema_for;

mod schema;

#[derive(Parser, Debug)]
#[command(name = "cryl", version, about = "Secret generation tool")]
struct Cli {
  #[command(subcommand)]
  command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
  /// Print the JSON schema to stdout
  Schema,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let cli = Cli::parse();

  match cli.command {
    Commands::Schema => {
      let schema = schema_for!(Specification);
      println!("{}", serde_json::to_string_pretty(&schema)?);
    }
  }

  Ok(())
}
