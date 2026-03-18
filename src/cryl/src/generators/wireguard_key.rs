use std::path::Path;

use crate::common::CrylResult;

pub fn generate_wireguard_key(
  _private: &Path,
  _public: &Path,
  _renew: bool,
) -> CrylResult<()> {
  Ok(())
}
