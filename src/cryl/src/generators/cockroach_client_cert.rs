use std::path::Path;

use crate::common::CrylResult;

pub fn generate_cockroach_client_cert(
  _ca_public: &Path,
  _ca_private: &Path,
  _public: &Path,
  _private: &Path,
  _user: &str,
  _renew: bool,
) -> CrylResult<()> {
  Ok(())
}
