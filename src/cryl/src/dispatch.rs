use crate::common::{CrylError, CrylResult, Format, serialize_to_file};
use crate::generators::MustacheInput;
use crate::{cli::*, exporters, importers};
use crate::{generators, schema::*};
use clap_stdin::FileOrStdin;
use std::path::{Path, PathBuf};

pub fn run_import_spec(cmd: &Import) -> CrylResult<()> {
  match cmd {
    Import::Copy {
      arguments:
        CopyImportArgs {
          from,
          listing,
          allow_fail,
        },
    } => {
      let (listing_format, listing_path) =
        ensure_written_arg(listing, &format!("{from}-listing"))?;
      importers::import_copy(
        Path::new(&from),
        listing_format,
        &listing_path,
        allow_fail.unwrap_or(false),
      )
    }
    Import::Vault {
      arguments:
        VaultImportArgs {
          path,
          listing,
          allow_fail,
        },
    } => {
      let (listing_format, listing_path) =
        ensure_written_arg(listing, &format!("{path}-listing"))?;
      importers::import_vault(
        path,
        listing_format,
        &listing_path,
        allow_fail.unwrap_or(false),
      )
    }
    Import::WorkingDirectory {
      arguments: WorkingDirectoryImportArgs { path },
    } => importers::import_working_directory(Path::new(&path)),
  }
}

pub fn run_generate_spec(
  cmd: &Generation,
  allow_script: bool,
) -> CrylResult<()> {
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
                  pathlen: _,
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
                  pathlen: _,
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
    Generation::Json {
      arguments: DataGenArgs { name, value, renew },
    } => {
      let (value_format, value_path) =
        ensure_written_arg(value, &format!("{name}-data"))?;
      generators::generate_json(
        Path::new(&name),
        value_format,
        &value_path,
        renew.unwrap_or(false),
      )
    }
    Generation::Yaml {
      arguments: DataGenArgs { name, value, renew },
    } => {
      let (value_format, value_path) =
        ensure_written_arg(value, &format!("{name}-data"))?;
      generators::generate_yaml(
        Path::new(&name),
        value_format,
        &value_path,
        renew.unwrap_or(false),
      )
    }
    Generation::Toml {
      arguments: DataGenArgs { name, value, renew },
    } => {
      let (value_format, value_path) =
        ensure_written_arg(value, &format!("{name}-data"))?;
      generators::generate_toml(
        Path::new(&name),
        value_format,
        &value_path,
        renew.unwrap_or(false),
      )
    }
    Generation::Env {
      arguments:
        EnvArgs {
          name,
          variables,
          renew,
        },
    } => {
      let (variables_format, variables_path) =
        ensure_written_arg(variables, &format!("{name}-variables"))?;
      generators::generate_env(
        Path::new(&name),
        variables_format,
        &variables_path,
        renew.unwrap_or(false),
      )
    }
    Generation::Mustache {
      arguments:
        MustacheArgs {
          name,
          template,
          listing,
          renew,
        },
    } => {
      let input = MustacheInput {
        template: template.to_string(),
        listing: listing.to_owned(),
      };
      let (input_format, input_path) =
        ensure_written_arg(&input, &format!("{name}-input"))?;
      generators::generate_mustache(
        Path::new(&name),
        input_format,
        &input_path,
        renew.unwrap_or(false),
      )
    }
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
      let (secrets_format, secrets_path) =
        ensure_written_arg(&secrets, &format!("{private}-secrets"))?;
      generators::generate_sops(
        Path::new(&age),
        Path::new(&public),
        Path::new(&private),
        secrets_format,
        &secrets_path,
        renew.unwrap_or(false),
      )
    }
    Generation::Script {
      arguments: ScriptArgs { name, text, renew },
    } => {
      if !allow_script {
        return Err(CrylError::Validation(
          "Script generator not allowed. Use --allow-script to enable."
            .to_string(),
        ));
      }
      generators::generate_script(
        Path::new(&name),
        text,
        renew.unwrap_or(false),
      )
    }
    Generation::WorkingDirectory {
      arguments: WorkingDirectoryArgs { path },
    } => generators::generate_working_directory(Path::new(&path)),
  }
}

pub fn run_export_spec(cmd: &Export) -> CrylResult<()> {
  match cmd {
    Export::Copy {
      arguments: CopyExportArgs { listing, to },
    } => {
      let (listing_format, listing_path) =
        ensure_written_arg(listing, &format!("{to}-listing"))?;

      exporters::export_copy(listing_format, &listing_path, Path::new(&to))
    }
    Export::Vault {
      arguments: VaultExportArgs { path, listing },
    } => {
      let (listing_format, listing_path) =
        ensure_written_arg(listing, &format!("{path}-listing"))?;
      exporters::export_vault(path, listing_format, &listing_path)
    }
    Export::WorkingDirectory {
      arguments: WorkingDirectoryExportArgs { path },
    } => exporters::export_working_directory(Path::new(&path)),
  }
}

