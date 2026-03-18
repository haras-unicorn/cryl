use std::path::Path;

use crate::common::CrylResult;

pub fn generate_mustache(
  _name: &Path,
  _format: &str,
  _variables_and_template: &Path,
  _renew: bool,
) -> CrylResult<()> {
  Ok(())
}
