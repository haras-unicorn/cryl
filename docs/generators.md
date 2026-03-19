# Generators

The following are all available generators in cryl. The `type` corresponds to
the `generator` field in the specification.

## Generate Copy

Uses raw read + save to copy a file.

- Type: `copy`
- Arguments:
  - `from` (`path`): Source file path.
  - `to` (`path`): Destination file path.
  - `renew` (`boolean`, `= false`): Overwrite the destination if it exists.

## Generate Text

Writes a text file during generation.

- Type: `text`
- Arguments:
  - `name` (`path`): Destination file name.
  - `text` (`string`): Text content to write.
  - `renew` (`boolean`, `= false`): Overwrite the destination if it exists.

## JSON

Generates a JSON file from input data.

- Type: `json`
- Arguments:
  - `name` (`path`): Path to the JSON file.
  - `value` (`object`): Contents of the JSON file.
  - `renew` (`boolean`, `= false`): Whether to overwrite on subsequent runs.

## YAML

Generates a YAML file from input data.

- Type: `yaml`
- Arguments:
  - `name` (`path`): Path to the YAML file.
  - `value` (`object`): Contents of the YAML file.
  - `renew` (`boolean`, `= false`): Whether to overwrite on subsequent runs.

## TOML

Generates a TOML file from input data.

- Type: `toml`
- Arguments:
  - `name` (`path`): Path to the TOML file.
  - `value` (`object`): Contents of the TOML file.
  - `renew` (`boolean`, `= false`): Whether to overwrite on subsequent runs.

## Id

Generates a random alphanumeric id and writes it to a file.

- Type: `id`
- Arguments:
  - `name` (`path`): Destination file name for the id.
  - `length` (`int`, `= 16`): Number of characters in the id.
  - `renew` (`boolean`, `= false`): Overwrite the destination if it exists.

## Key

Generates a random alphanumeric key and writes it to a file. 🔑

- Type: `key`
- Arguments:
  - `name` (`path`): Destination file name for the key.
  - `length` (`int`, `= 32`): Number of characters in the key.
  - `renew` (`boolean`, `= false`): Overwrite the destination if it exists.

## Pin

Creates a numeric PIN with the given length and writes it to a file.

- Type: `pin`
- Arguments:
  - `name` (`path`): Destination file name for the PIN.
  - `length` (`int`, `= 8`): Number of digits in the PIN.
  - `renew` (`boolean`, `= false`): Overwrite the destination if it exists.

## Password

Generates a random alphanumeric password, hashes it with Argon2, and saves:

- plaintext to `private`
- hashed (public) to `public`

- Type: `password`
- Arguments:
  - `public` (`path`): Destination for the hashed password.
  - `private` (`path`): Destination for the plaintext password.
  - `length` (`int`, `= 8`): Number of characters in the password.
  - `renew` (`boolean`, `= false`): Overwrite existing files.

## Password-crypt-3

Generates a random alphanumeric password, hashes it with yescrypt (via
mkpasswd), and saves:

- plaintext to `private`
- hashed (public) to `public`

Note: yescrypt hashes are widely compatible with modern Linux distributions ✅

- Type: `password-crypt-3`
- Arguments:
  - `public` (`path`): Destination for the yescrypt-hashed password.
  - `private` (`path`): Destination for the plaintext password.
  - `length` (`int`, `= 8`): Number of characters in the password.
  - `renew` (`boolean`, `= false`): Overwrite existing files.

## Age-key

Generates an age key pair (using age-keygen) and saves:

- private key to `private`
- public key to `public`

Note: This key pair is typically needed when using the sops generator, since
sops uses the age public key to encrypt secrets and the private key to decrypt
them. 🔐

- Type: `age-key`
- Arguments:
  - `public` (`string`): Destination for the public key.
  - `private` (`string`): Destination for the private key.
  - `renew` (`boolean`, `= false`): Overwrite existing files.

## SSH Key

Generates an Ed25519 SSH key pair (ssh-keygen -a 100) and saves:

- private key to `private`
- public key to `public`

- Type: `ssh-key`
- Arguments:
  - `name` (`string`): Key comment stored in the public key.
  - `public` (`path`): Destination for the public key.
  - `private` (`path`): Destination for the private key.
  - `password` (`string`, `= ""`): Path to a file containing the passphrase.
    Leave empty for no passphrase.
  - `renew` (`boolean`, `= false`): Overwrite existing files.

## Wireguard Key

Generates a Wireguard key pair and saves:

- private key to `private`
- public key to `public`

- Type: `wireguard-key`
- Arguments:
  - `public` (`path`): Destination for the public key.
  - `private` (`path`): Destination for the private key.
  - `renew` (`boolean`, `= false`): Overwrite existing files.

## Key split

Splits a key using Shamir secret sharing (via ssss-split) and writes each share
to files named with the given prefix and index, like prefix-0, prefix-1, etc.

