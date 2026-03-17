<!-- markdownlint-disable MD024 -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
and adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- **Core**: Initial Rust implementation with modular `cryl` workspace.
- **CLI**: `schema` command for runtime JSON schema generation and validation.
- **CI/CD**: Full GitHub Actions suite (linting, docs, release-please,
  multi-arch release, cachix).
- **Packaging**: Nix Flake with `makeWrapper` for tool isolation and `makeself`
  for portable bundles.
- **Legacy**: Embedded original `rumor` script and `schema.json` as migration
  assets.
- **Docs**: README, Changelog, and Code of Conduct.
