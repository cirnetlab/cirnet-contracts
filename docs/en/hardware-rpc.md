# Hardware RPC 2.0

English · [Tiếng Việt](../vi/hardware-rpc.md)

[Documentation index](index.md)

The [v2 protobuf](../../contracts/hardware/v2/rpc_v2.proto) owns the current lifecycle wire contract. Its package is `cirnet.hardware.v2`. Generated server registration also serves `netcircuit.hardware.v2`. Synchronous clients retry the legacy route only after `UNIMPLEMENTED`. The retained v1 `experiment.proto`, `instrument.proto` and `station.proto` preserve original namespace and source compatibility. They do not feed current lifecycle code generation. See [resource identities](../../contracts/hardware/README.md) and [frozen evidence](../../contracts/compatibility/README.md).

`InstrumentationExecutionPlan` and `ExecutionDecision` align with Physical Lab 1.0. `StartExecution` accepts station and experiment identity. `ConfigureVirtualCircuit` is virtual-only and physical stations refuse it. The service uses the stable error enum. Handlers validate ownership, state and domain payloads. Consumers validate domain JSON results. Hardware binds loopback or Unix sockets behind FastAPI. Raw register writes are outside this API.

The wheel build hook generates the bindings from one proto and normalizes relative imports deterministically. Use matching pinned `grpcio-tools`, `grpcio` and `protobuf` versions. For a separate source diagnostic with the code generator installed, use an already owned external output path.

~~~sh
python -B -m contracts.codegen.generate_hardware_rpc --output /absolute/owned-session/rpc-bindings
~~~

`ReadCapture` places bounded digital NCAP v1 bytes in `OperationReply.capture_payload` field 5. `result_json` carries availability, count and duration without samples. Remote decodes and validates binary and descriptor consistency before CaptureStore persistence. A capture of 100,000 events stays below default gRPC receive limits because signal names occur once in the bounded header. Unavailable capture has no binary payload. Malformed driver samples fail explicitly. Malformed wire metadata or binary produces `PROTOCOL_MISMATCH`. Use both local processes built from matching bindings. An older inline-JSON producer and a consumer expecting binary capture are incompatible. See the [original RPC notes](../../contracts/hardware/v2/README.md) and [NCAP layout](../../contracts/capture/README.md).
