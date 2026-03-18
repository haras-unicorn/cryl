use std::path::Path;

use crate::common::CrylResult;

pub fn generate_nebula_cert(
  _ca_public: &Path,
  _ca_private: &Path,
  _name: &str,
  _ip: &str,
  _public: &Path,
  _private: &Path,
  _renew: bool,
) -> CrylResult<()> {
  Ok(())
}
