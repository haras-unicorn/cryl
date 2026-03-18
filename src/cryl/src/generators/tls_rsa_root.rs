use std::path::Path;

use crate::common::CrylResult;

pub fn generate_tls_rsa_root(
  _common_name: &str,
  _organization: &str,
  _config: &Path,
  _private: &Path,
  _public: &Path,
  _pathlen: i32,
  _days: u32,
  _renew: bool,
) -> CrylResult<()> {
  Ok(())
}
