#[cfg(test)]
mod test {
  use crate::generators::{
    generate_cockroach_ca, generate_nebula_ca, generate_tls_root,
    generate_tls_rsa_root,
  };
  use std::{
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
  };
  use tempfile::TempDir;
  use testcontainers::{
    ContainerAsync, GenericImage, ImageExt, runners::AsyncRunner,
  };
  use tokio::net::TcpStream;

  pub async fn vault_container(
    root_token: &str,
  ) -> anyhow::Result<ContainerAsync<GenericImage>> {
    let container = GenericImage::new("hashicorp/vault", "1.14")
      .with_env_var("VAULT_DEV_ROOT_TOKEN_ID", root_token)
      .with_exposed_host_port(8200)
      .start()
      .await?;

    let host_port = container.get_host_port_ipv4(8200).await?;
    let addr = format!("127.0.0.1:{}", host_port);

    let start = Instant::now();
    let timeout = Duration::from_secs(30);

    loop {
      match TcpStream::connect(&addr).await {
        Ok(_) => break,
        Err(_) if start.elapsed() < timeout => {
          tokio::time::sleep(Duration::from_millis(100)).await;
          continue;
        }
        Err(e) => {
          return Err(anyhow::anyhow!(
            "Vault port never became reachable: {}",
            e
          ));
        }
      }
    }

    let vault_addr = format!("http://{}", addr);
    let client = reqwest::Client::new();
    let health_timeout = Duration::from_secs(10);

    loop {
      match client
        .get(format!("{}/v1/sys/health", vault_addr))
        .timeout(health_timeout)
        .send()
        .await
      {
        Ok(resp) if resp.status().is_success() => break,
        _ if start.elapsed() < timeout => {
          tokio::time::sleep(Duration::from_millis(500)).await;
          continue;
        }
        _ => return Err(anyhow::anyhow!("Vault health check never passed")),
      }
    }

    #[allow(unsafe_code, reason = "Tested in serial tests")]
    unsafe {
      std::env::set_var("VAULT_ADDR", &vault_addr);
      std::env::set_var("VAULT_TOKEN", root_token);
      std::env::set_var("VAULT_SKIP_VERIFY", "true");
    }

    Command::new("vault")
      .args(["secrets", "enable", "-path=kv", "kv-v2"])
      .output()?;

    Ok(container)
  }

  pub struct TempCurrentDir {
    dir: TempDir,
    cwd: PathBuf,
  }

  impl TempCurrentDir {
    pub fn new() -> std::io::Result<Self> {
      let dir = TempDir::new()?;
      let cwd = std::env::current_dir()?;
      std::env::set_current_dir(dir.path())?;
      Ok(Self { dir, cwd })
    }

    pub fn path(&self) -> &Path {
      self.dir.path()
    }
  }

  impl Drop for TempCurrentDir {
    fn drop(&mut self) {
      if let Err(error) = std::env::set_current_dir(&self.cwd) {
        eprintln!("Failed to restore cwd: {error}");
      }
    }
  }

  pub fn is_ci() -> bool {
    std::env::var("CI").is_ok()
  }

  pub fn mock_cockroach_ca(
    temp: &Path,
  ) -> anyhow::Result<(std::path::PathBuf, std::path::PathBuf)> {
    let ca_public = temp.join("ca.crt");
    let ca_private = temp.join("ca.key");

    generate_cockroach_ca(&ca_public, &ca_private, true)?;

    Ok((ca_public, ca_private))
  }

  pub fn mock_nebula_ca(
    temp: &Path,
  ) -> anyhow::Result<(std::path::PathBuf, std::path::PathBuf)> {
    let ca_public_path = temp.join("ca.crt");
    let ca_private_path = temp.join("ca.key");
    generate_nebula_ca(
      "Test CA",
      &ca_public_path,
      &ca_private_path,
      3650,
      true,
    )?;

    Ok((ca_public_path, ca_private_path))
  }

  pub fn mock_tls_ca(
    temp: &TempDir,
  ) -> anyhow::Result<(std::path::PathBuf, std::path::PathBuf)> {
    let ca_config = temp.path().join("ca.conf");
    let ca_private = temp.path().join("ca.key");
    let ca_public = temp.path().join("ca.crt");

    generate_tls_root(
      "Test Root CA",
      "Test Org",
      &ca_config,
      &ca_private,
      &ca_public,
      -1,
      3650,
      true,
    )?;

    Ok((ca_public, ca_private))
  }

  pub fn mock_tls_rsa_ca(
    temp: &TempDir,
  ) -> anyhow::Result<(std::path::PathBuf, std::path::PathBuf)> {
    let ca_config = temp.path().join("ca.conf");
    let ca_private = temp.path().join("ca.key");
    let ca_public = temp.path().join("ca.crt");

    generate_tls_rsa_root(
      "Test Root CA",
      "Test Org",
      &ca_config,
      &ca_private,
      &ca_public,
      -1,
      3650,
      true,
    )?;

    Ok((ca_public, ca_private))
  }
}

#[cfg(test)]
pub use test::*;
