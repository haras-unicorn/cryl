use testcontainers::{
  core::ContainerPort, runners::AsyncRunner, ContainerAsync, GenericImage,
  ImageExt,
};

pub async fn vault_container(
  root_token: &str,
) -> anyhow::Result<ContainerAsync<GenericImage>> {
  Ok(
    GenericImage::new("hashicorp/vault", "1.14")
      .with_env_var("VAULT_DEV_ROOT_TOKEN_ID", root_token)
      .with_env_var("VAULT_DEV_LISTEN_ADDRESS", "0.0.0.0:8200")
      .with_mapped_port(8200, ContainerPort::Tcp(8200))
      .start()
      .await?,
  )
}
