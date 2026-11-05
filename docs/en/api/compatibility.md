# Supported compatibility anchors

English · [Tiếng Việt](../../vi/api/compatibility.md)

[Documentation index](../index.md)

The [source baseline](../../../contracts/compatibility/2026-10-16/baseline.json) preserves pre-W1 schema and protobuf source bytes and their recorded Git provenance. The [public wire baseline](../../../contracts/compatibility/2026-10-16/public-wire-baseline.json) records post-W7 public operation and schema shapes and NCAP layout anchors. Both are supported unpublished baselines. They do not establish a released ABI or hardware operation. The public snapshot records the reviewed workspace without inventing a commit or tag for the phase.

After checking generated artifacts and allocating the session described in [verification](../verification.md), run from the repository root.

~~~sh
python -B scripts/check_public_compatibility.py
python -B -m pytest tests/contracts/test_public_compatibility.py --basetemp "$CIRNET_ARTIFACTS_SESSION/compatibility-pytest"
~~~

## Comparison scope

The checker compares each existing operation ID, method and path, parameters, request and response media and schemas, global and effective operation security requirements, effective JSON Schema dialect, named component definitions, schema URI/version/profile annotations and existing NCAP layout sections. Omitted root security and dialect in the original baseline resolve to the OpenAPI 3.1 defaults of no inherited security and the OAS 3.1 base dialect. Unsupported shapes or references fail explicitly. New operations and separately named or versioned contracts can coexist with supported anchors. The comparison covers frozen wire shapes. General JSON Schema subtyping and consumer semantics require separate evidence.

Schema and operation descriptions, titles and examples are editorial. The checker traverses actual schema positions. A property named `description`, or a default or example containing schema-like keys, remains data. Manifest ownership prose is outside the wire snapshot. Booleans and numbers remain distinct. NCAP magic, prefix, byte order, records, versions and resource limits are part of the supported format.

The check has no baseline update mode. A breaking change needs an explicit version and migration or a reviewed compatibility-policy decision. Keep the old version available alongside its replacement. Generation alone does not authorize changing a supported shape. CI also checks the original schema and proto byte baseline, shared Python and browser fixtures, negative mutations, generated operations and installed resources. Separate checks establish consumer behavior, electrical safety and physical capability.

## Name migration

Current schema hosts and the RPC v2 package use canonical CIRNET names. Explicit offline URI aliases and dual RPC routes retain legacy support. The source identity comparison reverses only the exact approved spelling substitutions before comparing original digests. OpenAPI retains legacy metadata and adds the canonical alias. Inconsistent alias metadata fails. The exact health service names `cirnet-api` and `netcircuit-api` are accepted together. Other values and shapes receive no widening exemption.

The security names `cirnet_session` and `X-CIRNET-CSRF` are exact aliases for `netcircuit_session` and `X-CSRF-Token`. Credential type, location, requirements and legacy runtime acceptance remain required. The canonical capture media type is accepted alongside an identical legacy media schema. Removing legacy media, changing either schema or introducing another credential name fails. Remote exercises dual cookie, header and capture Accept behavior.

The three current graph schema titles use `CIRNET`. Exact per-file aliases recover the original `net*CIRCUIT` titles for frozen-source digest comparison only. The preserved baseline bytes and digests remain the original evidence. See the [original compatibility reference](../../../contracts/compatibility/README.md).
