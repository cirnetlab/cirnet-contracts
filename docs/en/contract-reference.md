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

## Rust Contracts candidate

The `cirnet-contracts-native` crate reads the canonical manifest and schema
profiles from an explicit source root. It resolves both schema URI hosts
offline. It generates Rust Hardware RPC types and exposes the canonical and
legacy descriptors. The NCAP module handles digital and sampled captures
and returns sampled frame time as an exact decimal numerator and an integer
denominator. Input strings use Unicode scalar values.

Build with `cargo build --locked --release`. The toolchain is pinned in
`rust-toolchain.toml`. The `cirnet-contracts validate` command reads JSON
from standard input and prints the validated result or a structured issue.
Use `cirnet-contracts descriptor` or `cirnet-contracts descriptor legacy`
to read the corresponding binary descriptor.

Install the Node tools from the SDK lockfile in an owned external directory.
Run `cirnet-contracts generate-sdk <source-root> <output-root> <node-tools>`
to generate projections from the canonical source. Pass the generated
`contracts/browser` directory to the SDK build with `--source`, and pass
the same locked tools directory with `--node-tools`. The Rust build and
SDK generation run without Python. The optional Cargo feature
`python-tests` builds the binding used to compare against the legacy wheel.
Transition consumers continue to use the Python 0.1.2 wheel.
