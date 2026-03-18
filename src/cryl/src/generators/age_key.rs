use std::path::Path;

use crate::common::CrylResult;

pub fn generate_age_key(
  _public: &Path,
  _private: &Path,
  _renew: bool,
) -> CrylResult<()> {
  Ok(())
}
