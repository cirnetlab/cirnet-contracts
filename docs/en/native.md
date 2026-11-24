# Build and verify Rust Contracts

English · [Tiếng Việt](../vi/native.md) · [Documentation index](index.md)

The `cirnet-contracts-native` 0.3.0-dev.1 crate provides an offline validator, NCAP1 and NCAP2 codecs, canonical Hardware RPC types and descriptor, and a generator for the TypeScript SDK. Its binary is `cirnet-contracts`. The manifest, schema profiles, device library, capture layout, RPC proto and frozen OpenAPI handoff are shared inputs. [Contract reference](contract-reference.md)

## Build and run

Development uses the pinned Rust and Cargo 1.99.1 toolchain with the committed lockfile. The crate declares Rust 1.90 as its minimum supported compiler version. [Toolchain pin](../../rust-toolchain.toml) · [Cargo manifest](../../Cargo.toml) · [Cargo lockfile](../../Cargo.lock)

On 64-bit Windows, use the `x86_64-pc-windows-msvc` host with Visual Studio C++ Build Tools and a Windows SDK. Run `rustc -Vv` and `cargo -Vv` from the checkout to verify the version and host.

If Rustup has only the `stable` alias installed, check `rustup run stable rustc -vV` and `rustup run stable cargo -vV`. Use `rustup run stable cargo` for the commands below while the alias resolves to 1.99.1 on the required host.

Set `CIRNET_ARTIFACTS_SESSION` to an allocator-owned external session before running these POSIX shell commands from the checkout root. Keep the same shell for the later commands. [Allocate a session](verification.md#allocate-a-session) and [clean it up after inspection](verification.md#inspect-and-clean-up).

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
~~~

`validate` takes a source root, contract name and profile. It reads JSON from standard input, writes the validated JSON to standard output and returns exit code 2 with a structured issue on standard error for contract validation failures. `descriptor` writes the canonical binary protobuf descriptor.

## Windows native package

From the Contracts checkout in PowerShell with the pinned MSVC toolchain, allocate a verification session and build the candidate bundle.

~~~powershell
Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
$env:CIRNET_ARTIFACTS_SESSION = node tools/artifact-paths/artifact-paths.mjs
$bundle = Join-Path $env:CIRNET_ARTIFACTS_SESSION 'contracts-native'
node scripts/package-native.mjs $bundle
$receipt = Get-Content -Raw (Join-Path $bundle "contracts-native-manifest.json") | ConvertFrom-Json
$target = Join-Path $env:CIRNET_ARTIFACTS_SESSION "contracts-native-target"
node scripts/check-native-package.mjs $bundle (Join-Path $target ("package/" + $receipt.crate.filename))
~~~

The build uses locked Cargo dependencies and the static MSVC CRT. The `python-tests` feature stays disabled. The bundle contains `bin/cirnet-contracts.exe`, canonical runtime schema and catalog resources, and `contracts-native-manifest.json` with the source SHA-256, available Git revision, worktree state, toolchain and SHA-256 inventory. Point `CIRNET_CONTRACTS_ROOT` to the bundle's `resources` directory. Run `validate` against that directory and use `descriptor` to read the embedded RPC schema. The bundle requires no compiler, Node or Python to execute.

If the verified pinned version is installed under Rustup's `stable` alias only, set `$env:RUSTUP_TOOLCHAIN = 'stable'` for this build shell. The packaging command checks the actual Rust and Cargo versions before it builds.

The [Windows native workflow](../../.github/workflows/native.yml) builds this candidate and the npm SDK. SDK generation uses the canonical source checkout and locked Node tools before deployment. A packaged smoke test on a development host is separate from the system's clean-install and safety gates.

The packaging receipt records the compiler provider, RPC descriptor SHA-256,
and the separate Rust crate archive identity. The native crate excludes Python
source. Its optional parity feature belongs to the test lane. The default
compiler comes from the locked vendored package. To use a reviewed alternative
compiler, supply an absolute `PROTOC` and its lowercase
`CIRNET_PROTOC_SHA256`. Check the resulting descriptor against the baseline
before using the alternate build for integration.

## Generate and verify the SDK

Use Node 26. The SDK test output directory must exist. Use a fresh, absent projection destination with an existing external parent for each `generate-sdk` run.

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
  --node-tools "$CIRNET_SDK_NODE_TOOLS" --output "$CIRNET_ARTIFACTS_SESSION/native-sdk-pack"
~~~

The generator reads the frozen `contracts/api/openapi.json` by default. An optional fourth argument after `generate-sdk` accepts an explicitly reviewed OpenAPI input. Supply the exact Remote native export for its SDK. Record the raw export and packed SDK SHA-256 values together and retain the generator and consumer checks. The generated `contracts/fingerprints.json` describes the normalized projection. Runtime Rust and this SDK path require no Python. The pack check runs the extracted SDK. To compare modules, declarations and resources, pass `--baseline` with a reviewed archive of the same SDK version. Compare archive SHA-256 separately for exact byte identity. [Handoff and package identities](api/compatibility.md#release-and-openapi-identities)

For a reviewed native handoff, pass `--native-bundle ABSOLUTE_BUNDLE`,
`--native-bundle-sha256 REVIEWED_MANIFEST_SHA256`, `--native-crate ABSOLUTE_CRATE`
and `--remote-openapi ABSOLUTE_EXPORT` to `scripts/check_public_sdk.mjs` with
the same `--node-tools` and `--output` arguments. The checker runs the verified
binary against the verified source crate and records both identities, the
raw OpenAPI SHA-256 and the npm archive digest in `native-sdk-receipt.json`.
The `native-v1` canonical source fingerprint uses `contracts/capture/layout.json`
for NCAP. The frozen manifest entry and the schema and OpenAPI projections
retain their existing identities.

## Parity fixtures

| Test | Inputs and assertions |
| --- | --- |
| [Validation](../../tests/validation_parity.rs) | [Shared contract cases](../../contracts/fixtures/conformance.json), [assignment](../../contracts/fixtures/assignment-v2-conformance.json), [instrument control](../../contracts/fixtures/instrument-control-conformance.json), [history](../../contracts/fixtures/history-conformance.json) and [design fixtures](../../contracts/design-models/fixtures/) cover profiles, compatibility, reference designs and malformed inputs. |
| [NCAP](../../tests/capture_parity.rs) | [Capture conformance](../../contracts/fixtures/capture/conformance.json) covers NCAP1/NCAP2 bytes, timing, gaps, malformed inputs and INT16, UINT16 and FLOAT32 encodings. |
| [Hardware RPC v2](../../tests/hardware_v2_parity.rs) | The [canonical proto](../../contracts/hardware/v2/rpc_v2.proto) is the descriptor input. Tests cover both service names, wire tags and absent versus explicitly defaulted fields. |
| [SDK generation](../../tests/generator_parity.rs) | The ordinary Cargo run includes the local projection check. Run the five ignored cases separately with the locked Node tools and external scratch as shown above. They cover regeneration without committed browser snapshots, rejection of unsafe or stale inputs, and packaging. |

## Verification evidence

Record the source SHA-256, tools, test outcomes and artifact digests in the external session. The integrated candidate also records the exact Remote export SHA-256 alongside the SDK archive. Release selection remains with Remote.
