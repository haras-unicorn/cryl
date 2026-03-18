use std::path::Path;

use crate::common::CrylResult;

pub fn generate_tls_dhparam(_name: &Path, _renew: bool) -> CrylResult<()> {
  Ok(())
}
