use std::path::Path;

use crate::common::CrylResult;

pub fn generate_key_combine(
  _shares: &str,
  _key: &Path,
  _threshold: usize,
  _renew: bool,
) -> CrylResult<()> {
  Ok(())
}
