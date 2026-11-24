# Getting started with CIRNET Contracts

[Tiếng Việt](../vi/getting-started.md) · [Documentation](index.md)

The `cirnet-contracts-native` Rust crate validates schemas, provides Hardware RPC types and encodes NCAP. Its `cirnet-contracts` executable validates JSON, emits the protobuf descriptor and generates the SDK input. The [public SDK](../../packages/public-sdk/README.md) provides DTOs, validators, operation metadata, schema closure and conformance fixtures.

Use the [native build guide](native.md) to build version `0.3.0-dev.1` with the pinned Rust toolchain and npm lockfile. Use the exact Remote OpenAPI export and the matching Contracts crate for reproducible SDK generation. The [artifact allocator](../../tools/artifact-paths/README.md) is a separate development package for external build and test outputs.

The [compatibility reference](api/compatibility.md) records the wire and package identities.
