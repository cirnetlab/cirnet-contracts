# Verify CIRNET Contracts

[Tiếng Việt](../vi/verification.md) · [Documentation](index.md)

## Allocate a session

Set `CIRNET_ARTIFACTS_DIR` to an absolute directory outside all participating checkouts, worktrees and Git directories. The Node allocator creates an owned session there. Set `CIRNET_ARTIFACTS_CONTEXT` with the session owner and all participating roots for multi-repository verification. See the [allocator guide](../../tools/artifact-paths/README.md).

## Run checks

Run `cargo fmt --all --check`, `cargo test --locked` and the SDK regression suite from the [native build guide](native.md). The Windows workflow verifies the native package, the crate, the RPC descriptor and the generated npm package. From the checkout root, it packs the development allocator with `npm pack ./tools/artifact-paths --ignore-scripts --pack-destination <sdk-output>`. The system candidate additionally builds against the exact Remote export and records the Contracts source, manifest and crate digests.

## Verify compatibility against frozen reference outcomes

From the Contracts checkout, use Windows PowerShell 7, CPython 3.14.4 x64 and the pinned Rust toolchain. Allocate and export a fresh `CIRNET_ARTIFACTS_SESSION`, then run the same parity check as the Windows workflow.

~~~powershell
./scripts/test-native-parity.ps1
~~~

For an offline run, supply a local `-Wheelhouse <path>` with the locked third-party wheels. Point `CARGO_HOME` to a cache populated with the locked Rust crates and set `CARGO_NET_OFFLINE=true` in the build environment.

The 146 cases cover validation profiles, numeric boundaries, NCAP decoding and errors, canonical NCAP re-encoding and frame timing, and the Hardware RPC descriptor. The [verification fixtures](../../tests/fixtures/README.md) contain the expected outcomes from Contracts 0.1.2 and their provenance. Inspect `native-parity/inputs.json` in the session for fixture and binding digests. The runner alone enables `python-tests` for this comparison.

## Check the development allocator

The Windows workflow also runs the Node allocator and Python bridge tests. Set `ALLOCATOR_TEST_SOURCE` to the Contracts checkout, `ALLOCATOR_TEST_SCRATCH` to a directory in the owned session and `PYTHONPATH` to the absolute `tools/artifact-paths` directory to run these checks locally:

~~~powershell
node --test tools/artifact-paths/test/artifact-paths.test.mjs
python -B -m unittest discover -s tools/artifact-paths/test -p test_python_bridge.py
~~~

## Inspect and clean up

Keep raw output in the external session. Inspect the manifest and test logs and record approved summaries with the exact artifact identity. When finished, remove the owned session using `node tools/artifact-paths/artifact-paths.mjs --cleanup <session>`.

## Local development storage

Set `CIRNET_LOCAL_STORAGE_ROOT` to an existing external local directory to allocate by canonical GitHub identity. The parent is selected from the explicit directory, `CIRNET_ARTIFACTS_DIR`, the configured repository namespace, then OS temporary storage. The configured root is validated even with an explicit override. A configured parent and inherited session must agree. Linked worktrees share a namespace and have separate session ownership.
