# Verify CIRNET Contracts

[Tiếng Việt](../vi/verification.md) · [Documentation](index.md)

## Allocate a session

Set `CIRNET_ARTIFACTS_DIR` to an absolute directory outside all participating checkouts, worktrees and Git directories. The Node allocator creates an owned session there. Set `CIRNET_ARTIFACTS_CONTEXT` with the session owner and all participating roots for multi-repository verification. See the [allocator guide](../../tools/artifact-paths/README.md).

## Run checks

Run `cargo fmt --all --check`, `cargo test --locked` and the SDK regression suite from the [native build guide](native.md). The Windows workflow verifies the native package, the crate, the RPC descriptor and the generated npm package. From the checkout root, it packs the development allocator with `npm pack ./tools/artifact-paths --ignore-scripts --pack-destination <sdk-output>`. The system candidate additionally builds against the exact Remote export and records the Contracts source, manifest and crate digests.

## Inspect and clean up

Keep raw output in the external session. Inspect the manifest and test logs and record approved summaries with the exact artifact identity. When finished, remove the owned session using `node tools/artifact-paths/artifact-paths.mjs --cleanup <session>`.
