# Getting started with CIRNET Contracts

English · [Tiếng Việt](../vi/getting-started.md)

[Documentation index](index.md)

The current source builds Python `cirnet-contracts==0.1.2` and browser `@cirnet/contracts@0.1.2`. The Python import is `contracts` and requires Python 3.11 or later. Use artifacts and SHA256 digests from the selected system release. Use a separate environment for older `netcircuit-contracts` wheels because both distributions own the same import namespace.

After installing the selected Python wheel, check its distribution version from outside the checkout.

~~~sh
python -I -B -c "from importlib.metadata import version; print(version('cirnet-contracts'))"
~~~

The browser SDK provides public DTOs, offline validators, operation and capture-layout metadata, and public catalog projections. Its package includes the public schema reference closure and named fixtures. RPC protobuf, internal catalog sources and compatibility baselines are Python-side resources. See the [SDK package README](../../packages/public-sdk/README.md).

The independent DEV allocator is `cirnet-artifact-paths` for Python and `@cirnet/artifact-paths` for Node. It owns temporary output placement and guarded session cleanup. It is installed separately from Contracts runtime. New settings use `CIRNET_*` and take precedence over legacy `NETCIRCUIT_*`, including when the new value is empty. See the [allocator package README](../../tools/artifact-paths/README.md).

Contracts owns schema shapes, offline registries and generated bindings. Remote owns FastAPI operation registration and system release selection. The frozen OpenAPI handoff must match a Remote export from the exact final wheel. See the [contract reference map](contract-reference.md) and [verification guide](verification.md).
