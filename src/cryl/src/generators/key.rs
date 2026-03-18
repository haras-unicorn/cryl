use std::path::Path;

use crate::common::CrylResult;

pub fn generate_key(
  _name: &Path,
  _length: u32,
  _renew: bool,
) -> CrylResult<()> {
  Ok(())
}
