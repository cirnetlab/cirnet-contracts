# @cirnet/contracts

Public CIRNET DTOs, operation metadata, capture-layout constants, precompiled
offline structural/editor/transport validators, catalog projections and fixtures.
Build version 0.1.1 alongside the exact Contracts Python artifact set.
Use the archive and SHA256 selected by the system manifest.
The source repository is [cirnetlab/cirnet-contracts](https://github.com/cirnetlab/cirnet-contracts).
Previously published archives retain their original package name and digest.

Use the root exports for DTO types and metadata, `./validators` for transport,
`./validators.editor` and `./validators.structural` for the other full profiles.
The corresponding `.core` entrypoints retain the bounded eager validator sets.
JSON resources use `./data/*.json`, `./schemas/*.json` and `./fixtures/*.json`.
Validators perform no runtime schema compilation or network reference lookup.
Application semantic adapters continue to enforce their established domain rules.
