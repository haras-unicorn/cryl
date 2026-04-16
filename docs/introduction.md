# cryl

A small tool for generating, encrypting, and managing secrets.

cryl allows you to create and renew secrets using a specification. The
specification contains instructions for cryl for how to import existing secrets,
generate or renew secrets and export those secrets in that order.

All imports, generations and exports happen in the order of execution as
specified in the specification.

## Installation

cryl is available as the default nix package of the [cryl flake]. cryl is
supported on all default systems.

## Invoking

You can invoke cryl in two ways:

1. `cryl <path>`: This tells cryl to load the specification from the given path.
   Supported formats are json, yaml and toml. cryl automatically detects the
   format of the specification via the file extension.
2. `... | cryl stdin <format>`: This tells cryl to load the specification from
   standard input. In this mode you have to tell cryl the format of the
   specification.

cryl will always take these arguments into account:

- `--dry-run`- don't run exports
- `--allow-script`- allow script generator
- `--max-imports: int = 1024`- maximum allowed imports
- `--max-generations: int = 1024`- maximum allowed generations
- `--max-exports: int = 1024`- maximum allowed exports
- `--max-specification-size: int = (1024 * 1024)`- maximum allowed specification
  size in bytes
- `--manifest-format: string = "json"` - select manifest format from 'json',
  'yaml' and 'toml'
- `--verbose` - turn on logging from modules
- `--very-verbose` - turn on logging from tools (implies verbose)
- `--envsubst` - run environment substitution on specification string before
  running

### Manifest

After each successful run, cryl creates a manifest file (`cryl-manifest.json` by
default) in the working directory. This manifest serves as an audit trail and
helps detect supply chain attacks or tampering.

The manifest contains:

- **cryl_version** - The version of cryl used
- **timestamp** - When the run occurred (ISO 8601 / RFC 3339 format)
- **cli_hash** - SHA256 hash of the CLI invocation in format
  `<command> ...<args>` (space-delimited) before sandboxing
- **working_directory_hash** - SHA256 hash of the canonicalized working
  directory before sandboxing
- **spec_hash** - SHA256 hash of the input specification
- **spec_format** - Format of the specification (json, yaml, toml)
- **tools** - Map of tool names (openssl, ssh-keygen, etc.) to their versions
  and paths
- **environment_hashes** - Map of environment variable names to their content
  hashes before sandboxing
- **output_hashes** - SHA256 hashes of all generated files

You can control manifest creation with:

- `--no-manifest` - Don't create a manifest file
- `--manifest-format` - Change the format (json, yaml, toml; default: json)

The manifest is only created when the run succeeds. No manifest is written on
error. The manifest file itself is not included in output_hashes.

### Sandbox

By default, cryl runs in a [bubblewrap] sandbox. The `--nosandbox` argument can
be provided to disable the sandbox. When cryl is running in a sandbox the
following arguments will be taken into account:

- `--ro-binds: list<string> = []`- additional read-only bind mounts to add to
  bubblewrap in format `<target and source>,<target>:<source>,...`
- `--binds: list<string> = []`- additional bind mounts to add to bubblewrap in
  format `<target and source>,<target>:<source>,...`
- `--tools: list<string> = []`- additional list of tool binaries that cryl is
  allowed to access via PATH
- `--allow-net`- allow network while running

When not in a sandbox, cryl will take these arguments into account:

- `--stay`: By default, cryl will create a temporary directory and change its
  directory to it. You can instruct cryl to stay in the directory in which it
  was invoked by passing this argument.
- `--keep`: By default, cryl will delete the contents of the working directory
  at the end of its execution. This is a safety precaution so that your
  filesystem doesn't contain secrets in plaintext for anyone to see after it is
  done with work. You can disable this behavior by passing this argument.

cryl also allows you to invoke all of the importers, generators and exporters on
their own. Please note, however, that while cryl does have safety precautions
when using it in the main ways as described here, invoking the importers,
generators and exporters by themselves is done with minimal safety precautions
which is limited to setting file permissions on generated files.

