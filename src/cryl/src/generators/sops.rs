use std::path::Path;

use crate::common::CrylResult;

pub fn generate_sops(
  _age: &Path,
  _public: &Path,
  _private: &Path,
  _format: &str,
  _values: &Path,
  _renew: bool,
) -> CrylResult<()> {
  Ok(())
}
