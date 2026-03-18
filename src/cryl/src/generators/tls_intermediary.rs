use std::path::Path;

use crate::common::CrylResult;

pub fn generate_tls_intermediary(
  _common_name: &str,
  _organization: &str,
  _config: &Path,
  _request_config: &Path,
  _private: &Path,
  _request: &Path,
  _ca_public: &Path,
  _ca_private: &Path,
  _serial: &Path,
  _public: &Path,
  _pathlen: i32,
  _days: u32,
  _renew: bool,
) -> CrylResult<()> {
  Ok(())
}
