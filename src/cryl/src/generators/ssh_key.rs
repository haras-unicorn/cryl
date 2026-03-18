use std::path::Path;

use crate::common::CrylResult;

pub fn generate_ssh_key(
  _name: &str,
  _public: &Path,
  _private: &Path,
  _password: Option<&Path>,
  _renew: bool,
) -> CrylResult<()> {
  Ok(())
}
