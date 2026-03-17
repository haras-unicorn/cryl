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

### Sandbox

By default, cryl runs in a [bubblewrap] sandbox. The `--nosandbox` argument can
be provided to disable the sandbox. When cryl is running in a sandbox the
following arguments will be taken into account:

- `--ro-binds: list<string> = []`- additional read-only bind mounts to add to
  bubblewrap
- `--binds: list<string> = []`- additional bind mounts to add to bubblewrap
- `--tools: list<string> = []`- additional list of tool binaries that cryl is
  allowed to access via PATH
- `--allow-net`- allow network while running
- `--timeout: int = (1000 * 60 * 60)`- maximum allowed runtime in seconds
- `--max-mem: int = (1024 * 1024 * 128)`- maximum allowed memory while running
  in bytes
- `--max-tasks: int = 64`- maximum allowed tasks while running
- `--max-file-size: int = (1024 * 1024 * 128)`- maximum allowed generated file
  size while running in bytes
- `--max-open-files: int = 1024`- maximum allowed open files while running

When not in a sandbox, cryl will take these arguments into account:

- `--stay`: By default, cryl will create a temporary directory and change its
  directory to it. You can instruct cryl to stay in the directory in which it
  was invoked by passing this argument.
- `--keep`: By default, cryl will delete the contents of the working directory
  at the end of its execution. This is a safety precaution so that your
  filesystem doesn't contain secrets in plaintext for anyone to see after it is
  done with work. You can disable this behavior by passing this argument.

cryl also allows you to invoke all of the importers, generators and exporters on
their own which will be described in the following chapters. Please note,
however, that while cryl does some have safety precautions when using it in the
main ways as described here, invoking the importers, generators and exporters by
themselves is done with minimal safety precautions which is limited to setting
file permissions on generated files.

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
arguments.name = "key"
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

[schema.json]:
  https://github.com/haras-unicorn/cryl/blob/main/assets/schema.json
[cryl flake]: https://github.com/haras-unicorn/cryl
[bubblewrap]: https://github.com/containers/bubblewrap