### Additional Commands

In addition to the main commands above, cryl provides several other commands:

- `cryl schema`: Print the JSON schema used to validate specifications to
  stdout. This can be useful for IDE integration or validation tools.

- `cryl import <importer> [args]`: Run a specific importer directly. This allows
  you to test any importer (copy, vault, vault-file) without a full
  specification. See the [Importers](importers.md) chapter for available
  importers and their arguments.

- `cryl generate <generator> [args]`: Run a specific generator directly. This
  allows you to test any generator (id, key, password, tls-root, etc.) without a
  full specification. See the [Generators](generators.md) chapter for available
  generators and their arguments.

- `cryl export <exporter> [args]`: Run a specific exporter directly. This allows
  you to test any exporter (copy, vault, vault-file) without a full
  specification. See the [Exporters](exporters.md) chapter for available
  exporters and their arguments.

## Specification

Here is an example of the specification in TOML format:

```toml
[[imports]]
importer = "copy"
arguments.path = "../id"
arguments.to = "id"
arguments.allow_fail = true

[[imports]]
importer = "copy"
arguments.from = "../key"
arguments.to = "key"
arguments.allow_fail = true

[[generations]]
generator = "id"
arguments.name = "id"
arguments.length = 16

[[generations]]
generator = "key"
arguments.name = "./subdir/key"
arguments.length = 32
arguments.renew = true

[[exports]]
exporter = "copy"
arguments.from = "id"
arguments.to = "../id"

[[exports]]
exporter = "copy"
arguments.from = "key"
arguments.to = "../key"
```

This specification will instruct cryl to do the following:

1. Copy the `../id` and then `../key` files into the working directory while
   allowing cryl to fail if the files do not exist (useful when generating
   secrets for the first time) time)

2. Generate the `id` file with the contents of a alphanumeric identifier of
   length 16 if it doesn't exist

3. Generate the `key` file with the contents of a alphanumeric key of length 32
   overwriting the original if it exists (renewal)

4. Copy the `id` file into `../id` and then the `key` file into `../key`
   overwriting the original files if they exist

cryl validates every specification against the [schema.json] file.

### Directory listing

Some importers/exporters/generators accept arguments of the type
`DirectoryListing`. This type is useful to tell `cryl` how to read multiple
files in the working directory (ie. to read files for SOPS, environment files,
mustache variables, etc.). This type is a tagged enum with the following
variants:

- `flat`: reads all files in the current working directory not descending into
  subdirectories. In example:

  ```toml
  [listing]
  type = "flat"
  ```

- `deep`: reads all files in the current working directory descending into
  subdirectories recursively. In example:

  ```toml
  [listing]
  type = "deep"
  ```

- `list`: reads all files in the provided list throwing an error if any are not
  present. In example:

  ```toml
  [listing]
  type = "list"
  value = [ "file1" "subdir/file2" "file3" ]
  ```

- `map`: reads all files in the provided map values throwing an error if any are
  not present. You can map certain file content to a certain key this way (ie.
  for SOPS, the keys will become keys in the private YAML file with the file
  content as the value). In example:

  ```toml
  [listing]
  type = "list"

  [listing.value]
  key1 = "file1"
  key2 = "subdir/file2"
  ```

Some generators/importers/exporters will interpret keys/paths in a flat manner
while some will interpret them in a deep manner. In example, the `vault`
exporter will export to vault following the directory structure when using
non-map variants and split keys with a forward slash (`/`) when using the map
variant. On the other hand, the SOPS generator will flatten all keys/paths using
a dash (`-`) to form kebab-cased keys in the private YAML file.

[schema.json]:
  https://github.com/haras-unicorn/cryl/blob/main/assets/schema.json
[cryl flake]: https://github.com/haras-unicorn/cryl
[bubblewrap]: https://github.com/containers/bubblewrap
