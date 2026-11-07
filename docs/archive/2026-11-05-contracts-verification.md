# Contracts verification archive

English · [English history index](../en/history.md) · [Mục lục lịch sử tiếng Việt](../vi/history.md)

These dated results are reproduced verbatim from the [Contracts README at commit e640f80b1199779659d3db7817fee16d13cc24c1](https://github.com/cirnetlab/cirnet-contracts/blob/e640f80b1199779659d3db7817fee16d13cc24c1/README.md).

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

## Python 3.14 verification (2026-11-05)

Primary CI uses Python 3.14 and retains the 3.11 compatibility entry. A fresh
normal-GIL CPython 3.14.6 Windows AMD64 environment passes deterministic
generation/catalog/topology/compatibility, all 516 source tests and 209 isolated
installed-wheel conformance tests. The exact source candidate also rebuilds
from its independently extracted sdist to identical wheel bytes. All original
RECORD payloads and installation archive hashes were verified.

Sixteen official, downloaded-and-verified CPython 3.14 Windows/Linux wheel hashes
extend the two existing locks without changing dependency versions or removing
older hashes. All five producer lock closures pass offline binary-only hash
verification for Windows AMD64 and glibc Linux x86_64. Consumer builds of the
older pinned Contracts source use hash-only profiles bound to its original
lock digests, recorded in their CI build manifests. Package/runtime/source
release pins remain separate from these local verification candidates.

The API audit found no use requiring replacement of a removed stdlib API or
annotation behavior. Raw evidence is external under the session owned by
`cirnet-python314-compat-20261105`; historical results remain unchanged.
Hosted Actions and publication are NOT_RUN for this upgrade.

Independent normal-GIL Linux 3.14.6 verification also passes all 516 tests,
three CI bootstrap checks and all four deterministic commands. The actual
consumer helper builds the original pinned source's Python wheel and SDK/npm
package using the original lock digests plus the reviewed hash-only profile.
No published source/artifact digest is replaced.
