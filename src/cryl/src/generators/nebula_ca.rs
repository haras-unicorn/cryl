use std::path::Path;

use crate::common::CrylResult;

pub fn generate_nebula_ca(
  _name: &str,
  _public: &Path,
  _private: &Path,
  _days: u32,
  _renew: bool,
) -> CrylResult<()> {
  Ok(())
}