- Type: `key-split`
- Arguments:
  - `key` (`string`): Path to the key file to split (read as raw).
  - `prefix` (`string`): Prefix for each output share file.
  - `threshold` (`int`): Minimum shares needed to reconstruct.
  - `shares` (`int`): Total number of shares to create.
  - `renew` (`boolean`, `= false`): Overwrite existing files.

## Key combine

Combines Shamir shares (via ssss-combine) to reconstruct the original key and
saves it to key.

- Type: `key-combine`
- Arguments:
  - `shares` (`string`): Comma-separated paths to share files.
  - `key` (`string`): Destination path for the reconstructed key.
  - `threshold` (`int`): Number of shares required to reconstruct (must match
    what was used during split).
  - `renew` (`boolean`, `= false`): Overwrite existing file.

Note: Provide at least threshold valid shares. Fewer than that won’t reconstruct
the secret. 🧩🔐

## TLS root

Creates a Root CA:

- writes a minimal OpenSSL config to config
- generates a private key to private
- issues a self-signed root certificate to public

Algorithm: EC with curve prime256v1 (aka P‑256). 🧠⚡

- Type: `tls-root`
- Arguments:
  - `common_name` (`string`): CN for the Root CA.
  - `organization` (`string`): Organization name.
  - `config` (`path`): Where the temporary OpenSSL config is saved.
  - `private` (`path`): Destination for the private key.
  - `public` (`path`): Destination for the root certificate.
  - `pathlen` (`int`, `= 1`): Max chain depth; -1 means no limit.
  - `days` (`int`, `= 3650`): Validity period.
  - `renew` (`boolean`, `= false`): Overwrite existing files.

Notes:

- basicConstraints is set to CA:true (with pathlen if provided).
- keyUsage includes keyCertSign and cRLSign.
- Hash: sha256. ✅

## TLS intermediary

Creates an Intermediate CA signed by your Root:

- writes minimal req/ext config to config (merging request_config)
- generates a private key and CSR
- signs CSR with the Root CA, manages serial file

Algorithm: EC with curve prime256v1 (P‑256). 🌿🔐

- Type: `tls-intermediary`
- Arguments:
  - `common_name` (`string`): CN for the Intermediate CA.
  - `organization` (`string`): Organization name.
  - `config` (`path`): Output merged OpenSSL config.
  - `private` (`path`): Intermediate private key path.
  - `request` (`path`): CSR path.
  - `request_config` (`path`): Base req config to extend.
  - `ca_public` (`path`): Root certificate.
  - `ca_private` (`path`): Root private key.
  - `serial` (`path`): Serial tracking file.
  - `public` (`path`): Signed intermediate certificate.
  - `pathlen` (`int`, `= 0`): Max chain depth under this intermediate.
  - `days` (`int`, `= 3650`): Validity period.
  - `renew` (`boolean`, `= false`): Overwrite existing files.

Notes:

- basicConstraints set to CA:true (with pathlen if provided).
- keyUsage: keyCertSign, cRLSign. Hash: sha256.
- Keeps your serial tidy by temp’ing and then writing back. ✅

## TLS leaf

Issues a leaf (end-entity) certificate:

- builds req config with SANs (DNS + IP)
- generates private key and CSR
- signs with the provided issuer, manages serial file

Algorithm: EC with curve prime256v1 (P‑256). 🍃🔐

- Type: `tls-leaf`
- Arguments:
  - `common_name` (`string`): CN for the cert.
  - `organization` (`string`): Org name.
  - `sans` (`string`): Comma-separated SANs.
  - `config` (`path`): Output ext config used for signing.
  - `request_config` (`path`): Req config for CSR.
  - `private` (`path`): Private key path.
  - `request` (`path`): CSR path.
  - `ca_public` (`path`): Issuer certificate.
  - `ca_private` (`path`): Issuer private key.
  - `serial` (`path`): Serial tracking file.
  - `public` (`path`): Signed certificate output.
  - `days` (`int`, `= 3650`): Validity period.
  - `renew` (`boolean`, `= false`): Overwrite existing files.

Notes:

- basicConstraints = CA:false.
- keyUsage adjusts for RSA vs EC; extKeyUsage includes serverAuth, clientAuth.
- Hash: sha256. ✅

## TLS RSA root

Creates a Root CA (self-signed):

- writes minimal ext config for a CA
- generates a private key and self-signed cert
- sets basicConstraints with pathlen

Algorithm: RSA 4096. 🔒

