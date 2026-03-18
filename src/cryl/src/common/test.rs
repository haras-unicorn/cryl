use std::{
  process::Command,
  time::{Duration, Instant},
};
use testcontainers::{
  runners::AsyncRunner, ContainerAsync, GenericImage, ImageExt,
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
        return Err(anyhow::anyhow!("Vault port never became reachable: {}", e))
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

  std::env::set_var("VAULT_ADDR", &vault_addr);
  std::env::set_var("VAULT_TOKEN", root_token);
  std::env::set_var("VAULT_SKIP_VERIFY", "true");

  Command::new("vault")
    .args(["secrets", "enable", "-path=kv", "kv-v2"])
    .output()?;

  Ok(container)
}
