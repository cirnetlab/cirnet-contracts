# Verify CIRNET Contracts

[Tiếng Việt](../vi/verification.md) · [Documentation](index.md)

## Allocate a session

Set `CIRNET_ARTIFACTS_DIR` to an absolute directory outside all participating checkouts, worktrees and Git directories. The Node allocator creates an owned session there. Set `CIRNET_ARTIFACTS_CONTEXT` with the session owner and all participating roots for multi-repository verification. See the [allocator guide](../../tools/artifact-paths/README.md).

## Run checks

Run `cargo fmt --all --check`, `cargo test --locked` and the SDK regression suite from the [native build guide](native.md). The Windows workflow verifies the native package, the crate, the RPC descriptor and the generated npm package. From the checkout root, it packs the development allocator with `npm pack ./tools/artifact-paths --ignore-scripts --pack-destination <sdk-output>`. The system candidate additionally builds against the exact Remote export and records the Contracts source, manifest and crate digests.

## Compare the installed legacy implementation

From the Next checkout, use Windows PowerShell 7, CPython 3.14.4 x64 and the pinned Rust toolchain. Set `CIRNET_ARTIFACTS_CONTEXT` to include both Contracts checkouts before allocating and exporting a fresh `CIRNET_ARTIFACTS_SESSION`. Run:

~~~powershell
./scripts/test-legacy-parity.ps1 -LegacyCheckout ../cirnet-contracts
~~~

The runner verifies the Git tree at Contracts 0.1.2, archives it into the session and builds a reference wheel there. Supply `-SealedWheel <path>` in place of `-LegacyCheckout` to verify against the original published wheel. The runner checks its pinned SHA-256. An optional `-Wheelhouse <path>` uses a local wheelhouse for the locked dependencies.

The restored parity cases compare profiles, numeric boundaries, NCAP bytes and timing, and the Hardware RPC descriptor with the installed wheel. Inspect `legacy-parity/inputs.json` for reference and binding digests. The rebuilt wheel is a source reference. The production build keeps the `python-tests` feature disabled.

The Windows workflow also runs the Node allocator and Python bridge tests. Set `ALLOCATOR_TEST_SOURCE` to the Next checkout, `ALLOCATOR_TEST_SCRATCH` to a directory in the owned session and `PYTHONPATH` to the absolute `tools/artifact-paths` directory to run these checks locally:

~~~powershell
node --test tools/artifact-paths/test/artifact-paths.test.mjs
python -B -m unittest discover -s tools/artifact-paths/test -p test_python_bridge.py
~~~

## Inspect and clean up

Keep raw output in the external session. Inspect the manifest and test logs and record approved summaries with the exact artifact identity. When finished, remove the owned session using `node tools/artifact-paths/artifact-paths.mjs --cleanup <session>`.
