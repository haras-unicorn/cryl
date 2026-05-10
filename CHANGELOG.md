# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0](https://github.com/haras-unicorn/cryl/compare/v0.3.0...v0.4.0) - 2026-05-10

### Added

- expand listings to copy importer, copy exporter, vault importer ([#37](https://github.com/haras-unicorn/cryl/pull/37))

## [0.3.0](https://github.com/haras-unicorn/cryl/compare/v0.2.0...v0.3.0) - 2026-04-22

### Added

- *(flake)* nix modules ([#32](https://github.com/haras-unicorn/cryl/pull/32))
- make directory traversal configurable ([#30](https://github.com/haras-unicorn/cryl/pull/30))
- add working-directory importer and exporter ([#29](https://github.com/haras-unicorn/cryl/pull/29))
- *(manifest)* add working_directory_hash ([#28](https://github.com/haras-unicorn/cryl/pull/28))
- *(generators/working-directory)* add working-directory generator ([#27](https://github.com/haras-unicorn/cryl/pull/27))
- *(manifest)* add cli and environment hashes ([#26](https://github.com/haras-unicorn/cryl/pull/26))
- *(sandbox)* add asymmetric bind support ([#24](https://github.com/haras-unicorn/cryl/pull/24))
- *(cli)* add envsubst argument and environment substitution ([#23](https://github.com/haras-unicorn/cryl/pull/23))
- ensure working with subdirectories ([#21](https://github.com/haras-unicorn/cryl/pull/21))

### Fixed

- *(manifest)* recursively record output hashes ([#25](https://github.com/haras-unicorn/cryl/pull/25))

## [0.2.0](https://github.com/haras-unicorn/cryl/compare/v0.1.1...v0.2.0) - 2026-04-08

### Fixed

- fix!(generators): remove automatic trimming of generated values ([#20](https://github.com/haras-unicorn/cryl/pull/20))

### Other

- migration cleanup ([#16](https://github.com/haras-unicorn/cryl/pull/16))

## [0.1.1](https://github.com/haras-unicorn/cryl/compare/v0.1.0...v0.1.1) - 2026-03-19

### Other

- update readme location in Cargo.toml ([#12](https://github.com/haras-unicorn/cryl/pull/12))

## [0.1.0](https://github.com/haras-unicorn/cryl/releases/tag/v0.1.0) - 2026-03-19

### Added

- migrate all previous nushell functionalities ([#6](https://github.com/haras-unicorn/cryl/pull/6))
- initial implementation of cryl

### Other

- fix workflows ([#8](https://github.com/haras-unicorn/cryl/pull/8))
- *(main)* release cryl 0.1.0 ([#3](https://github.com/haras-unicorn/cryl/pull/3))
- remove changelog and just let the release-please bot manage it
- move changelog to proper path
