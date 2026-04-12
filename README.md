# cryl

A small tool for generating, encrypting, and managing secrets.

`cryl` allows you to create and renew secrets using a specification. The
specification contains instructions for `cryl` for how to import existing
secrets, generate or renew secrets and export those secrets in that order.

`cryl` can also be used as a Nix flake module or NixOS test module directly
integrating with `sops-nix` to allow you to generate all secrets you need for
all of your NixOS configurations and test them.

## Installation

`cryl` is distributed as a [Nix flake](https://github.com/haras-unicorn/cryl).

If you are using Nix, you can run it directly:

```bash
nix run github:haras-unicorn/cryl -- <path-to-spec>
```

Alternatively, download the standalone binary bundle from the
[Releases page](https://github.com/haras-unicorn/cryl/releases).

## Usage

`cryl` follows a three-phase execution model defined in a specification file
(`json`, `yaml`, or `toml`): **Import**, **Generate**, and **Export**.

### Modes

1. **File Input**: `cryl <path>`
2. **Standard Input**: `cat spec.toml | cryl stdin toml`

### Sandbox Security

By default, `cryl` executes tasks inside a strictly isolated sandbox. This
prevents the generation process from accessing your host filesystem, network, or
environment variables unless explicitly permitted. Use `--nosandbox` to disable
this behavior for local testing.

## Specification

The specification defines the sequence of operations. Every specification is
validated against a formal JSON
[schema](https://github.com/haras-unicorn/cryl/blob/main/src/cryl/schema.json)
to ensure correctness before execution.

```toml
[[imports]]
importer = "copy"
arguments.from = "../id"
arguments.to = "id"
arguments.allow_fail = true

[[generations]]
generator = "id"
arguments.name = "id"
arguments.length = 16

[[exports]]
exporter = "copy"
arguments.from = "id"
arguments.to = "../id"
```

## Features

- **Type-Safe**: Written in Rust to eliminate shell-injection and
  argument-parsing bugs.
- **Sandboxed**: Hardened with `bubblewrap` to prevent unauthorized side
  effects.
- **Hermetic**: Bundled with all necessary dependencies (OpenSSL, SSH, Age,
  SOPS, etc.).
- **Reproducible**: Every operation is tracked, and output file hashes are
  validated against the specification.

---

_For full documentation, configuration options, and schema references, see the
[official documentation](https://haras-unicorn.github.io/cryl/)._
