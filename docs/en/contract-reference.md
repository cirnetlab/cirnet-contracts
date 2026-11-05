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