- Type: tls-root (RSA)
- See: [TLS root](#tls-root) heading

## TLS RSA intermediary

Creates an Intermediate CA signed by your Root:

- builds req/ext config (merging req into ext)
- generates private key and CSR
- signs with the Root, manages serial file

Algorithm: RSA 4096. 🏗️

- Type: tls-intermediary (RSA)
- See: [TLS intermediary](#tls-intermediary) heading

## TLS RSA leaf

Issues a leaf (end-entity) certificate:

- builds req config with SANs (DNS + IP)
- generates private key and CSR
- signs with the provided issuer, manages serial file

Algorithm: RSA 4096. 🍃

- Type: tls-leaf (RSA)
- See: [TLS leaf](#tls-leaf) heading

## OpenSSL Diffie-Hellman parameters

Generates an OpenSSL Diffie-Hellman parameters file with `2048` bits.

- Type: `tls-dhparam`
- Arguments:
  - `name` (`path`): Path to the file to be generated.
  - `renew` (`boolean`, `= false`): Whether to overwrite the parameters file if
    it already exists.

## Nebula CA

Generates a Nebula Certificate Authority (CA) certificate and private key using
`nebula-cert ca`. Duration is computed as hours: `days * 24`.

- Type: `nebula-ca`
- Arguments:
  - `name` (`string`): Common name for the CA.
  - `public` (`path`): Output path for the CA certificate.
  - `private` (`path`): Output path for the CA private key.
  - `days` (`int`, `= 3650`): Certificate validity in days.
  - `renew` (`boolean`, `= false`): Whether to overwrite existing outputs.

## Nebula certificate

Generates a Nebula node certificate and private key signed by the provided
Nebula CA using `nebula-cert sign`.

- Type: `nebula-cert`
- Arguments:
  - `ca_public` (`path`): Path to the Nebula CA certificate.
  - `ca_private` (`path`): Path to the Nebula CA private key.
  - `name` (`string`): Common name for the node certificate.
  - `ip` (`string`): Node IP (CIDR or plain IP).
  - `public` (`path`): Output path for the node certificate.
  - `private` (`path`): Output path for the node private key.
  - `renew` (`boolean`, `= false`): Whether to overwrite existing outputs.

## CockroachDB CA

Generates a CockroachDB Certificate Authority (CA) certificate and private key
using `cockroach cert create-ca`.

- Type: `cockroach-ca`
- Arguments:
  - `public` (`path`): Output path for the CA certificate.
  - `private` (`path`): Output path for the CA private key.
  - `renew` (`boolean`, `= false`): Whether to overwrite existing outputs.

## CockroachDB node certificate

Generates a CockroachDB node certificate and key using
`cockroach cert create-node`, signed by the provided CA. Hosts are taken from a
comma-separated list.

- Type: `cockroach-node-cert`
- Arguments:
  - `ca_public` (`path`): Path to the CA certificate.
  - `ca_private` (`path`): Path to the CA private key.
  - `public` (`path`): Output path for the node certificate.
  - `private` (`path`): Output path for the node private key.
  - `hosts` (`string`): Comma-separated host names/IPs for SANs.
  - `renew` (`boolean`, `= false`): Whether to overwrite existing outputs.

## CockroachDB client certificate

Generates a CockroachDB client certificate and key for a specific user using
`cockroach cert create-client`, signed by the provided CA.

- Type: `cockroach-client-cert`
- Arguments:
  - `ca_public` (`path`): Path to the CA certificate.
  - `ca_private` (`path`): Path to the CA private key.
  - `public` (`path`): Output path for the client certificate.
  - `private` (`path`): Output path for the client private key.
  - `user` (`string`): CockroachDB username to embed in the cert filename and
    CN.
  - `renew` (`boolean`, `= false`): Whether to overwrite existing outputs.

## Environment file

Generates and environment (`.env`) file.

- Type: `env`
- Arguments:
  - `name` (`string`): Where to save the environment file.
  - `variables` (`object`): Variables pointing to files which to open and
    include.
  - `renew` (`boolean`, `= false`): Whether to overwrite on subsequent
    generations.

## Moustache template

Generates a populated Mustache template.

- Type: `moustache`
- Arguments:
  - `name` (`string`): Where to save the populated template.
  - `template` (`string`): The Mustache template.
  - `variables` (`object`): Variables to insert into the template.
  - `renew` (`boolean`, `= false`): Whether to overwrite on subsequent
    generations.

## Script

Generates a Nushell script and executes it.

- Type: `script`
- Arguments:
  - `name` (`string`): Where to save the script.
  - `text` (`string`): The Nushell script contents.
  - `renew` (`boolean`, `= false`): Whether to overwrite on subsequent
    generations.

Notes:

- Running this requires the `--allow-script` flag enabled for execution.

## SOPS

Generates SOPS-encrypted secrets from a key-value map.

- Type: `sops`
- Arguments:
  - `age` (`string`): Age recipient(s) used for encryption.
  - `public` (`string`): Where to save the encrypted secrets (SOPS YAML).
  - `private` (`string`): Where to save the plaintext secrets (YAML).
  - `secrets` (`object`): Secrets map to insert. Values may be inline strings or
    file paths to inline.
  - `renew` (`boolean`, default: false): Whether to overwrite on subsequent
    generations.

Notes:

- Flow:
  1. Secrets -> YAML saved to `private` (plaintext).
  2. Encrypt with
     `sops encrypt --input-type yaml --age <age> --output-type yaml`.
  3. Save encrypted output to `public`.
- `public` is saved as a public artifact; `private` contains plaintext—handle
  with care.
