# CIRNET Contracts

Current repository: [CIRNETLAB/CIRNET-Contracts](https://github.com/CIRNETLAB/CIRNET-Contracts).
System architecture and integrated release ownership remain in
[CIRNET-Remote](https://github.com/CIRNETLAB/CIRNET-Remote).
The historical names `TINIHI-Labs/net-Circuit-Contracts` and
`net-Circuit-Remote/net-Circuit-Remote` in pinned migration/provenance records
refer to the same preserved repositories, now under `CIRNETLAB/CIRNET-*`.
Standalone product version **0.1.1** was originally extracted from Remote revision
`295797c`. The current source builds the Python distribution `cirnet-contracts`
with the import namespace `contracts`. The browser package is `@tinihi-labs/contracts`.
Schemas, revisions, protocol versions, frozen wire anchors and public operation
semantics retain their existing meaning. Build from a clean committed revision,
verify the standalone and integrated gates, then compare against immutable
`v0.1.1` artifacts with exact SHA256 and source provenance. Changed artifacts
require an explicit release decision; published bytes and tags stay preserved. Publication and
consumer access are recorded in the system release manifest; package checks
do not establish physical support.

Existing immutable [release v0.1.1](https://github.com/CIRNETLAB/CIRNET-Contracts/releases/tag/v0.1.1)
records the original `netcircuit-contracts` distribution and its published source
revision and artifact bytes. Select the renamed wheel through the system
release manifest with the corresponding consumer wheels and exact digests.

Python runtime requires 3.11+. Generator CI uses Python 3.12 and Node 26.
For reproducible CI/release verification, install the complete `requirements-build.lock` with `pip --require-hashes` from a reviewed wheelhouse; `requirements-runtime.lock` independently pins the full runtime closure for Python 3.11/3.12 on Windows AMD64 and Linux x86_64. Install `tools/artifact-paths` with `--no-index --no-build-isolation --no-deps` after the locked build tools. The standalone workflow records the exact offline commands. Then run `npm ci --no-audit --no-fund` in
`packages/public-sdk`. Generation needs no Interface, API, Simulator or Hardware
checkout:

```sh
python -m contracts.codegen.generate_contracts
python -m contracts.codegen.generate_contracts --check
python -m contracts.codegen.sync_editor_catalog --check
python -m contracts.codegen.sync_breadboard_topologies --check
python scripts/check_public_compatibility.py
python -m pytest
node packages/public-sdk/scripts/build.mjs
python -m build --no-isolation --wheel --sdist --outdir /absolute/external/artifacts
```

Before verification, allocate a session through the separately versioned shared
allocator and set `TMP`, `TEMP`, pytest `--basetemp`, logs, npm pack destination
and npm cache to owned external children. Diagnostics never belong in a source
checkout. `scripts/check_public_sdk.mjs --output /absolute/external/owned-child`
inspects the actual npm tarball and executes its extracted runtime without
generator dependencies. It does not publish anything. Bootstrap
`@tinihi-labs/artifact-paths@1.0.0-m1.0` as a separate dev artifact before this
gate, or point `CIRNET_ARTIFACT_PATHS_MODULE` at that installed package's
absolute module file URL. Outside a Git checkout, supply the shared allocator's
explicit `CIRNET_ARTIFACTS_CONTEXT` owner/rootset. The gate delegates output
validation to that implementation before creating files; it has no separate
path-validation algorithm and adds no allocator dependency to the core wheel.

`contracts/browser/generated` holds prebuilt DTOs, full/core offline Ajv
validators, operations and capture-layout constants. `contracts/browser/data`
holds the existing public browser catalog projections; `contracts/browser/types`
holds the retained lab compatibility facade. The explicit
`contracts/browser/artifacts.json` maps legacy Remote/Web snapshot paths to
installed package resources. `contracts.resources.generated_artifacts()` reads
those resources without importing or searching for a consumer implementation.
`contracts.codegen.generate_contracts.export_artifacts(output_root=...)` compares/exports
that immutable handoff into an explicit destination. Legacy `apps/web` paths are
export format keys only; generation never discovers or writes another checkout. Consumers must not regenerate it from retained source.

The npm tarball includes only built public resources under `dist`. Public schema
bundles use the registry's public-reference closure; public profile overrides
and named fixture cases exclude internal identities. Raw mixed-visibility schema
documents, RPC/proto files, canonical device inputs and compatibility baselines
remain available to Python and are absent from npm. The package's validators
enforce canonical schema/profile constraints. Existing Interface semantic
adapters (`parseLab`, document normalization, electrical and capture rules)
remain consumer behavior and must retain their existing conformance tests.

In the source repository, `device-library/**` is the only editable catalog
authority. Generation prepares its `contracts/catalog-source/**` package mirror
and all catalog projections together; check mode detects canonical edits even
when an old packaged mirror still exists. Installed wheels read the immutable
package mirror because they contain no source Device Library directory. Do not
edit that generated mirror as a second catalog authority.

FastAPI registrations in Remote remain the operation authority.
`contracts/api/openapi.json` is the reviewed frozen handoff input, never an
independently editable route model. Generate prerelease schema packages, export
OpenAPI from Remote on that candidate, review/freeze the document, build final
artifacts, then require Remote re-export on the exact final wheel to equal the
frozen bytes. For the M6 release, Python/npm distribution versions and the Remote API
application-info version align at 0.1.1. CircuitGraph/schema versions, Hardware
RPC, NCAP and other wire protocol versions remain unchanged.

`contracts/fingerprints.json` records separate canonical-source/public-schema/
OpenAPI fingerprints. Release artifact digests are recorded externally by the
integration handoff; Python and npm artifacts have different contents.

Pure contract assertions are retained under `tests/contracts`. Consumer and
Remote workflow assertions are preserved under `migration/consumer-tests`, with
the exact split in `migration/test-ownership.json`; that handoff directory is not
part of the standalone test suite. No assertion was converted into an automatic
pass to satisfy extraction.

The development allocator distribution is `cirnet-artifact-paths`, imported as
`cirnet_artifact_paths`. The old `netcircuit_artifact_paths` import and
`NETCIRCUIT_*` environment settings remain compatibility inputs. New settings
use `CIRNET_*`; when both are present the new value wins, including an empty
value. `contracts.runtime_config.getenv` reads aliases without mutation and
`export_environment` returns a copy with both names synchronized for child
processes. The allocator creates `cirnet-*` sessions with
`.cirnet-artifact-session.json` and verifies ownership before reading or
cleaning either current or valid legacy sessions.

Schema identities use `https://cirnet.local/`; the offline registry accepts the
old `https://netcircuit.local/` identities and preserves fragment resolution.
The browser SDK exports `schemaUriAliases` and `canonicalSchemaUri`. OpenAPI
includes canonical `x-cirnet-contract` metadata alongside the retained legacy
`x-netcircuit-contract` extension. Hardware RPC v2 uses `cirnet.hardware.v2`;
generated server registration accepts both service routes, and synchronous
stubs retry the old route only after `UNIMPLEMENTED`. Message field numbers,
serialized bytes and protocol versions are unchanged. The original DC supply
part descriptor stays in `historicalParts` for pinned legacy documents; the
current catalog calls it `CIRNET-DC-SUPPLY`.

## Namespace verification (2026-11-04)

The final CIRNET source candidate passes 516 Python tests, deterministic schema/catalog/topology generation, the CI bootstrap check and extracted public SDK checks. RPC compatibility preserves nonblocking futures, cancellation, callbacks, early metadata and the original total deadline; retry is limited to UNIMPLEMENTED on the canonical route. The runtime wheel SHA256 is `e129eb9075083dccbce9a37a595abe04f405197c2c06a86e651cb47b9a8acead`. Original published artifacts and frozen compatibility inputs retain their identities. Raw diagnostics are external; hosted Actions were not awaited.

## Node 26 LTS CI baseline (2026-11-05)

Both checks in `.github/tests/test_ci_bootstrap.py` pass with portable Node
26.13.0 on PATH. actionlint 1.7.12 passes the workflow with
`-shellcheck= -pyflakes=`. CI selects major 26 before allocation. SDK/DEV package minima remain
`>=22`, which already admits Node 26 and preserves consumer compatibility.
Contracts generation, the full Python/SDK gates and hosted Actions are NOT_RUN
for this toolchain policy change; schemas, generated payloads, package locks and
published artifacts are unchanged. Raw evidence was inspected outside Git in
the shared session owned by `cirnet-node26-compat-20261105`.
