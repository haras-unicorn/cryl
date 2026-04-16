# Exporters

The following are all available exporters in cryl. The type corresponds to the
`exporter` field in the specification.

## Copy

Copies a file overwriting destination if exists.

- Type: `copy`
- Arguments:
  - `from` (`path`): Source file to copy.
  - `to` (`path`): Destination path.

## Vault

Exports all files in the current directory into a [Vault] KV store path using
[`medusa`].

- Type: `vault`
- Arguments:
  - `path` (`string`): Base KV path. Leading/trailing slashes are trimmed.
  - `listing` (`DirectoryListing`): Determines which files will be inserted into
    the KV store. This exporter interprets keys deeply, meaning it will export
    subdirectories/subkeys to subpaths in the KV store (ie. the key/path
    `./subdir/file1` will go under the `<path>/subdir` secret and `file1` key).

## Vault file

Sends one file's contents into [Vault] KV.

- Type: `vault-file`
- Arguments:
  - `path` (`string`): Base KV path. Slashes trimmed.
  - `file` (`string`): Local file whose content becomes the value.

## Working directory

Changes the working directory optionally creating it if it doesn't exist.

- Type: `working-directory`
- Arguments:
  - `path` (`path`): Path to the new working directory (relative to current).

[`medusa`]: https://github.com/jonasvinther/medusa
[Vault]: https://www.vaultproject.io/
