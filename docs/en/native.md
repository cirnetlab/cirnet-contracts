# Build and verify Rust Contracts

English · [Tiếng Việt](../vi/native.md) · [Documentation index](index.md)

The `cirnet-contracts-native` 0.1.2 crate provides an offline validator, NCAP1 and NCAP2 codecs, Hardware RPC types and descriptors, and a generator for the TypeScript SDK. Its binary is `cirnet-contracts`. The manifest, schema profiles, device library, capture layout, RPC proto and frozen OpenAPI handoff are shared inputs. [Contract reference](contract-reference.md)

## Build and run

Use [Rust 1.90.0](../../rust-toolchain.toml) with [Cargo.lock](../../Cargo.lock). Set `CIRNET_ARTIFACTS_SESSION` to an allocator-owned external session before running these POSIX shell commands from the checkout root. Keep the same shell for the later commands. [Allocate a session](verification.md#allocate-a-session) and [clean it up after inspection](verification.md#inspect-and-clean-up).

~~~sh
repo="$(pwd -P)"
test -n "$CIRNET_ARTIFACTS_SESSION"
export CARGO_TARGET_DIR="$CIRNET_ARTIFACTS_SESSION/cargo-target"
export TMP="$CIRNET_ARTIFACTS_SESSION" TEMP="$CIRNET_ARTIFACTS_SESSION" TMPDIR="$CIRNET_ARTIFACTS_SESSION"

cargo build --locked --release
cargo test --locked

printf '%s\n' '{"status":"ok","service":"cirnet-api","hardware_mode":"simulation"}' |
  cargo run --locked --release -- validate "$repo" HealthResponse transport
cargo run --locked --release -- descriptor > "$CIRNET_ARTIFACTS_SESSION/rpc-v2.pb"
cargo run --locked --release -- descriptor legacy > "$CIRNET_ARTIFACTS_SESSION/rpc-v2-legacy.pb"
~~~

`validate` takes a source root, contract name and profile. It reads JSON from standard input, writes the validated JSON to standard output and returns exit code 2 with a structured issue on standard error for contract validation failures. The descriptor commands write binary protobuf descriptors.

## Generate and verify the SDK

Use Node 26 and set `selected_sdk` to the absolute path of the selected npm tarball after checking its SHA-256 against the system manifest. The SDK test output directory must exist. Use a fresh, absent projection destination with an existing external parent for each `generate-sdk` run.

~~~sh
export CIRNET_SDK_NODE_TOOLS="$CIRNET_ARTIFACTS_SESSION/sdk-node-tools"
export CIRNET_SDK_TEST_OUTPUT="$CIRNET_ARTIFACTS_SESSION/sdk-test-output"
mkdir -p "$CIRNET_SDK_NODE_TOOLS" "$CIRNET_SDK_TEST_OUTPUT"
cp packages/public-sdk/package.json packages/public-sdk/package-lock.json "$CIRNET_SDK_NODE_TOOLS/"
npm --prefix "$CIRNET_SDK_NODE_TOOLS" ci --no-audit --no-fund --cache "$CIRNET_ARTIFACTS_SESSION/npm-cache"

cargo test --locked --test generator_parity -- --ignored

sdk_projection="$CIRNET_ARTIFACTS_SESSION/native-sdk"
cargo run --locked --release -- generate-sdk "$repo" "$sdk_projection" "$CIRNET_SDK_NODE_TOOLS"
node packages/public-sdk/scripts/build.mjs --source "$sdk_projection/contracts/browser" \
  --node-tools "$CIRNET_SDK_NODE_TOOLS" --output "$CIRNET_ARTIFACTS_SESSION/native-sdk-dist"
export CIRNET_ARTIFACT_PATHS_MODULE="$(node --input-type=module -e 'import {resolve} from "node:path"; import {pathToFileURL} from "node:url"; console.log(pathToFileURL(resolve("tools/artifact-paths/cirnet_artifact_paths/artifact-paths.mjs")).href)')"
node scripts/check_public_sdk.mjs --source "$sdk_projection/contracts/browser" \
  --node-tools "$CIRNET_SDK_NODE_TOOLS" --output "$CIRNET_ARTIFACTS_SESSION/native-sdk-pack" \
  --baseline "$selected_sdk"
~~~

The generator reads `contracts/api/openapi.json` by default. An optional fourth argument after `generate-sdk` accepts an explicitly reviewed OpenAPI input. Runtime Rust and this SDK path require no Python. The pack check runs the extracted SDK and compares modules, declarations and resources with the selected tarball. Compare the archive SHA-256 separately when checking exact byte identity.

## Parity fixtures

| Test | Inputs and assertions |
| --- | --- |
| [Validation](../../native/tests/validation_parity.rs) | [Shared contract cases](../../contracts/fixtures/conformance.json), [assignment](../../contracts/fixtures/assignment-v2-conformance.json), [instrument control](../../contracts/fixtures/instrument-control-conformance.json), [history](../../contracts/fixtures/history-conformance.json) and [design fixtures](../../contracts/design-models/fixtures/) cover profiles, compatibility, reference designs and malformed inputs. |
| [NCAP](../../native/tests/capture_parity.rs) | [Capture conformance](../../contracts/fixtures/capture/conformance.json) covers NCAP1/NCAP2 bytes, timing, gaps, malformed inputs and INT16, UINT16 and FLOAT32 encodings. |
| [Hardware RPC v2](../../native/tests/hardware_v2_parity.rs) | The [canonical proto](../../contracts/hardware/v2/rpc_v2.proto) is the descriptor input. Tests cover both service names, wire tags and absent versus explicitly defaulted fields. |
| [SDK generation](../../native/tests/generator_parity.rs) | The ordinary Cargo run includes the local projection check. Run the five ignored cases separately with the locked Node tools and external scratch as shown above. They cover regeneration without committed browser snapshots, rejection of unsafe or stale inputs, and packaging. |

## Optional installed-wheel comparison

Prepare a dedicated external Python environment with pytest, the locked runtime dependencies including the RPC closure, and the selected 0.1.2 wheel installed directly from the hash-verified archive. Set `venv_python` to that environment's absolute Python executable. The comparison test verifies the installed distribution's recorded wheel hash and imports the Cargo-built extension from the supplied absolute path. The binding is a test harness.

~~~sh
PYO3_PYTHON="$venv_python" cargo build --locked --release --features python-tests
export CIRNET_CONTRACTS_PYTHON_SHA256=7b092af5817b0e15f02f9f5f292a8e1b20fe1d512600f9c0260d05a58ddaaf42
export CIRNET_CONTRACTS_NATIVE_BINDING="$CARGO_TARGET_DIR/release/libcirnet_contracts_native.so"
(
  cd "$CIRNET_ARTIFACTS_SESSION"
  "$venv_python" -I -B -m pytest -q -p no:cacheprovider \
    --basetemp "$CIRNET_ARTIFACTS_SESSION/native-pytest" \
    "$repo/native/tests/test_installed_native_parity.py"
)
~~~

The example names the Linux extension. On Windows MSVC, copy `cirnet_contracts_native.dll` from the release output to an external `cirnet_contracts_native.pyd` and set `CIRNET_CONTRACTS_NATIVE_BINDING` to that copy before running pytest. Use the same Python environment for the PyO3 build and the comparison.

## Verification evidence

- **Current source.** Rust implementation commit `e640f80b1199779659d3db7817fee16d13cc24c1` builds on `f559ec5b21ea7ecf50db4dc5c5e1dc7c5eb5ad39`. Use the exact reviewed checkout revision when recording new results.
- **Verified candidate.** Gate 2 passed on R2 for the recorded source snapshot `d9d6de46c7cc4b46cf2fe0926ce962a5f2775a2bb8965a0e74e26f2a8447aca8`, with 42 Windows Rust tests, 146 installed-wheel cases, 35 Linux native runtime tests and the sealed SDK/Interface comparison. The first candidate's compound-invalid diagnostic failure stays in the record. Other inputs with multiple simultaneous schema violations can still produce different first issues. [Stage 2 candidate and R2 evidence](https://github.com/cirnetlab/cirnet-remote/blob/13a35145b6d15644c7be10d95a5410bf69e11867/docs/en/architecture/native-migration.md#stage-2-candidate-verification)
- **Selected release.** Remote's system 0.1.4 manifest selects the Contracts v0.1.2 Python wheel and npm SDK from `524cc7a5e2e65f39077feefe70c7fb2ae3a773d6`. The R2 SDK matched the selected npm archive. The R2 executable and crate are candidate artifacts. [System release manifest](https://github.com/cirnetlab/cirnet-remote/blob/master/integration/release-set.json)
- **CI and this documentation revision.** Standalone CI runs the legacy Python and SDK lanes on Linux with Python 3.11/3.14 and Node 26. Cargo, the external Rust SDK tests and the PyO3 comparison are outside that workflow. Rust builds, parity tests, hosted CI and release publication are **NOT RUN** for this documentation revision. [Standalone workflow](../../.github/workflows/contracts.yml)
