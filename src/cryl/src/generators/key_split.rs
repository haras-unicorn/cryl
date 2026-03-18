use std::path::Path;

use crate::common::CrylResult;

pub fn generate_key_split(
  _key: &Path,
  _prefix: &str,
  _threshold: usize,
  _shares: usize,
  _renew: bool,
) -> CrylResult<()> {
  Ok(())
}
