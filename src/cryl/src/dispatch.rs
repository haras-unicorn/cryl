use crate::common::CrylResult;
use crate::{cli::*, exporters, importers};
use crate::{generators, schema::*};
use std::path::Path;

pub fn run_import_spec(cmd: &Import) -> CrylResult<()> {
  match cmd {
    Import::Copy {
      arguments:
        CopyImportArgs {
          from,
          to,
          allow_fail,
        },
    } => importers::import_copy(
      Path::new(&from),
      Path::new(&to),
      allow_fail.unwrap_or(false),
    ),
    Import::Vault {
      arguments: VaultImportArgs { path, allow_fail },
    } => importers::import_vault(path, allow_fail.unwrap_or(false)),
    Import::VaultFile {
      arguments:
        VaultFileImportArgs {
          path,
          file,
          allow_fail,
        },
    } => importers::import_vault_file(path, file, allow_fail.unwrap_or(false)),
  }
}

pub fn run_generate_spec(cmd: &Generation) -> CrylResult<()> {
  match cmd {
    Generation::Copy {
      arguments: CopyGenArgs { from, to, renew },
    } => generators::generate_copy(
      Path::new(&from),
      Path::new(&to),
      renew.unwrap_or(false),
    ),
    Generation::Text {
      arguments: TextGenArgs { name, text, renew },
    } => {
      generators::generate_text(Path::new(&name), text, renew.unwrap_or(false))
    }
    Generation::Json {
      arguments: DataGenArgs { name, value, renew },
    } => {
      let data_path_str = format!("{}-json.json", name);
      let data_path = Path::new(&data_path_str);
      crate::common::serialize_to_file(&value, data_path)?;
      generators::generate_json(
        Path::new(&name),
        "json",
        data_path,
        renew.unwrap_or(false),
      )
    }
    Generation::Yaml {
      arguments: DataGenArgs { name, value, renew },
    } => {
      let data_path_str = format!("{}-yaml.yaml", name);
      let data_path = Path::new(&data_path_str);
      crate::common::serialize_to_file(&value, data_path)?;
      generators::generate_yaml(
        Path::new(&name),
        "yaml",
        data_path,
        renew.unwrap_or(false),
      )
    }
    Generation::Toml {
      arguments: DataGenArgs { name, value, renew },
    } => {
      let data_path_str = format!("{}-toml.toml", name);
      let data_path = Path::new(&data_path_str);
      crate::common::serialize_to_file(&value, data_path)?;
      generators::generate_toml(
        Path::new(&name),
        "toml",
        data_path,
        renew.unwrap_or(false),
      )
    }
    Generation::Id {
      arguments:
        IdGenArgs {
          name,
          length,
          renew,
        },
    } => generators::generate_id(
      Path::new(&name),
      length.unwrap_or(16),
      renew.unwrap_or(false),
    ),
    Generation::Key {
      arguments:
        IdGenArgs {
          name,
          length,
          renew,
        },
    } => generators::generate_key(
      Path::new(&name),
      length.unwrap_or(32),
      renew.unwrap_or(false),
    ),
    Generation::Pin {
      arguments:
        PinGenArgs {
          name,
          length,
          renew,
        },
    } => generators::generate_pin(
      Path::new(&name),
      length.unwrap_or(8),
      renew.unwrap_or(false),
    ),
    Generation::Password {
      arguments:
        PasswordGenArgs {
          public,
          private,
          length,
          renew,
        },
    } => generators::generate_password(
      Path::new(&public),
      Path::new(&private),
      length.unwrap_or(16) as usize,
      renew.unwrap_or(false),
    ),
    Generation::PasswordCrypt3 {
      arguments:
        PasswordGenArgs {
          public,
          private,
          length,
          renew,
        },
    } => generators::generate_password_crypt3(
      Path::new(&public),
      Path::new(&private),
      length.unwrap_or(16) as usize,
      renew.unwrap_or(false),
    ),
    Generation::AgeKey {
      arguments:
        AgeKeyArgs {
          public,
          private,
          renew,
        },
    } => generators::generate_age_key(
      Path::new(&public),
      Path::new(&private),
      renew.unwrap_or(false),
    ),
    Generation::SshKey {
      arguments:
        SshKeyArgs {
          name,
          public,
          private,
          renew,
        },
    } => generators::generate_ssh_key(
      name,
      Path::new(&public),
      Path::new(&private),
      None,
      renew.unwrap_or(false),
    ),
    Generation::WireguardKey {
      arguments:
        WireguardKeyArgs {
          public,
          private,
          renew,
        },
    } => generators::generate_wireguard_key(
      Path::new(&public),
      Path::new(&private),
      renew.unwrap_or(false),
    ),
    Generation::KeySplit {
      arguments:
        KeySplitArgs {
          key,
          prefix,
          threshold,
          shares,
          renew,
        },
    } => generators::generate_key_split(
      Path::new(&key),
      prefix,
      *threshold as usize,
      *shares as usize,
      renew.unwrap_or(false),
    ),
    Generation::KeyCombine {
      arguments:
        KeyCombineArgs {
          shares,
          key,
          threshold,
          renew,
        },
    } => generators::generate_key_combine(
      &shares.join(","),
      Path::new(&key),
      *threshold as usize,
      renew.unwrap_or(false),
    ),
    Generation::TlsRoot {
      arguments:
        TlsRootArgs {
          common_name,
          organization,
          config,
          private,
          public,
          pathlen,
          days,
          renew,
        },
    } => generators::generate_tls_root(
      common_name,
      organization,
      Path::new(&config),
      Path::new(&private),
      Path::new(&public),
      pathlen.unwrap_or(1),
      days.unwrap_or(3650),
      renew.unwrap_or(false),
    ),
    Generation::TlsIntermediary {
      arguments:
        TlsIntermediaryArgs {
          root:
            TlsRootArgs {
              common_name,
              organization,
              config,
              private,
              public,
              pathlen,
              days,
              renew,
            },
          ca_public,
          ca_private,
          request,
          request_config,
          serial,
        },
    } => generators::generate_tls_intermediary(
      common_name,
      organization,
      Path::new(&config),
      Path::new(&request_config),
      Path::new(&private),
      Path::new(&request),
      Path::new(&ca_public),
      Path::new(&ca_private),
      Path::new(&serial),
      Path::new(&public),
      pathlen.unwrap_or(0),
      days.unwrap_or(3650),
      renew.unwrap_or(false),
    ),
    Generation::TlsLeaf {
      arguments:
        TlsLeafArgs {
          inter:
            TlsIntermediaryArgs {
              root:
                TlsRootArgs {
                  common_name,
                  organization,
                  config,
                  private,
                  public,
                  pathlen,
                  days,
                  renew,
                },
              ca_public,
              ca_private,
              request,
              request_config,
              serial,
            },
          sans,
        },
    } => generators::generate_tls_leaf(
      common_name,
      organization,
      &sans.join(","),
      Path::new(&config),
      Path::new(&request_config),
      Path::new(&private),
      Path::new(&request),
      Path::new(&ca_public),
      Path::new(&ca_private),
      Path::new(&serial),
      Path::new(&public),
      days.unwrap_or(3650),
      renew.unwrap_or(false),
    ),
    Generation::TlsRsaRoot {
      arguments:
        TlsRootArgs {
          common_name,
          organization,
          config,
          private,
          public,
          pathlen,
          days,
          renew,
        },
    } => generators::generate_tls_rsa_root(
      common_name,
      organization,
      Path::new(&config),
      Path::new(&private),
      Path::new(&public),
      pathlen.unwrap_or(1),
      days.unwrap_or(3650),
      renew.unwrap_or(false),
    ),
    Generation::TlsRsaIntermediary {
      arguments:
        TlsIntermediaryArgs {
          root:
            TlsRootArgs {
              common_name,
              organization,
              config,
              private,
              public,
              pathlen,
              days,
              renew,
            },
          ca_public,
          ca_private,
          request,
          request_config,
          serial,
        },
    } => generators::generate_tls_rsa_intermediary(
      common_name,
      organization,
      Path::new(&config),
      Path::new(&request_config),
      Path::new(&private),
      Path::new(&request),
      Path::new(&ca_public),
      Path::new(&ca_private),
      Path::new(&serial),
      Path::new(&public),
      pathlen.unwrap_or(0),
      days.unwrap_or(3650),
      renew.unwrap_or(false),
    ),
    Generation::TlsRsaLeaf {
      arguments:
        TlsLeafArgs {
          inter:
            TlsIntermediaryArgs {
              root:
                TlsRootArgs {
                  common_name,
                  organization,
                  config,
                  private,
                  public,
                  pathlen,
                  days,
                  renew,
                },
              ca_public,
              ca_private,
              request,
              request_config,
              serial,
            },
          sans,
        },
    } => generators::generate_tls_rsa_leaf(
      common_name,
      organization,
      &sans.join(","),
      Path::new(&config),
      Path::new(&request_config),
      Path::new(&private),
      Path::new(&request),
      Path::new(&ca_public),
      Path::new(&ca_private),
      Path::new(&serial),
      Path::new(&public),
      days.unwrap_or(3650),
      renew.unwrap_or(false),
    ),
    Generation::TlsDhparam {
      arguments: DhparamArgs { name, renew },
    } => {
      generators::generate_tls_dhparam(Path::new(&name), renew.unwrap_or(false))
    }
    Generation::NebulaCa {
      arguments:
        NebulaCaArgs {
          name,
          public,
          private,
          days,
          renew,
        },
    } => generators::generate_nebula_ca(
      name,
      Path::new(&public),
      Path::new(&private),
      days.unwrap_or(3650),
      renew.unwrap_or(false),
    ),
    Generation::NebulaCert {
      arguments:
        NebulaCertArgs {
          ca_public,
          ca_private,
          name,
          ip,
          public,
          private,
          renew,
        },
    } => generators::generate_nebula_cert(
      Path::new(&ca_public),
      Path::new(&ca_private),
      name,
      ip,
      Path::new(&public),
      Path::new(&private),
      renew.unwrap_or(false),
    ),
    Generation::CockroachCa {
      arguments:
        CockroachCaArgs {
          public,
          private,
          renew,
        },
    } => generators::generate_cockroach_ca(
      Path::new(&public),
      Path::new(&private),
      renew.unwrap_or(false),
    ),
    Generation::CockroachNodeCert {
      arguments:
        CockroachNodeCertArgs {
          ca_public,
          ca_private,
          public,
          private,
          hosts,
          renew,
        },
    } => generators::generate_cockroach_node_cert(
      Path::new(&ca_public),
      Path::new(&ca_private),
      Path::new(&public),
      Path::new(&private),
      &hosts.join(","),
      renew.unwrap_or(false),
    ),
    Generation::CockroachClientCert {
      arguments:
        CockroachClientCertArgs {
          ca_public,
          ca_private,
          public,
          private,
          user,
          renew,
        },
    } => generators::generate_cockroach_client_cert(
      Path::new(&ca_public),
      Path::new(&ca_private),
      Path::new(&public),
      Path::new(&private),
      user,
      renew.unwrap_or(false),
    ),
    Generation::Env {
      arguments:
        EnvArgs {
          name,
          variables,
          renew,
        },
    } => {
      let vars_path_str = format!("{}-vars.json", name);
      let vars_path = Path::new(&vars_path_str);
      crate::common::serialize_to_file(&variables, vars_path)?;
      generators::generate_env(
        Path::new(&name),
        "json",
        vars_path,
        renew.unwrap_or(false),
      )
    }
    Generation::Moustache {
      arguments:
        MoustacheArgs {
          name,
          template,
          variables,
          renew,
        },
    } => {
      #[derive(serde::Serialize)]
      struct MustacheInput {
        template: String,
        variables: std::collections::HashMap<String, String>,
      }
      let input = MustacheInput {
        template: template.to_string(),
        variables: variables.to_owned(),
      };
      let input_path_str = format!("{}-input.json", name);
      let input_path = Path::new(&input_path_str);
      crate::common::serialize_to_file(&input, input_path)?;
      generators::generate_mustache(
        Path::new(&name),
        "json",
        input_path,
        renew.unwrap_or(false),
      )
    }
    Generation::Script {
      arguments: ScriptArgs { name, text, renew },
    } => generators::generate_script(
      Path::new(&name),
      text,
      renew.unwrap_or(false),
    ),
    Generation::Sops {
      arguments:
        SopsArgs {
          age,
          public,
          private,
          secrets,
          renew,
        },
    } => {
      // Serialize secrets to temp file
      let secrets_path_str = format!("{}-secrets.json", private);
      let secrets_path = Path::new(&secrets_path_str);
      crate::common::serialize_to_file(&secrets, secrets_path)?;
      generators::generate_sops(
        Path::new(&age),
        Path::new(&public),
        Path::new(&private),
        "json",
        secrets_path,
        renew.unwrap_or(false),
      )
    }
  }
}