pub fn run_import_command(cmd: &ImportCommands) -> CrylResult<()> {
  match cmd {
    ImportCommands::Copy {
      from,
      format,
      listing,
      allow_fail,
    } => {
      let listing_path =
        ensure_written_command(listing, *format, &format!("{from}-data"))?;
      importers::import_copy(
        Path::new(&from),
        *format,
        &listing_path,
        *allow_fail,
      )
    }
    ImportCommands::Vault {
      path,
      format,
      listing,
      allow_fail,
    } => {
      let listing_path =
        ensure_written_command(listing, *format, &format!("{path}-data"))?;
      importers::import_vault(path, *format, &listing_path, *allow_fail)
    }
    ImportCommands::WorkingDirectory { path } => {
      importers::import_working_directory(Path::new(&path))
    }
  }
}

pub fn run_generate_command(cmd: &GenerateCommands) -> CrylResult<()> {
  match cmd {
    GenerateCommands::Copy { from, to, renew } => {
      generators::generate_copy(Path::new(&from), Path::new(&to), *renew)
    }
    GenerateCommands::Text { name, text, renew } => {
      generators::generate_text(Path::new(name), text, *renew)
    }
    GenerateCommands::Id {
      name,
      length,
      renew,
    } => generators::generate_id(Path::new(name), *length, *renew),
    GenerateCommands::Key {
      name,
      length,
      renew,
    } => generators::generate_key(Path::new(name), *length, *renew),
    GenerateCommands::Pin {
      name,
      length,
      renew,
    } => generators::generate_pin(Path::new(name), *length, *renew),
    GenerateCommands::Password {
      public,
      private,
      length,
      renew,
    } => generators::generate_password(
      Path::new(public),
      Path::new(private),
      *length,
      *renew,
    ),
    GenerateCommands::PasswordCrypt3 {
      public,
      private,
      length,
      renew,
    } => generators::generate_password_crypt3(
      Path::new(public),
      Path::new(private),
      *length,
      *renew,
    ),
    GenerateCommands::AgeKey {
      public,
      private,
      renew,
    } => generators::generate_age_key(
      Path::new(public),
      Path::new(private),
      *renew,
    ),
    GenerateCommands::SshKey {
      name,
      public,
      private,
      password,
      renew,
    } => generators::generate_ssh_key(
      name,
      Path::new(public),
      Path::new(private),
      password.as_ref().map(Path::new),
      *renew,
    ),
    GenerateCommands::WireguardKey {
      public,
      private,
      renew,
    } => generators::generate_wireguard_key(
      Path::new(public),
      Path::new(private),
      *renew,
    ),
    GenerateCommands::KeySplit {
      key,
      prefix,
      threshold,
      shares,
      renew,
    } => generators::generate_key_split(
      Path::new(key),
      prefix,
      *threshold,
      *shares,
      *renew,
    ),
    GenerateCommands::KeyCombine {
      shares,
      key,
      threshold,
      renew,
    } => generators::generate_key_combine(
      shares,
      Path::new(key),
      *threshold,
      *renew,
    ),
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
      Path::new(config),
      Path::new(private),
      Path::new(public),
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
      Path::new(config),
      Path::new(request_config),
      Path::new(private),
      Path::new(request),
      Path::new(ca_public),
      Path::new(ca_private),
      Path::new(serial),
      Path::new(public),
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
      Path::new(config),
      Path::new(request_config),
      Path::new(private),
      Path::new(request),
      Path::new(ca_public),
      Path::new(ca_private),
      Path::new(serial),
      Path::new(public),
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
      Path::new(config),
      Path::new(private),
      Path::new(public),
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
      Path::new(config),
      Path::new(request_config),
      Path::new(private),
      Path::new(request),
      Path::new(ca_public),
      Path::new(ca_private),
      Path::new(serial),
      Path::new(public),
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
      Path::new(config),
      Path::new(request_config),
      Path::new(private),
      Path::new(request),
      Path::new(ca_public),
      Path::new(ca_private),
      Path::new(serial),
      Path::new(public),
      *days,
      *renew,
    ),
    GenerateCommands::TlsDhparam { name, renew } => {
      generators::generate_tls_dhparam(Path::new(name), *renew)
    }
    GenerateCommands::NebulaCa {
      name,
      public,
      private,
      days,
      renew,
    } => generators::generate_nebula_ca(
      name,
      Path::new(public),
      Path::new(private),
      *days,
      *renew,
    ),
    GenerateCommands::NebulaCert {
      ca_public,
      ca_private,
      name,
      ip,
      public,
      private,
      renew,
    } => generators::generate_nebula_cert(
      Path::new(ca_public),
      Path::new(ca_private),
      name,
      ip,
      Path::new(public),
      Path::new(private),
      *renew,
    ),
    GenerateCommands::CockroachCa {
      public,
      private,
      renew,
    } => generators::generate_cockroach_ca(
      Path::new(public),
      Path::new(private),
      *renew,
    ),
    GenerateCommands::CockroachNodeCert {
      ca_public,
      ca_private,
      public,
      private,
      hosts,
      renew,
    } => generators::generate_cockroach_node_cert(
      Path::new(ca_public),
      Path::new(ca_private),
      Path::new(public),
      Path::new(private),
      hosts,
      *renew,
    ),
    GenerateCommands::CockroachClientCert {
      ca_public,
      ca_private,
      public,
      private,
      user,
      renew,
    } => generators::generate_cockroach_client_cert(
      Path::new(ca_public),
      Path::new(ca_private),
      Path::new(public),
      Path::new(private),
      user,
      *renew,
    ),
    GenerateCommands::Json {
      name,
      format,
      data,
      renew,
    } => {
      let data_path =
        ensure_written_command(data, *format, &format!("{name}-data"))?;
      generators::generate_json(Path::new(name), *format, &data_path, *renew)
    }
    GenerateCommands::Yaml {
      name,
      format,
      data,
      renew,
    } => {
      let data_path =
        ensure_written_command(data, *format, &format!("{name}-data"))?;
      generators::generate_yaml(Path::new(name), *format, &data_path, *renew)
    }
    GenerateCommands::Toml {
      name,
      format,
      data,
      renew,
    } => {
      let data_path =
        ensure_written_command(data, *format, &format!("{name}-data"))?;
      generators::generate_toml(Path::new(name), *format, &data_path, *renew)
    }
    GenerateCommands::Env {
      name,
      format,
      variables,
      renew,
    } => {
      let variables_path = ensure_written_command(
        variables,
        *format,
        &format!("{name}-variables"),
      )?;
      generators::generate_env(
        Path::new(name),
        *format,
        &variables_path,
        *renew,
      )
    }
    GenerateCommands::Mustache {
      name,
      format,
      listing_and_template,
      renew,
    } => {
      let variables_and_template_path = ensure_written_command(
        listing_and_template,
        *format,
        &format!("{name}-variables-and-template"),
      )?;
      generators::generate_mustache(
        Path::new(name),
        *format,
        &variables_and_template_path,
        *renew,
      )
    }
    GenerateCommands::Sops {
      age,
      public,
      private,
      format,
      values,
      renew,
    } => {
      let values_path =
        ensure_written_command(values, *format, &format!("{private}-values"))?;
      generators::generate_sops(
        Path::new(age),
        Path::new(public),
        Path::new(private),
        *format,
        &values_path,
        *renew,
      )
    }
    GenerateCommands::Script { name, text, renew } => {
      generators::generate_script(Path::new(name), text, *renew)
    }
    GenerateCommands::WorkingDirectory { path } => {
      generators::generate_working_directory(Path::new(path))
    }
  }
}

