# AGENTS.md

## 1. Core Architecture

- **Workspace Structure**: `cryl/src/main.rs` (entry point), `cryl/src/lib.rs`
  (common utilities), and `cryl/src/{importers,generators,exporters}/*.rs`.
- **Manifest Layout**: Omit script-source embedding. Focus on:
  - `version`: SemVer of `cryl`.
  - `environment`: Map of `{ tool_name: version_or_hash }`.
  - `spec_hash`: SHA256 of the input specification file.
  - `output_hashes`: Map of `file_path: sha256_hash`.
- **Tool Execution**: Use a `CommandBuilder` trait in `lib.rs` to standardize
  how tools are called (with/without `bwrap`).

## 2. Directory Layout

```text
src/
├── main.rs            # CLI Entry & Command Router
├── lib.rs             # Common traits, Error types, Bwrap logic
├── generators/        # tls.rs, sops.rs, ssh.rs, wireguard.rs, etc.
├── importers/         # vault.rs, copy.rs
├── exporters/         # vault.rs, copy.rs
└── tests/             # Unit and E2E integration tests
```

## 3. Development Standards

- **Clap/Serde**: All commands use `clap` structs; all manifests use `serde` for
  serialization.
- **Dry/Modular**: Common tool execution logic (e.g., `openssl` interaction,
  file saving, sandbox mounting) resides in `lib.rs`.
- **Safety**: No magic shell strings. Every argument is passed as a
  `Vec<String>` to `std::process::Command`.
- **Rust safety**: check the disallowed directives from `main.rs` - no `unwrap`
  and `panic!` allowed

## 4. Testing Requirements

- **Unit Tests**: Every generator/importer must have tests verifying the
  arguments passed to the underlying binary (mock the `Command` runner if
  necessary).
- **E2E Tests**:
  - `sandbox_test`: Run a simple generation task with `bwrap` enabled.
  - `native_test`: Run the same task without `bwrap`.
  - Verification: Assert that output file hashes match expected results.

## 5. Execution Strategy

- **Manager/Worker Pattern**:
  - If `env::var("CRYL_SANDBOX")` is empty:
    - Manager constructs `bwrap` args, mounts the current binary as
      `/proc/self/exe`, and executes.
  - If `CRYL_SANDBOX` is "1":
    - Worker runs the requested generator/importer function directly.

## 6. Documentation

- Use standard `///` docstrings for all public modules and functions.
- Avoid inline commentary; rely on descriptive function names (e.g.,
  `execute_openssl_root_ca(...)` instead of `run_openssl_command`).

## 7. Commands

Assume you are already running inside the default nix development shell.

- test: `dev-test`
- format: `dev-format`
- lint: `dev-lint`

You can also use `cargo` commands when you need something more specific (ie.
testing a specific function with `cargo test`).

A lot of tests use `/tmp` for testing but please never try to read, delete or
modify test output from there - actually never read `/tmp`. You can always just
use `println!` for debugging.
