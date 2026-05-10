# Importers

The following are all available importers in cryl. The `type` corresponds to the
`importer` field in the specification.

## Copy

Copies files from an external directory.

- Type: `copy`
- Arguments:
  - `from` (`path`): From where to copy files.
  - `listing` (`DirectoryListing`): Determines which files will be copied into
    the working directory. This importer interprets keys deeply, meaning it will
    import subdirectories/subkeys to subdirectories in the working directory
    (ie. the value `./secret/key` with path/key `subdir/file` will be imported
    from the `key` file in the `<from>/secret` directory to the `file` file in
    the `subdir` subdirectory of the working directory).
  - `allow_fail` (`boolean`, `= false`): Allow failing to copy the file.
  - `renew` (`boolean`, `= false`): Overwrite the destination file if it exists.

## Vault

Uses [`medusa`] and [Vault] CLI to import files from [Vault].

- Type: `vault`
- Arguments:
  - `path` (`string`): [Vault] path where to load files from.
  - `listing` (`DirectoryListing`): Determines which files will be written into
    the working directory. This importer interprets keys deeply, meaning it will
    import subdirectories/subkeys to subdirectories in the working directory
    (ie. the value `./secret/key` with path/key `subdir/file` will be imported
    from the `key` key of the `<path>/secret` secret to the `file` file of the
    `subdir` subdirectory of the working directory).
  - `allow_fail` (`boolean`, `= false`): Allow failing to load files.

## Working directory

Changes the working directory optionally creating it if it doesn't exist.

- Type: `working-directory`
- Arguments:
  - `path` (`path`): Path to the new working directory (relative to current).

[`medusa`]: https://github.com/jonasvinther/medusa
[Vault]: https://www.vaultproject.io/
