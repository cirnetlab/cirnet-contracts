# Contract reference map

English · [Tiếng Việt](../vi/contract-reference.md)

[Documentation index](index.md)

The [resource manifest](../../contracts/manifest.json) records identities, versions, ownership and visibility. Schema URIs use `https://cirnet.local/`. The offline registry also resolves legacy `https://netcircuit.local/` URIs without fetching schemas over the network.

The [Ecosystem Architecture](https://github.com/cirnetlab/cirnet-rules/blob/master/docs/tai-lieu-kien-tuc-he-thong/cirnetlab-architecture-en.md) assigns shared contract ownership. The [Lab-First Architecture](https://github.com/cirnetlab/cirnet-rules/blob/master/docs/tai-lieu-kien-tuc-he-thong/cirnet-architecture-lab-first-en.md) defines how Lab services use those contracts.

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

## Repository paths

| Path | Responsibility |
| --- | --- |
| [`contracts/`](../../contracts/README.md) | Canonical schema families, validation profiles, Hardware RPC protobuf and NCAP layout. The active resource manifest registers source and projection paths. The checked-in OpenAPI handoff preserves the published FastAPI profile. Remote supplies its exact export for each native SDK build. |
| [`device-library/`](../../device-library/README.md) and [`contracts/design-models/`](../../contracts/design-models/README.md) | Edit device definitions in Device Library and shared topology and mechanics in design models. The native generator projects physical parts and signal ports into the runtime catalog paths under `contracts/design-models/` and publishes browser catalogs in `contracts/browser/data/`. The [catalog-source mirror](../../contracts/catalog-source/README.md) records the historical Python package layout. |
| `src/` and `build.rs` | Rust validation, NCAP codecs, SDK generation and the CLI. The Cargo build compiles the canonical RPC proto into Prost types and an embedded descriptor in Cargo's output directory. |
| `contracts/browser/` and [`packages/public-sdk/`](../../packages/public-sdk/README.md) | Checked-in browser projections provide comparison inputs. Native generation writes a fresh browser projection from canonical sources and the selected OpenAPI export. The npm package builds from that projection. `contracts/browser/artifacts.json` documents the historical mapping of copied paths and Python package resources. |
| `contracts/codegen/`, `scripts/` and [`tools/artifact-paths/`](../../tools/artifact-paths/README.md) | The Node emitter generates TypeScript and precompiled validators from the Rust generator's bundles. The scripts build and check the native and npm packages. The allocator is a separate development package for external outputs. [Build and verification commands](native.md) |
| `contracts/fixtures/`, `contracts/design-models/fixtures/` and [`tests/fixtures/`](../../tests/fixtures/README.md) | Contract conformance inputs feed native tests and selected SDK fixtures. The test fixture directory holds parity oracles and historical browser evidence. Frozen schema and wire anchors live under [`contracts/compatibility/`](../../contracts/compatibility/README.md). |

The tracked `contracts/fingerprints.json` preserves the historical fingerprint record. Each native generation writes its own canonical-source, public-schema and normalized OpenAPI fingerprints to the output tree. The release manifest records distribution digests. [Release and OpenAPI identities](api/compatibility.md#release-and-openapi-identities)
