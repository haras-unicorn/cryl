use std::path::Path;

use crate::common::CrylResult;

pub fn generate_password_argon2(
  _public: &Path,
  _private: &Path,
  _length: usize,
  _renew: bool,
) -> CrylResult<()> {
  Ok(())
}
