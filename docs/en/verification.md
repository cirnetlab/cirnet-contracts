# Verify Contracts and the public SDK

English · [Tiếng Việt](../vi/verification.md)

[Documentation index](index.md)

Use a fresh Python environment outside the checkout with Node 26 and Git on PATH. Current CI covers Python 3.14 and 3.11. Use the reviewed hash-locked build and runtime dependency sets. Before producing any diagnostics, allocate a session. For a cross-repository check, set `CIRNET_ARTIFACTS_CONTEXT` to an explicit owner and every participating Git root before allocation.

Run this block in a dedicated POSIX shell at the repository root. The shell exits on failure. Keep it open after a successful run for the installed checks and cleanup.

~~~sh
set -e
CIRNET_ARTIFACTS_SESSION="$(node tools/artifact-paths/artifact-paths.mjs)"
test -n "$CIRNET_ARTIFACTS_SESSION"
export CIRNET_ARTIFACTS_SESSION
printf '%s\n' "$CIRNET_ARTIFACTS_SESSION"
export TMP="$CIRNET_ARTIFACTS_SESSION" TEMP="$CIRNET_ARTIFACTS_SESSION" TMPDIR="$CIRNET_ARTIFACTS_SESSION"
export PIP_CACHE_DIR="$CIRNET_ARTIFACTS_SESSION/pip-cache"
export npm_config_cache="$CIRNET_ARTIFACTS_SESSION/npm-cache"
export PYTHONDONTWRITEBYTECODE=1
python -B -m pip download --require-hashes --only-binary=:all: -r requirements-build.lock --dest "$CIRNET_ARTIFACTS_SESSION/build-wheelhouse"
python -B -m pip install --require-hashes --no-index --find-links "$CIRNET_ARTIFACTS_SESSION/build-wheelhouse" -r requirements-build.lock
python -B -m pip install --no-index --no-build-isolation --no-deps ./tools/artifact-paths
npm --prefix packages/public-sdk ci --no-audit --no-fund
python -B -m contracts.codegen.generate_contracts --check
python -B -m contracts.codegen.sync_editor_catalog --check
python -B -m contracts.codegen.sync_breadboard_topologies --check
python -B scripts/check_public_compatibility.py
export CIRNET_ARTIFACT_PATHS_MODULE="$(node --input-type=module -e 'import {resolve} from "node:path"; import {pathToFileURL} from "node:url"; console.log(pathToFileURL(resolve("tools/artifact-paths/cirnet_artifact_paths/artifact-paths.mjs")).href)')"
python -B -m pytest -q --basetemp "$CIRNET_ARTIFACTS_SESSION/pytest"
node scripts/check_public_sdk.mjs --output "$CIRNET_ARTIFACTS_SESSION/sdk-pack"
python -B -m build --no-isolation --wheel --sdist --outdir "$CIRNET_ARTIFACTS_SESSION/dist"
~~~

The SDK check builds and unpacks the npm tarball in the owned session and runs its public runtime without generator dependencies. It checks the public schema closure and rejects internal payloads. The wheel build invokes the hardware RPC generator and packages its v2 bindings.

For the installed check, use a new environment and an external working directory. Install the complete `requirements-runtime.lock` closure offline by hash, then install the exact wheel with `--no-index --no-deps`. Run `pip check` and `python -I -B ABSOLUTE_CHECKOUT/scripts/probe_installed_contracts.py`. Build the extracted sdist independently and compare wheel bytes with the same reproducible inputs. In Remote, re-export the OpenAPI document on the exact final wheel and compare it to the frozen handoff.

Record the tested revision, archive digests and gate scope. Inspect the raw results, retain selected release artifacts separately and clean only your owned session.

~~~sh
node tools/artifact-paths/artifact-paths.mjs --cleanup "$CIRNET_ARTIFACTS_SESSION"
~~~

The [standalone workflow](../../.github/workflows/contracts.yml) records the full build and installed gates. [Dated results](../../README.md#namespace-verification-2026-11-04) distinguish local runs from published artifact evidence.
