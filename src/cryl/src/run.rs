use crate::cli::*;
use crate::common::{deserialize, CrylError, CrylResult, Format};
use crate::dispatch::*;
use crate::schema::*;
use crate::{exporters, generators, importers};
use clap::Parser;
use schemars::schema_for;
use std::io::{self, Read};
use std::path::Path;

pub fn print_schema() -> CrylResult<()> {
  let schema = schema_for!(Specification);
  println!("{}", serde_json::to_string_pretty(&schema)?);
  Ok(())
}

pub fn run_from_path(
  spec_path: &Path,
  common: &CommonArgs,
  sandbox: &SandboxArgs,
) -> CrylResult<()> {
  let format = Format::detect_from_path(spec_path)?;
  let content = std::fs::read_to_string(spec_path)?;

  // Validate specification size
  if content.len() > common.max_specification_size {
    return Err(CrylError::Validation(format!(
      "Specification size ({} bytes) exceeds maximum allowed ({} bytes)",
      content.len(),
      common.max_specification_size
    )));
  }

  let spec: Specification = deserialize(&content, format)?;

  run(&spec, common, sandbox)?;

  Ok(())
}

pub fn run_from_stdin(
  format: &str,
  common: &CommonArgs,
  sandbox: &SandboxArgs,
) -> CrylResult<()> {
  let format = Format::parse(format)?;
  let mut content = String::new();
  io::stdin().read_to_string(&mut content)?;

  // Validate specification size
  if content.len() > common.max_specification_size {
    return Err(CrylError::Validation(format!(
      "Specification size ({} bytes) exceeds maximum allowed ({} bytes)",
      content.len(),
      common.max_specification_size
    )));
  }

  let spec: Specification = deserialize(&content, format)?;

  run(&spec, common, sandbox)?;

  Ok(())
}

fn run(
  spec: &Specification,
  common: &CommonArgs,
  sandbox: &SandboxArgs,
) -> CrylResult<()> {
  if !sandbox.nosandbox && std::env::var("CRYL_SANDBOX").is_err() {
    return run_sandbox(spec, common, sandbox);
  }

  // Validate import count
  if spec.imports.len() > common.max_imports {
    return Err(CrylError::Validation(format!(
      "Import count ({}) exceeds maximum allowed ({})",
      spec.imports.len(),
      common.max_imports
    )));
  }

  // Validate generation count
  if spec.generations.len() > common.max_generations {
    return Err(CrylError::Validation(format!(
      "Generation count ({}) exceeds maximum allowed ({})",
      spec.generations.len(),
      common.max_generations
    )));
  }

  // Validate export count
  if spec.exports.len() > common.max_exports {
    return Err(CrylError::Validation(format!(
      "Export count ({}) exceeds maximum allowed ({})",
      spec.exports.len(),
      common.max_exports
    )));
  }

  for import in spec.imports.iter() {
    run_import_spec(import)?;
  }

  for generation in spec.generations.iter() {
    run_generate_spec(generation, common.allow_script)?;
  }

  if common.dry_run {
    return Ok(());
  }

  for export in spec.exports.iter() {
    run_export_spec(export)?;
  }

  Ok(())
}

fn run_sandbox(
  _spec: &Specification,
  _common: &CommonArgs,
  _sandbox: &SandboxArgs,
) -> CrylResult<()> {
  Ok(())
}
