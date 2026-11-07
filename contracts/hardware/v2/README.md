# Hardware RPC 2.0

Package cirnet.hardware.v2. Generated server registration also accepts the legacy netcircuit.hardware.v2 route; synchronous stubs retry that route only on UNIMPLEMENTED. Typed InstrumentationExecutionPlan/ExecutionDecision align with physical-lab 1.0. StartExecution accepts station/experiment identity only. ConfigureVirtualCircuit is virtual-only; physical stations refuse it. Stable error enum; consumers validate domain JSON results.

The wheel build invokes `contracts.codegen.generate_hardware_rpc` with the pinned `grpcio-tools` version. Its CLI defaults to the ignored `contracts/generated` directory. A single proto and deterministic relative imports keep generation reproducible within the pinned toolchain. Use the matching `grpcio` and `protobuf` runtime versions. See the [verification guide](../../../docs/en/verification.md) for wheel and installed-binding checks.

Handlers validate ownership, state and domain payloads; server binds loopback/Unix behind FastAPI. No raw register writer. Parent v1 protos are deprecated and unavailable to new consumers.

ReadCapture uses OperationReply.capture_payload (field 5) for bounded NCAP v1 digital data; result_json carries availability/count/duration only, without samples. The API decodes/validates binary and descriptor consistency before CaptureStore persistence. 100k events, including long signal names, remain below default gRPC receive limits because names occur once in the bounded header. Unavailable capture has no binary payload. Malformed driver samples fail explicitly; malformed wire metadata/binary yields PROTOCOL_MISMATCH. Rebuild both local processes from this unpublished RPC v2 contract; an old inline-JSON producer must not be paired with the corrected consumer.
