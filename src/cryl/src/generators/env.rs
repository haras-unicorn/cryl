use std::path::Path;

use crate::common::CrylResult;

pub fn generate_env(
  _name: &Path,
  _format: &str,
  _vars: &Path,
  _renew: bool,
) -> CrylResult<()> {
  Ok(())
}
