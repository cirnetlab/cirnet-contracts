# CIRNET artifact allocator development tooling

This standalone package owns the shared artifact containment and session policy. It is a development dependency, independent of Contracts schemas and runtime packages. Node 22 or later and Git must be in PATH. Python 3.11 or later callers use the packaged Node implementation; Python does not duplicate path validation.

```js
import { getArtifactDirectory, cleanupArtifactSession } from '@cirnet/artifact-paths'
const context = { owner: 'my-repo-verification', roots: ['/absolute/current-checkout'] }
const session = getArtifactDirectory('', { context })
const screenshots = getArtifactDirectory('screenshots')
// Inspect evidence before cleanup.
cleanupArtifactSession(session)
```

```python
from cirnet_artifact_paths import artifact_directory, cleanup_artifact_session
context = {'owner': 'my-repo-verification', 'roots': ['/absolute/current-checkout']}
session = artifact_directory(context=context)
screenshots = artifact_directory('screenshots')
cleanup_artifact_session(session)
```

Set `CIRNET_LOCAL_STORAGE_ROOT` to an existing absolute local directory outside Git.
The configured default is `<root>/.cirnet-build-output/<canonical-repository>`.
The owner must match a canonical CIRNETLAB Git origin in the context. Linked
worktrees use their own Git context. Remote owns the native-candidate context.
An explicit external parent or `CIRNET_ARTIFACTS_DIR` overrides the default.
When centralized storage is configured, these settings must agree with any
inherited session. Start an independent process without inherited artifact
settings to allocate for another task.

Invocation context is explicit through API `context`, CLI `--context JSON`, or inherited `CIRNET_ARTIFACTS_CONTEXT`. It binds a stable owner and a canonical rootset to the session. Integration callers list every involved checkout; standalone callers require only their own checkout. Every root expands to its Git administration paths and entire linked-worktree family. Enumeration failures reject allocation. A context that omits the current Git checkout is rejected. The nearest existing destination ancestor is also inspected independently: output inside an unlisted checkout, Git administration directory or bare Git repository is rejected. Only a known non-Git destination is allowed; other discovery failures fail closed. Git location override environment variables cannot substitute another repository during these checks.

Without explicit or inherited context, the current working directory determines the Git checkout and owner `git:<canonical-root>`. Outside a checkout, context is required. Installed module location never selects repository ownership. Repository wrappers that support arbitrary CWD should pass their repository root explicitly.

The allocator sets `CIRNET_ARTIFACTS_SESSION` and `CIRNET_ARTIFACTS_CONTEXT` for child processes on implicit-parent allocation. Passing `directory` allocates independently without reassigning the caller's inherited session/context. `CIRNET_ARTIFACTS_DIR` selects an absolute external parent. Session v2 stores canonical context; inheritance and cleanup require exactly matching owner/rootset. A legacy v1 marker is accepted only for its original sole repository and default Git owner.

```text
node artifact-paths.mjs [subdirectory] [--directory EXTERNAL_PARENT] [--context JSON]
node artifact-paths.mjs --validate EXTERNAL_PATH [--context JSON]
node artifact-paths.mjs --cleanup SESSION [--context JSON]
node artifact-paths.mjs --context-info [--context JSON]
node artifact-paths.mjs --parent-info [--context JSON]
node artifact-paths.mjs --session-info [--context JSON]
```

Session aliases, output traversal, repository symlink destinations and unowned cleanup are rejected. Cleanup removes only a verified canonical `cirnet-*` or valid legacy `netcircuit-*` session. It does not follow internal directory symlinks. As with the original allocator, protection is a point-in-time filesystem check, not an operating-system security boundary against hostile concurrent path replacement.

Run tests with explicit `ALLOCATOR_TEST_SOURCE` and external `ALLOCATOR_TEST_SCRATCH`, then `npm test` and `python -B -m unittest discover -s test -p test_python_bridge.py`. Tests create no source/target commits or branches. These prerelease versions are local candidates, not published artifacts.

The Python distribution is `cirnet-artifact-paths`. The legacy import
`netcircuit_artifact_paths` and CLI name `netcircuit-artifact-paths` remain aliases.
`NETCIRCUIT_*` settings remain readable when the corresponding `CIRNET_*` value
is absent. Canonical values win; implicit allocation exports both prefixes to
child processes. New sessions use `.cirnet-artifact-session.json`; old v1/v2
markers remain readable under the same ownership and containment rules.
Conflicting dual markers or a marker symlink fail closed.

`--session-info` validates an inherited session and emits its canonical path, context,
and marker SHA-256 without making directories. `getExistingArtifactDirectory`
selects an existing child without allocation. For a new task-bound session set
`CIRNET_ARTIFACTS_TASK_KEY` to the original Task Continuity key before allocation
and when resuming. A mismatched or missing key cannot reuse that session.

With centralized storage configured, a new ambient `CIRNET_ARTIFACTS_DIR`
must stay in the repository namespace. Select an authorized alternate parent with
the explicit `--directory` parameter. An inherited, validated session can
keep its explicitly selected parent. Cleanup still requires separate review of
current references, active users, and retained dependencies.
