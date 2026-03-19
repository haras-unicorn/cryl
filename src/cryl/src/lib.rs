//! cryl - Secret generation tool
//!
//! A high-performance, sandboxed CLI tool for generating, encrypting, and
//! managing infrastructure secrets.

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

mod common;
mod exporters;
mod generators;
mod importers;

pub use common::*;
pub use exporters::*;
pub use generators::*;
pub use importers::*;