pub fn run_export_spec(cmd: &Export) -> CrylResult<()> {
  match cmd {
    Export::Copy {
      arguments: CopyExportArgs { from, to },
    } => exporters::export_copy(Path::new(&from), Path::new(&to)),
    Export::Vault {
      arguments: VaultExportArgs { path },
    } => exporters::export_vault(path),
    Export::VaultFile {
      arguments: VaultFileExportArgs { path, file },
    } => exporters::export_vault_file(path, file),
  }
}

pub fn run_import_command(cmd: &ImportCommands) -> CrylResult<()> {
  match cmd {
    ImportCommands::Copy {
      from,
      to,
      allow_fail,
    } => importers::import_copy(from, to, *allow_fail),
    ImportCommands::Vault { path, allow_fail } => {
      importers::import_vault(path, *allow_fail)
    }
    ImportCommands::VaultFile {
      path,
      file,
      allow_fail,
    } => importers::import_vault_file(path, file, *allow_fail),
  }
}

pub fn run_generate_command(cmd: &GenerateCommands) -> CrylResult<()> {
  match cmd {
    GenerateCommands::Copy { from, to, renew } => {
      generators::generate_copy(from, to, *renew)
    }
    GenerateCommands::Text { name, text, renew } => {
      generators::generate_text(name, text, *renew)
    }
    GenerateCommands::Json {
      name,
      in_format,
      data,
      renew,
    } => generators::generate_json(name, in_format, data, *renew),
    GenerateCommands::Yaml {
      name,
      in_format,
      data,
      renew,
    } => generators::generate_yaml(name, in_format, data, *renew),
    GenerateCommands::Toml {
      name,
      in_format,
      data,
      renew,
    } => generators::generate_toml(name, in_format, data, *renew),
    GenerateCommands::Id {
      name,
      length,
      renew,
    } => generators::generate_id(name, *length, *renew),
    GenerateCommands::Key {
      name,
      length,
      renew,
    } => generators::generate_key(name, *length, *renew),
    GenerateCommands::Pin {
      name,
      length,
      renew,
    } => generators::generate_pin(name, *length, *renew),
    GenerateCommands::Password {
      public,
      private,
      length,
      renew,
    } => generators::generate_password(public, private, *length, *renew),
    GenerateCommands::PasswordCrypt3 {
      public,
      private,
      length,
      renew,
    } => generators::generate_password_crypt3(public, private, *length, *renew),
    GenerateCommands::AgeKey {
      public,
      private,
      renew,
    } => generators::generate_age_key(public, private, *renew),
    GenerateCommands::SshKey {
      name,
      public,
      private,
      password,
      renew,
    } => generators::generate_ssh_key(
      name,
      public,
      private,
      password.as_deref(),
      *renew,
    ),
    GenerateCommands::WireguardKey {
      public,
      private,
      renew,
    } => generators::generate_wireguard_key(public, private, *renew),
    GenerateCommands::KeySplit {
      key,
      prefix,
      threshold,
      shares,
      renew,
    } => {
      generators::generate_key_split(key, prefix, *threshold, *shares, *renew)
    }
    GenerateCommands::KeyCombine {
      shares,
      key,
      threshold,
      renew,
    } => generators::generate_key_combine(shares, key, *threshold, *renew),
    GenerateCommands::TlsRoot {
      common_name,
      organization,
      config,
      private,
      public,
      pathlen,
      days,
      renew,
    } => generators::generate_tls_root(
      common_name,
      organization,
      config,
      private,
      public,
      *pathlen,
      *days,
      *renew,
    ),
    GenerateCommands::TlsIntermediary {
      common_name,
      organization,
      config,
      request_config,
      private,
      request,
      ca_public,
      ca_private,
      serial,
      public,
      pathlen,
      days,
      renew,
    } => generators::generate_tls_intermediary(
      common_name,
      organization,
      config,
      request_config,
      private,
      request,
      ca_public,
      ca_private,
      serial,
      public,
      *pathlen,
      *days,
      *renew,
    ),
    GenerateCommands::TlsLeaf {
      common_name,
      organization,
      sans,
      config,
      request_config,
      private,
      request,
      ca_public,
      ca_private,
      serial,
      public,
      days,
      renew,
    } => generators::generate_tls_leaf(
      common_name,
      organization,
      sans,
      config,
      request_config,
      private,
      request,
      ca_public,
      ca_private,
      serial,
      public,
      *days,
      *renew,
    ),
    GenerateCommands::TlsRsaRoot {
      common_name,
      organization,
      config,
      private,
      public,
      pathlen,
      days,
      renew,
    } => generators::generate_tls_rsa_root(
      common_name,
      organization,
      config,
      private,
      public,
      *pathlen,
      *days,
      *renew,
    ),
    GenerateCommands::TlsRsaIntermediary {
      common_name,
      organization,
      config,
      request_config,
      private,
      request,
      ca_public,
      ca_private,
      serial,
      public,
      pathlen,
      days,
      renew,
    } => generators::generate_tls_rsa_intermediary(
      common_name,
      organization,
      config,
      request_config,
      private,
      request,
      ca_public,
      ca_private,
      serial,
      public,
      *pathlen,
      *days,
      *renew,
    ),
    GenerateCommands::TlsRsaLeaf {
      common_name,
      organization,
      sans,
      config,
      request_config,
      private,
      request,
      ca_public,
      ca_private,
      serial,
      public,
      days,
      renew,
    } => generators::generate_tls_rsa_leaf(
      common_name,
      organization,
      sans,
      config,
      request_config,
      private,
      request,
      ca_public,
      ca_private,
      serial,
      public,
      *days,
      *renew,
    ),
    GenerateCommands::TlsDhparam { name, renew } => {
      generators::generate_tls_dhparam(name, *renew)
    }
    GenerateCommands::NebulaCa {
      name,
      public,
      private,
      days,
      renew,
    } => generators::generate_nebula_ca(name, public, private, *days, *renew),
    GenerateCommands::NebulaCert {
      ca_public,
      ca_private,
      name,
      ip,
      public,
      private,
      renew,
    } => generators::generate_nebula_cert(
      ca_public, ca_private, name, ip, public, private, *renew,
    ),
    GenerateCommands::CockroachCa {
      public,
      private,
      renew,
    } => generators::generate_cockroach_ca(public, private, *renew),
    GenerateCommands::CockroachNodeCert {
      ca_public,
      ca_private,
      public,
      private,
      hosts,
      renew,
    } => generators::generate_cockroach_node_cert(
      ca_public, ca_private, public, private, hosts, *renew,
    ),
    GenerateCommands::CockroachClientCert {
      ca_public,
      ca_private,
      public,
      private,
      user,
      renew,
    } => generators::generate_cockroach_client_cert(
      ca_public, ca_private, public, private, user, *renew,
    ),
    GenerateCommands::Env {
      name,
      format,
      vars,
      renew,
    } => generators::generate_env(name, format, vars, *renew),
    GenerateCommands::Mustache {
      name,
      format,
      variables_and_template,
      renew,
    } => generators::generate_mustache(
      name,
      format,
      variables_and_template,
      *renew,
    ),
    GenerateCommands::Script { name, text, renew } => {
      generators::generate_script(name, text, *renew)
    }
    GenerateCommands::Sops {
      age,
      public,
      private,
      format,
      values,
      renew,
    } => {
      generators::generate_sops(age, public, private, format, values, *renew)
    }
  }
}

pub fn run_export_command(cmd: &ExportCommands) -> CrylResult<()> {
  match cmd {
    ExportCommands::Copy { from, to } => exporters::export_copy(from, to),
    ExportCommands::Vault { path } => exporters::export_vault(path),
    ExportCommands::VaultFile { path, file } => {
      exporters::export_vault_file(path, file)
    }
  }
}
