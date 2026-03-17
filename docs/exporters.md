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

Exports all files in the current directory into a [Vault] KV store twice:

- once to `<path>/current`
- once to `<path>/<timestamp>`

- Type: `vault`
- Arguments:
  - `path` (`string`): Base KV path. Leading/trailing slashes are trimmed.

Behavior:

- Reads all entries from the working directory (ls).
- For each file: key = basename, value = file contents (raw, trimmed).
- Emits a YAML map, then pipes it to:
  - [`medusa`] import `<path>/current` -
  - [`medusa`] import `<path>/<timestamp>` -
- Overwrites keys on the "current" path; timestamped path is append-only by
  nature.

Notes:

- Only top-level files are considered (no recursion).
- Binary files will be read raw and trimmed; stick to text files.

## Vault file

Sends one file’s contents into [Vault] KV:

- writes to `<path>/current`
- also snapshots to `<path>/<timestamp>`

- Type: `vault-file`
- Arguments:
  - `path` (`string`): Base KV path. Slashes trimmed.
  - `file` (`string`): Local file whose content becomes the value.

[`medusa`]: https://github.com/jonasvinther/medusa
[Vault]: https://www.vaultproject.io/
