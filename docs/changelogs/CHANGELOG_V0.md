# CIRNET Contracts version 0 changelog

[Changelog index](README.md)

## Unreleased

### Changed

- Standardize English major-version changelog navigation.

## v0.3.0-dev.1 — Development

Git tags and GitHub Releases provide no publication date for this development version.

### Added

- Added Windows native packaging and SDK verification with source, crate, descriptor and artifact SHA-256 records.

### Changed

- Migrated the Rust crate, canonical schemas and catalogs, Hardware RPC and NCAP, TypeScript generation and conformance fixtures.
- Replaced the installed wheel dependency with frozen Contracts 0.1.2 outcomes and a normalized Hardware RPC descriptor. Windows CI exercises the same parity cases.
- Updated the contract ownership references to the Ecosystem and CIRNET Lab-First architectures.
- Aligned the active manifest with the native NCAP source and current projection paths. Documented the owners of catalog inputs, generated resources, SDK packaging, developer tools and parity fixtures.

### Fixed

- Fixed Windows CI packaging of the development artifact allocator by passing its directory directly to npm.
- Restored installed legacy wheel parity and the development allocator’s Python bridge regression checks.

**Source**

- [Native implementation baseline](https://github.com/cirnetlab/cirnet-contracts/commit/34b0fbc9a1cc3ecec40c984607450b17d411a317)
