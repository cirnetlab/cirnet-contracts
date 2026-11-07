# Native Rust Contracts

[English guide](../docs/en/native.md) · [Hướng dẫn tiếng Việt](../docs/vi/native.md) · [Repository README](../README.md)

`src/` implements offline validation, NCAP codecs, Hardware RPC types and the SDK generator. `build.rs` generates both RPC descriptors from the canonical [Hardware RPC v2 proto](../contracts/hardware/v2/rpc_v2.proto). `tests/` compares Rust against the shared fixtures and, through the optional `python-tests` feature, the selected Python wheel.

Use the [native guide](../docs/en/native.md) for locked build and test commands, external output paths, and the distinction between source, verified candidate and selected artifacts.
