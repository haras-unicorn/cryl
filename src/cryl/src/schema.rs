use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct Specification {
  pub imports: Vec<Import>,
  pub generations: Vec<Generation>,
  pub exports: Vec<Export>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(tag = "importer", rename_all = "snake_case")]
pub enum Import {
  Vault { arguments: VaultImportArgs },
  VaultFile { arguments: VaultFileImportArgs },
  Copy { arguments: CopyImportArgs },
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(tag = "generator", rename_all = "kebab-case")]
pub enum Generation {
  Copy {
    arguments: CopyGenArgs,
  },
  Text {
    arguments: TextGenArgs,
  },
  Json {
    arguments: DataGenArgs,
  },
  Yaml {
    arguments: DataGenArgs,
  },
  Toml {
    arguments: DataGenArgs,
  },
  Id {
    arguments: IdGenArgs,
  },
  Key {
    arguments: IdGenArgs,
  },
  KeySplit {
    arguments: KeySplitArgs,
  },
  KeyCombine {
    arguments: KeyCombineArgs,
  },
  Pin {
    arguments: PinGenArgs,
  },
  Password {
    arguments: PasswordGenArgs,
  },
  #[serde(rename = "password-crypt-3")]
  PasswordCrypt3 {
    arguments: PasswordGenArgs,
  },
  AgeKey {
    arguments: AgeKeyArgs,
  },
  SshKey {
    arguments: SshKeyArgs,
  },
  WireguardKey {
    arguments: WireguardKeyArgs,
  },
  TlsRoot {
    arguments: TlsRootArgs,
  },
  TlsIntermediary {
    arguments: TlsIntermediaryArgs,
  },
  TlsLeaf {
    arguments: TlsLeafArgs,
  },
  TlsRsaRoot {
    arguments: TlsRootArgs,
  },
  TlsRsaIntermediary {
    arguments: TlsIntermediaryArgs,
  },
  TlsRsaLeaf {
    arguments: TlsLeafArgs,
  },
  TlsDhparam {
    arguments: DhparamArgs,
  },
  NebulaCa {
    arguments: NebulaCaArgs,
  },
  NebulaCert {
    arguments: NebulaCertArgs,
  },
  CockroachCa {
    arguments: CockroachCaArgs,
  },
  CockroachNodeCert {
    arguments: CockroachNodeCertArgs,
  },
  CockroachClientCert {
    arguments: CockroachClientCertArgs,
  },
  Env {
    arguments: EnvArgs,
  },
  Moustache {
    arguments: MoustacheArgs,
  },
  Script {
    arguments: ScriptArgs,
  },
  Sops {
    arguments: SopsArgs,
  },
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(tag = "exporter", rename_all = "snake_case")]
pub enum Export {
  Vault { arguments: VaultExportArgs },
  VaultFile { arguments: VaultFileExportArgs },
  Copy { arguments: CopyExportArgs },
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct VaultImportArgs {
  pub path: String,
  pub allow_fail: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct VaultFileImportArgs {
  pub path: String,
  pub file: String,
  pub allow_fail: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct CopyImportArgs {
  pub from: String,
  pub to: String,
  pub allow_fail: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct CopyGenArgs {
  pub from: String,
  pub to: String,
  pub renew: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct TextGenArgs {
  pub name: String,
  pub text: String,
  pub renew: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct DataGenArgs {
  pub name: String,
  pub value: serde_json::Value,
  pub renew: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct IdGenArgs {
  pub name: String,
  pub length: Option<u32>,
  pub renew: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct KeySplitArgs {
  pub key: String,
  pub prefix: String,
  pub shares: u32,
  pub threshold: u32,
  pub renew: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct KeyCombineArgs {
  pub key: String,
  pub shares: Vec<String>,
  pub threshold: u32,
  pub renew: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct PinGenArgs {
  pub name: String,
  pub length: Option<u32>,
  pub renew: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct PasswordGenArgs {
  pub public: String,
  pub private: String,
  pub length: Option<u32>,
  pub renew: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct AgeKeyArgs {
  pub public: String,
  pub private: String,
  pub renew: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct SshKeyArgs {
  pub name: String,
  pub public: String,
  pub private: String,
  pub renew: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct WireguardKeyArgs {
  pub public: String,
  pub private: String,
  pub renew: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct TlsRootArgs {
  pub common_name: String,
  pub organization: String,
  pub config: String,
  pub public: String,
  pub private: String,
  pub pathlen: Option<u32>,
  pub days: Option<u32>,
  pub renew: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct TlsIntermediaryArgs {
  #[serde(flatten)]
  pub root: TlsRootArgs,
  pub ca_public: String,
  pub ca_private: String,
  pub request: String,
  pub request_config: String,
  pub serial: String,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct TlsLeafArgs {
  #[serde(flatten)]
  pub inter: TlsIntermediaryArgs,
  pub sans: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct DhparamArgs {
  pub name: String,
  pub renew: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct NebulaCaArgs {
  pub name: String,
  pub public: String,
  pub private: String,
  pub days: Option<u32>,
  pub renew: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct NebulaCertArgs {
  pub ca_public: String,
  pub ca_private: String,
  pub name: String,
  pub ip: String,
  pub public: String,
  pub private: String,
  pub renew: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct CockroachCaArgs {
  pub public: String,
  pub private: String,
  pub renew: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct CockroachNodeCertArgs {
  pub ca_public: String,
  pub ca_private: String,
  pub hosts: Vec<String>,
  pub public: String,
  pub private: String,
  pub renew: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct CockroachClientCertArgs {
  pub ca_public: String,
  pub ca_private: String,
  pub user: String,
  pub public: String,
  pub private: String,
  pub renew: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct EnvArgs {
  pub name: String,
  pub variables: HashMap<String, String>,
  pub renew: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct MoustacheArgs {
  pub name: String,
  pub template: String,
  pub variables: HashMap<String, String>,
  pub renew: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct ScriptArgs {
  pub name: String,
  pub text: String,
  pub renew: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct SopsArgs {
  pub age: String,
  pub public: String,
  pub private: String,
  pub secrets: serde_json::Value,
  pub renew: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct VaultExportArgs {
  pub path: String,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct VaultFileExportArgs {
  pub path: String,
  pub file: String,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct CopyExportArgs {
  pub from: String,
  pub to: String,
}
