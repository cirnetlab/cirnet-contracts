# Circuit Graph contract

English · [Tiếng Việt](../../vi/api/circuit-graph.md)

[Documentation index](../index.md)

[CircuitGraph](../../../contracts/circuit-schema/circuit.schema.json) represents application-level logical intent. The 1.0 shape requires `schema_version`, `circuit_id`, `modules` and `connections`. The following is a minimal structural example.

~~~json
{"schema_version":"1.0","circuit_id":"empty","modules":[],"connections":[]}
~~~

Keep FPGA register addresses, Linux SPI paths and raw switching-fabric addresses out of the graph. The Hardware Service owns translation to hardware operations. Breaking schema changes require a new schema version and consumer updates.

The same schema reads 1.1 rigid physical mounting through the [module schema](../../../contracts/circuit-schema/module.schema.json). The additive [1.2 schema](../../../contracts/circuit-schema/circuit-v1.2.schema.json) supports flexible assemblies and pinned free physical descriptors. `AnySupportedCircuitGraph` selects the supported legacy or 1.2 shape. Legacy 1.0 and 1.1 readers keep their frozen version boundary. See [component placement](https://github.com/cirnetlab/cirnet-remote/blob/295797c/docs/COMPONENT_PLACEMENT.md) and [flexible placement](../../../contracts/design-models/flexible-placement.md) for persistence and API behavior.

## Legacy editor fields

Modules may include `position: {x,y,z}` in Three world coordinates, with Y as elevation and XZ as the work surface. The optional `rotation` is in degrees around world Y. These presentation fields do not change electrical meaning. The extensible module schema already permitted rotation. It gained an explicit numeric definition without changing graph version 1.0.

A [connection](../../../contracts/circuit-schema/connection.schema.json) has named endpoints.

~~~json
{"source":"moduleID.portID","destination":"moduleID.portID"}
~~~

Editor commands use ports from the [device catalog](../../../device-library/editor/components.json). They reject incompatible directions or widths, duplicate edges and additional input drivers. Fanout is permitted. Passive terminals use `inout`. Backend and simulator code analyze electrical nets. Deleting a module removes incident edges, and moving or rotating a module preserves endpoint IDs.

Local import rejects duplicate module IDs, dangling module references, nonfinite geometry and malformed local memory images. Unknown component types and port names survive import. Fallback visuals and Inspector expose them without inventing mappings. Structural acceptance by schema or API does not establish electrical correctness or execution support. See the [original graph reference](../../../contracts/circuit-schema/README.md) and the [rigid mounting example](../../../contracts/circuit-schema/examples/physical-hc08-led.json).
