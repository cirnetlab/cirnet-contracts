# Versioned assignment contracts

English · [Tiếng Việt](../vi/assignment.md)

[Documentation index](index.md)

[AssignmentDefinition 1.0](../../contracts/assignment/assignment.schema.json) uses JSON Schema 2020-12 and shares definitions with [Physical Lab 1.0](../../contracts/physical-lab/lab.schema.json). The semantic validator in `contracts.validation` requires unique LabEndpoint IDs. `HYBRID` is the recommended mode for the original definition. Python service consumers use the installed Contracts package.

The legacy definition remains available. [AssignmentDefinitionV2 2.0](../../contracts/public/assignments-v2.schema.json) is additive and accepts CircuitGraph 1.2 reference graphs. References and assessment do not gate safety. Graph, assignment, Physical Lab and Hardware RPC versions are independent. See the [original assignment note](../../contracts/assignment/README.md) and [versioned placement and assignments](../../contracts/design-models/flexible-placement.md).
