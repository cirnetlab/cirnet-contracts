# CIRNET Contracts

[English documentation](docs/en/index.md) · [Tài liệu tiếng Việt](docs/vi/index.md) · [Release changelogs](docs/changelogs/README.md)

CIRNET Contracts owns the shared schemas, device catalog sources, wire formats and generated bindings used by CIRNET. The Python distribution is `cirnet-contracts` and imports as `contracts`. The browser SDK is `@cirnet/contracts`. [CIRNET Remote](https://github.com/cirnetlab/cirnet-remote) owns public API operations, system integration and release selection.

## Source and release

| Status | Contracts 0.1.2 |
| --- | --- |
| Current source | The Python and SDK packages and the [Rust implementation](native/README.md) live in this repository. |
| Verified Rust candidate | The Stage 2 R2 matrix passed for its recorded source snapshot and artifacts. See the [native guide](docs/en/native.md#verification-evidence). |
| Selected release | The system 0.1.4 manifest pins the Contracts v0.1.2 Python wheel, sdist, npm SDK and separate allocator artifacts from source `524cc7a5e2e65f39077feefe70c7fb2ae3a773d6`. The selected manifest has no Rust artifact. [Tagged system manifest](https://github.com/cirnetlab/cirnet-remote/blob/38aea6120920f48cea817827ac2277cce4bef25e/integration/release-set.json) |

## Start here

- [Install or use the selected packages](docs/en/getting-started.md).
- [Find canonical schemas, protocols and fixtures](docs/en/contract-reference.md).
- [Build and verify the Python package and SDK](docs/en/verification.md), or [build and test Rust Contracts](docs/en/native.md).
- [Read earlier verification and migration records](docs/en/history.md).