pub fn run_export_command(cmd: &ExportCommands) -> CrylResult<()> {
  match cmd {
    ExportCommands::Copy {
      format,
      listing,
      to,
    } => {
      let listing_path =
        ensure_written_command(listing, *format, &format!("{to}-listing"))?;
      exporters::export_copy(*format, &listing_path, Path::new(to))
    }
    ExportCommands::Vault {
      path,
      format,
      listing,
    } => {
      let listing_path =
        ensure_written_command(listing, *format, &format!("{path}-listing"))?;
      exporters::export_vault(path, *format, &listing_path)
    }
    ExportCommands::WorkingDirectory { path } => {
      exporters::export_working_directory(Path::new(path))
    }
  }
}

fn ensure_written_arg<T: serde::Serialize>(
  value: &T,
  name: &str,
) -> CrylResult<(Format, PathBuf)> {
  const FORMAT: Format = Format::Json;
  let actual_name = format!("{}.{}", name, FORMAT.extension());
  let actual_path = Path::new(&actual_name);
  serialize_to_file(value, actual_path, Some(FORMAT))?;
  Ok((FORMAT, actual_path.to_owned()))
}

fn ensure_written_command(
  file_or_stdin: &FileOrStdin,
  format: Format,
  stdin_name: &str,
) -> CrylResult<PathBuf> {
  let file_or_stdin_path_str = if file_or_stdin.is_file() {
    file_or_stdin.filename().to_string()
  } else {
    format!("{}.{}", stdin_name, format)
  };
  let file_or_stdin_path = Path::new(&file_or_stdin_path_str);
  if file_or_stdin.is_stdin() {
    if let Some(parent) = file_or_stdin_path.parent() {
      std::fs::create_dir_all(parent)?;
    }
    std::fs::write(file_or_stdin_path, file_or_stdin.clone().contents()?)?;
  }
  Ok(file_or_stdin_path.to_owned())
}
