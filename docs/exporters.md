# Exporters

The following are all available exporters in cryl. The type corresponds to the
`exporter` field in the specification.

## Copy

Copies files overwriting destinations if they already exist.

- Type: `copy`
- Arguments:
  - `to` (`path`): Destination path.
  - `listing` (`DirectoryListing`): Determines which files will be copied into
    the `<to>` directory. This importer interprets keys deeply, meaning it will
    export subdirectories/subkeys to subdirectories in the `<to>` directory (ie.
    the value `./secret/key` with path/key `subdir/file` will be exported to the
    `key` file in the `<to>/secret` directory from the `file` file in the
    `subdir` subdirectory of the working directory).

## Vault

Exports all files in the current directory into a [Vault] KV store path using
[`medusa`] and [Vault] CLI.

- Type: `vault`
- Arguments:
  - `path` (`string`): Base KV path. Leading/trailing slashes are trimmed.
  - `listing` (`DirectoryListing`): Determines which files will be inserted into
    the KV store. This exporter interprets keys deeply, meaning it will export
    subdirectories/subkeys to subpaths in the KV store (ie. the key/path
    `./subdir/file1` will go under the `<path>/subdir` secret and `file1` key).

## Working directory

Changes the working directory optionally creating it if it doesn't exist.

- Type: `working-directory`
- Arguments:
  - `path` (`path`): Path to the new working directory (relative to current).

[`medusa`]: https://github.com/jonasvinther/medusa
[Vault]: https://www.vaultproject.io/
