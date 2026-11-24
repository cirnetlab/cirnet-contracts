# Contract reference map

English · [Tiếng Việt](../vi/contract-reference.md)

[Documentation index](index.md)

The [resource manifest](../../contracts/manifest.json) records identities, versions, ownership and visibility. Schema URIs use `https://cirnet.local/`. The offline registry also resolves legacy `https://netcircuit.local/` URIs without fetching schemas over the network.

The [Ecosystem Architecture](https://github.com/cirnetlab/cirnet-remote/blob/master/docs/cirnetlab-architecture-en.md) assigns shared contract ownership. The [Lab-First Architecture](https://github.com/cirnetlab/cirnet-remote/blob/master/docs/cirnet-architecture-lab-first-en.md) defines how Lab services use those contracts.

| Owner | Boundary |
| --- | --- |
| Contracts | Canonical CircuitGraph and other schemas, device catalog inputs, Hardware RPC and FPGA logical descriptors, NCAP formats, conformance fixtures, and Rust bindings and generated TypeScript bindings. |
| Remote | Application route registration, behavior and authorization, the OpenAPI export of each API profile, and system release selection. Contracts consumes the reviewed export for SDK generation. |
| Hardware and FPGA | Hardware owns local station behavior and private transport. FPGA owns controller RTL and board evidence. Contracts owns the shared logical shape at those boundaries. |

| Topic | Current boundary and reference |
| --- | --- |
| CircuitGraph | The graph is the logical design source of truth. Geometry rendered in 3D creates no electrical connectivity. Versions 1.0 and 1.1 remain readable. Additive 1.2 supports flexible assemblies and pinned descriptors. See [CircuitGraph](api/circuit-graph.md) and [flexible placement](../../contracts/design-models/flexible-placement.md). |
| Assignment and physical lab | Assignment 1.0 remains available alongside AssignmentDefinitionV2 2.0. Physical Lab 1.0 has its own version. See [assignment](assignment.md) and [Physical Lab](api/physical-lab.md). |
| Public API and real-time | Remote owns operation registration. The frozen FastAPI handoff preserves the Python API. The native candidate exports its supported operations for its own SDK build. See [release and OpenAPI identities](api/compatibility.md#release-and-openapi-identities). |
| Capture | [NCAP formats](api/ncap.md) describe v1 digital event and v2 sampled formats. Validate layout and limits independently of application version. |
| Editor and memory | See [editor descriptors and properties](../../contracts/design-models/README.md), [catalog authoring](catalog.md) and [memory images](memory.md). |
| Hardware RPC | [RPC 2.0](hardware-rpc.md) uses `cirnet.hardware.v2`. The published Python profile supports the legacy service name. The native profile uses the canonical service. |
| FPGA envelope | The [control and protocol guide](fpga-protocol.md) describes logical `1.0-scaffold` and conceptual `1.0-reserved`. Physical framing and firmware require hardware evidence. |
| Compatibility | [Frozen anchors](api/compatibility.md) cover supported public wire, schema and NCAP shapes. Add separately versioned contracts for breaking changes. |

CircuitGraph, assignment, Lab, RPC, NCAP and FPGA versions evolve independently from the distribution version. A schema or transport check does not establish electrical safety or physical availability.

## Rust implementation

The `cirnet-contracts-native` crate resolves both schema URI hosts offline from an explicit source root. The 0.3.0-dev.1 build exposes the canonical Hardware RPC descriptor. Its NCAP module returns sampled frame time as an exact decimal numerator and integer denominator. Input strings use Unicode scalar values. [Build, CLI and parity guide](native.md)

## Resource ownership

- [Device Library](../../device-library/README.md) is the editable catalog source. Generation prepares the `contracts/catalog-source/` package mirror and the catalog projections together. Installed wheels read that mirror.
- [Public SDK resources](../../packages/public-sdk/README.md) include public validators, operations and fixtures. `contracts/browser/artifacts.json` maps legacy snapshot paths to the installed resources exposed by `contracts.resources.generated_artifacts()`. Consumers take these artifacts from Contracts.
- [Remote's OpenAPI handoff](../../contracts/api/README.md) comes from the FastAPI operation registrations. `contracts/fingerprints.json` records separate canonical-source, public-schema and OpenAPI fingerprints. The release manifest records distribution digests.
