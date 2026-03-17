# cryl

`cryl` is a high-performance, sandboxed CLI tool for generating, encrypting, and
managing infrastructure secrets. It allows you to orchestrate the lifecycle of
secrets—from importing and generation to encrypted export—using a declarative,
versioned specification.

`cryl` is built for security-first environments, automatically isolating
sensitive generation processes within a `bubblewrap` sandbox.

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
