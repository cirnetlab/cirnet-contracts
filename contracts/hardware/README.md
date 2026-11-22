# Hardware RPC resource identities

`v2/rpc_v2.proto` is the current CIRNET lifecycle contract and uses the
`cirnet.hardware.v2` package. Generated server registration also serves the
legacy `netcircuit.hardware.v2` route. Synchronous clients try the canonical
route and retry the legacy route only for an unsupported method.

The deprecated `experiment.proto`, `instrument.proto`, and `station.proto`
resources preserve their original v1 namespace for legacy serialized and source
compatibility. They are not active lifecycle-codegen inputs. Frozen v2 source
and wire evidence remain in `contracts/compatibility/2026-10-16`.
