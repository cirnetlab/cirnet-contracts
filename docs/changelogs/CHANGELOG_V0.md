# CIRNET Contracts changelog for major version 0

[Changelog index](README.md) · [Verification and migration history](../en/history.md)

## Contents

- [Unreleased](#unreleased)
- [v0.1.2](#v012)
- [v0.1.1](#v011)

## Unreleased

### Added

- Added the Rust Contracts crate and CLI for offline validation, NCAP codecs, Hardware RPC v2 descriptors and SDK generation. The Stage 2 R2 verification applies to the recorded candidate. The selected system release continues to use the v0.1.2 Python and npm artifacts. [Rust implementation](https://github.com/cirnetlab/cirnet-contracts/commit/e640f80b1199779659d3db7817fee16d13cc24c1) · [Candidate and release evidence](../en/native.md#verification-evidence)

### Changed

- Paired the current documentation in English and Vietnamese, indexed earlier verification and introduced major-version changelogs. [Documentation conventions](../en/contributing/writing-documentation.md) · [Historical records](../en/history.md)

## v0.1.2

Published **2026-11-05** at `15:52:19 +07:00`. Tag target `524cc7a`. [GitHub release](https://github.com/cirnetlab/cirnet-contracts/releases/tag/v0.1.2) · [Tagged source](https://github.com/cirnetlab/cirnet-contracts/commit/524cc7a5e2e65f39077feefe70c7fb2ae3a773d6) · [System 0.1.4 tagged manifest](https://github.com/cirnetlab/cirnet-remote/blob/38aea6120920f48cea817827ac2277cce4bef25e/integration/release-set.json)

### Changed

- Published `cirnet-contracts` for Python and `@cirnet/contracts` for npm under the CIRNET names. The Python import remains `contracts`. The offline registry resolves legacy schema URIs, and Hardware RPC v2 keeps a legacy route with fallback limited to `UNIMPLEMENTED`. [Tagged package metadata](https://github.com/cirnetlab/cirnet-contracts/blob/524cc7a5e2e65f39077feefe70c7fb2ae3a773d6/pyproject.toml) · [Compatibility anchors](../en/api/compatibility.md)
- Moved standalone CI to Node 26 and Python 3.14 while retaining the Python 3.11 lane. The SDK and standalone allocator still accept Node 22 or later. [Node verification](../en/history/verification/2026-11-05-contracts-verification.md#node-26-lts-ci-baseline-2026-11-05) · [Python verification](../en/history/verification/2026-11-05-contracts-verification.md#python-314-verification-2026-11-05)
- Published the standalone DEV allocator with separate versions, Python `1.0.0a1` and npm `1.0.0-m1.0`. The system manifest records the two allocator archives alongside the Contracts wheel, sdist and npm SDK. [Tagged artifact manifest](https://github.com/cirnetlab/cirnet-remote/blob/38aea6120920f48cea817827ac2277cce4bef25e/integration/release-set.json)

## v0.1.1

Published **2026-10-28** at `22:51:06 +07:00`. Tag target `54ef9f6`. [GitHub release](https://github.com/cirnetlab/cirnet-contracts/releases/tag/v0.1.1) · [Tagged source](https://github.com/cirnetlab/cirnet-contracts/commit/54ef9f6a11f4906c11ec9789de47cf06d5100734) · [System 0.1.1 tagged manifest](https://github.com/cirnetlab/cirnet-remote/blob/065179ead7c7cf4f44483b74edca99bb84f8f2a5/integration/release-set.json)

### Added

- Published standalone Contracts as the `netcircuit-contracts` Python wheel and sdist and the `@tinihi-labs/contracts` browser SDK. The artifacts carry the shared schemas, generated bindings and public resources under their original distribution names. [Tagged package guide](https://github.com/cirnetlab/cirnet-contracts/blob/54ef9f6a11f4906c11ec9789de47cf06d5100734/README.md) · [M6 release work](../../migration/m6-release-work.md)

### Changed

- Froze the Remote-owned OpenAPI handoff with application metadata version 0.1.1 and kept the existing wire, schema and protocol anchors. The standalone generator can export the packaged resources to an explicit destination. [OpenAPI handoff revision](https://github.com/cirnetlab/cirnet-contracts/commit/6278e477be97714311bb814767026f9f74515407) · [Compatibility anchors](../en/api/compatibility.md)

### Fixed

- Made generated resources deterministic across checkout newline settings and kept allocator CI fixtures within the owned verification session. [Deterministic generation](https://github.com/cirnetlab/cirnet-contracts/commit/295f662) · [Allocator fixture fix](https://github.com/cirnetlab/cirnet-contracts/commit/54ef9f6a11f4906c11ec9789de47cf06d5100734)
