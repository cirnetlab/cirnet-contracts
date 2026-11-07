# Contract reference map

English · [Tiếng Việt](../vi/contract-reference.md)

[Documentation index](index.md)

The [resource manifest](../../contracts/manifest.json) records identities, versions, ownership and visibility. Schema URIs use `https://cirnet.local/`. The offline registry also resolves legacy `https://netcircuit.local/` URIs without fetching schemas over the network.

| Topic | Current boundary and reference |
| --- | --- |
| CircuitGraph | 1.0 and 1.1 remain readable. Additive 1.2 supports flexible assemblies and pinned descriptors. See [CircuitGraph](api/circuit-graph.md) and [flexible placement](../../contracts/design-models/flexible-placement.md). |
| Assignment and physical lab | Assignment 1.0 remains available alongside AssignmentDefinitionV2 2.0. Physical Lab 1.0 has its own version. See [assignment](assignment.md) and [Physical Lab](api/physical-lab.md). |
| Public API and real-time | Remote FastAPI routes own operation registration. The [API guide](../../contracts/api/README.md) describes legacy operations and additive versioned paths. |
| Capture | [NCAP formats](api/ncap.md) describe v1 digital event and v2 sampled formats. Validate layout and limits independently of application version. |
| Editor and memory | See [editor descriptors and properties](../../contracts/design-models/README.md), [catalog authoring](catalog.md) and [memory images](memory.md). |
| Hardware RPC | [RPC 2.0](hardware-rpc.md) uses `cirnet.hardware.v2`. Bindings register the legacy route and retry only after `UNIMPLEMENTED`. The wheel build hook generates v2 bindings. |
| FPGA envelope | The [control and protocol guide](fpga-protocol.md) describes logical `1.0-scaffold` and conceptual `1.0-reserved`. Physical framing and firmware require hardware evidence. |
| Compatibility | [Frozen anchors](api/compatibility.md) cover supported public wire, schema and NCAP shapes. Add separately versioned contracts for breaking changes. |

CircuitGraph, assignment, Lab, RPC, NCAP and FPGA versions evolve independently from the distribution version. A schema or transport check does not establish electrical safety or physical availability.

## Rust implementation

The `cirnet-contracts-native` crate resolves both schema URI hosts offline from an explicit source root and exposes the canonical and legacy Hardware RPC descriptors. Its NCAP module returns sampled frame time as an exact decimal numerator and integer denominator. Input strings use Unicode scalar values. [Build, CLI and parity guide](native.md)

## Resource ownership

- [Device Library](../../device-library/README.md) is the editable catalog source. Generation prepares the `contracts/catalog-source/` package mirror and the catalog projections together. Installed wheels read that mirror.
- [Public SDK resources](../../packages/public-sdk/README.md) include public validators, operations and fixtures. `contracts/browser/artifacts.json` maps legacy snapshot paths to the installed resources exposed by `contracts.resources.generated_artifacts()`. Consumers take these artifacts from Contracts.
- [Remote's OpenAPI handoff](../../contracts/api/README.md) comes from the FastAPI operation registrations. `contracts/fingerprints.json` records separate canonical-source, public-schema and OpenAPI fingerprints. The release manifest records distribution digests.
- [Test ownership](../../migration/test-ownership.json) distinguishes standalone contract tests under `tests/contracts/` from the retained consumer and Remote workflow assertions under `migration/consumer-tests/`.
