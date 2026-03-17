<!-- markdownlint-disable MD024 -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
and adheres to [Semantic Versioning](https://semver.org/).

## [0.1.0](https://github.com/haras-unicorn/cryl/compare/cryl-v0.0.1...cryl-v0.1.0) (2026-03-17)


### Features

* initial implementation of cryl ([7dc13b7](https://github.com/haras-unicorn/cryl/commit/7dc13b76edd88b4d6e0f1b815b626dea5bfed94d))

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
