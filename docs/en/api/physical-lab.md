# Physical Lab contracts 1.0

English · [Tiếng Việt](../../vi/api/physical-lab.md)

[Documentation index](../index.md)

The [Lab schema](../../../contracts/physical-lab/lab.schema.json) defines Physical Lab 1.0 in JSON Schema 2020-12. `DetectedTopologyGraph` preserves student wiring mistakes and opaque or ambiguous observations. Lab 1.0 is frozen independently of editor CircuitGraph 1.0, 1.1 and additive 1.2. [Versioned assignments](../assignment.md) can carry 1.2 reference graphs without rewriting Lab 1.0.

Safety, topology and functional results are independent. The schema names `SafetyVerificationResult`, `TopologyVerificationResult`, `FunctionalVerificationResult` and `ExecutionDecision` separately. `InstrumentationExecutionPlan` routes lab endpoints only. Internal student wiring remains an observation, outside the routing plan. The closed field shapes exclude raw transport controls. `contracts.validation` checks semantic endpoint uniqueness and binding.

The [v1 capabilities example](../../../contracts/physical-lab/examples/v1-target-capabilities.json) is a design target with `available: false`. Its design values, including the illustrative `2.0-scaffold` protocol label, are not runtime evidence. Runtime channel, support and slot counts come from actual drivers and station profiles. Electrical limits require evidence rather than invented values.

The pinned [FPGA proposal mapping](https://github.com/cirnetlab/cirnet-remote/blob/295797c/docs/architecture/fpga-contract-conformance.md) distinguishes the implemented `InstrumentationExecutionPlan` from the historical `PhysicalExecutionPlan` name and proposed board image. ABI, freshness and hardware gates remain independent. The mapping does not enable a physical driver. The [NCAP format reference](ncap.md) separates runtime v1 output from sampled-v2 helpers. See the [original Lab reference](../../../contracts/physical-lab/README.md).
