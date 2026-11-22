# Supported compatibility anchors

English · [Tiếng Việt](../../vi/api/compatibility.md)

[Documentation index](../index.md)

## Release and OpenAPI identities

System 0.1.4 selected the published 0.1.2 wheel and SDK in the original repository generation. This repository produces the Rust crate and SDK at 0.3.0-dev.1. Remote records candidate source, filename, size and SHA-256 for each selected artifact.

| Field or domain | Meaning |
| --- | --- |
| `openapi: 3.1.0` | Specification used to read the handoff. |
| Frozen `info.version: 0.1.2` | Version of the FastAPI handoff document. Its SHA-256 is `dc39c874386eba2b58f8b5c631cbee0d63ded86cf287aea63f54e4a237ff8f08`. |
| Native `info.version: 0.2.4` | Version of the document exported by the Remote 0.2.4 candidate, derived from the frozen handoff. This export has its own digest and supported operations. |
| `/api/v2` and `operationId` | The path selects a wire variant. The case-sensitive ID identifies an operation and its generated binding. |
| DTOs, schemas and wire formats | CircuitGraph, assignment, RPC 2.0, NCAP and the FPGA envelope have independent versions. |

The two OpenAPI version fields follow separate definitions in the [OpenAPI 3.1.0 specification](https://spec.openapis.org/oas/v3.1.0.html#info-object). Remote owns the exported operations. Preserve the frozen bytes and fingerprint, including metadata. Verify the candidate SDK against the exact native export, and record both the raw export SHA-256 and packed SDK SHA-256. The generated OpenAPI fingerprint describes the normalized projection. A candidate SDK generated from the frozen FastAPI handoff has a different compatibility scope.

Use [SemVer](https://semver.org/) for the crate and npm SDK. During 0.x, each supported profile keeps its published operation IDs and declared compatibility until an explicit retirement decision. Classify a changed binding or operation ID as a source change, a changed route, DTO, MIME or protocol as a wire change, and changed authorization, errors or effects as a behavior change. Record the requirement or decision, affected producers and consumers, migration, exact artifact identities and the tests that establish the supported scope. An operation suffix such as `V2` carries no package-version requirement.

During migration, clients use the selected profile's operation binding. An unknown result of a write calls for explicit reconciliation through the documented read operation before any intentional resubmission.

The [source baseline](../../../contracts/compatibility/2026-10-16/baseline.json) preserves pre-W1 schema and protobuf source bytes and their recorded Git provenance. The [public wire baseline](../../../contracts/compatibility/2026-10-16/public-wire-baseline.json) records post-W7 public operation and schema shapes and NCAP layout anchors. Both are supported unpublished baselines. They do not establish a released ABI or hardware operation. The public snapshot records the reviewed workspace without inventing a commit or tag for the phase.

## Comparison scope

The Python and Rust generators check operation ID uniqueness across direct HTTP operations in the handoff before selecting the SDK routes. They reject unsupported operation-bearing shapes such as webhooks, callbacks and referenced path items. The wire comparison also protects the IDs and shapes already recorded for supported consumers.

Schema and operation descriptions, titles and examples are editorial. The checker traverses actual schema positions. A property named `description`, or a default or example containing schema-like keys, remains data. Manifest ownership prose is outside the wire snapshot. Booleans and numbers remain distinct. NCAP magic, prefix, byte order, records, versions and resource limits are part of the supported format.

## Name migration

Current schema hosts and the RPC v2 package use canonical CIRNET names. The Contracts 0.3.0-dev.1 native package exposes the canonical descriptor only. Offline URI aliases and the older RPC route remain part of the published comparison inputs. The source identity comparison reverses only the exact approved spelling substitutions before comparing original digests. The frozen OpenAPI retains legacy metadata alongside the canonical alias. Inconsistent alias metadata fails. Its exact health service names `cirnet-api` and `netcircuit-api` are accepted together. Other values and shapes receive no widening exemption.

The frozen handoff compares `cirnet_session` and `X-CIRNET-CSRF` as exact aliases for `netcircuit_session` and `X-CSRF-Token`. It records credential types, locations and requirements along with both capture media types. The Python profile exercised dual cookie, header and capture Accept behavior. The native candidate uses the canonical credentials and capture MIME. [Capture transport by profile](ncap.md#http-transport-by-profile)

The three current graph schema titles use `CIRNET`. Exact per-file aliases recover the original `net*CIRCUIT` titles for frozen-source digest comparison only. The preserved baseline bytes and digests remain the original evidence. See the [original compatibility reference](../../../contracts/compatibility/README.md).
