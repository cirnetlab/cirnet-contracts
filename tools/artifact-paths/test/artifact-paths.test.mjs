import { test } from 'node:test'
import assert from 'node:assert/strict'
import { spawnSync, execFileSync } from 'node:child_process'
import { mkdirSync, mkdtempSync, writeFileSync, readFileSync, existsSync, symlinkSync, realpathSync, cpSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'

const packageRoot = dirname(dirname(fileURLToPath(import.meta.url)))
const cli = join(packageRoot, 'artifact-paths.mjs')
const source = process.env.ALLOCATOR_TEST_SOURCE
const scratch = process.env.ALLOCATOR_TEST_SCRATCH
assert.ok(source && scratch, 'Tests require explicit external scratch and source fixture roots')
mkdirSync(scratch, { recursive: true })
const root = realpathSync(mkdtempSync(join(scratch, 'fixtures-')))
const repoA = join(root, 'repo-a')
const repoB = join(root, 'repo-b')
for (const repo of [repoA, repoB]) {
  mkdirSync(repo)
  execFileSync('git', ['init', '-q', repo], { windowsHide: true })
}
const unlisted = join(root, 'unlisted-repository')
mkdirSync(unlisted)
execFileSync('git', ['init', '-q', unlisted], { windowsHide: true })
const bare = join(root, 'unlisted-bare.git')
execFileSync('git', ['init', '-q', '--bare', bare], { windowsHide: true })
const outside = join(root, 'outside')
mkdirSync(outside)
const context = { owner: 'allocator-integration', roots: [repoA, repoB, source] }
function environment(extra = {}) {
  const env = { ...process.env }
  for (const key of ['CIRNET_ARTIFACTS_DIR', 'CIRNET_ARTIFACTS_SESSION', 'CIRNET_ARTIFACTS_CONTEXT', 'NETCIRCUIT_ARTIFACTS_DIR', 'NETCIRCUIT_ARTIFACTS_SESSION', 'NETCIRCUIT_ARTIFACTS_CONTEXT']) delete env[key]
  return { ...env, ...extra }
}
function invoke(args = [], { cwd = outside, env = environment(), executable = cli } = {}) {
  return spawnSync(process.execPath, [executable, ...args], { cwd, env, encoding: 'utf8', windowsHide: true })
}
const ctxArgs = (value = context) => ['--context', JSON.stringify(value)]
function allocate(value = context, executable = cli) {
  const result = invoke(['--directory', outside, ...ctxArgs(value)], { executable })
  assert.equal(result.status, 0, result.stderr)
  return result.stdout.trim()
}
function cleanup(session, value = context, executable = cli) {
  const result = invoke(['--cleanup', session, ...ctxArgs(value)], { executable })
  assert.equal(result.status, 0, result.stderr)
}

test('installed module allocates from arbitrary CWD with explicit involved roots', () => {
  const installed = join(root, 'installed', 'node_modules', '@tinihi-labs', 'artifact-paths')
  mkdirSync(installed, { recursive: true })
  cpSync(cli, join(installed, 'artifact-paths.mjs'))
  mkdirSync(join(installed, 'cirnet_artifact_paths'))
  cpSync(join(packageRoot, 'cirnet_artifact_paths/artifact-paths.mjs'), join(installed, 'cirnet_artifact_paths/artifact-paths.mjs'))
  const installedCli = join(installed, 'artifact-paths.mjs')
  const session = allocate(context, installedCli)
  try {
    assert.equal(dirname(session), realpathSync(outside))
    assert.ok(existsSync(join(session, '.cirnet-artifact-session.json')))
  } finally { cleanup(session, context, cli) }
})

test('protects all involved Git families, linked worktrees and Git administration paths', () => {
  const worktreeLines = execFileSync('git', ['-C', source, 'worktree', 'list', '--porcelain', '-z'], { encoding: 'utf8' }).split('\0')
  const sourceWorktrees = worktreeLines.filter(line => line.startsWith('worktree ')).map(line => line.slice(9))
  const gitDir = execFileSync('git', ['-C', source, 'rev-parse', '--absolute-git-dir'], { encoding: 'utf8' }).trim()
  for (const forbidden of [repoA, repoB, gitDir, ...sourceWorktrees]) {
    const result = invoke(['--validate', join(forbidden, 'must-not-create'), ...ctxArgs()])
    assert.notEqual(result.status, 0, forbidden)
    assert.match(result.stderr, /outside/)
    assert.equal(existsSync(join(forbidden, 'must-not-create')), false)
  }
})

test('default Git CWD preserves repository-root behavior without package-location assumptions', () => {
  const result = invoke(['--directory', outside], { cwd: repoA })
  assert.equal(result.status, 0, result.stderr)
  const session = result.stdout.trim()
  try {
    const inherited = environment({ CIRNET_ARTIFACTS_SESSION: session })
    const child = invoke(['screens'], { cwd: repoA, env: inherited })
    assert.equal(child.status, 0, child.stderr)
    assert.equal(child.stdout.trim(), join(session, 'screens'))
  } finally {
    const removed = invoke(['--cleanup', session], { cwd: repoA })
    assert.equal(removed.status, 0, removed.stderr)
  }
})

test('stable inherited owner/context lets child and relocated package share and clean a session', () => {
  const session = allocate()
  const marker = JSON.parse(readFileSync(join(session, '.cirnet-artifact-session.json'), 'utf8'))
  const env = environment({ CIRNET_ARTIFACTS_SESSION: session, CIRNET_ARTIFACTS_CONTEXT: JSON.stringify(marker.context) })
  const child = invoke(['python-child'], { env })
  assert.equal(child.status, 0, child.stderr)
  assert.equal(child.stdout.trim(), join(session, 'python-child'))
  cleanup(session)
  assert.equal(existsSync(session), false)
})

test('rejects owner/rootset mismatch, omitted current repository and malformed contexts', () => {
  const session = allocate()
  try {
    for (const wrong of [
      { owner: 'someone-else', roots: context.roots },
      { owner: context.owner, roots: [repoA, source] },
      { owner: '', roots: context.roots },
      { owner: context.owner, roots: [] },
      { owner: context.owner, roots: ['relative'] },
    ]) {
      assert.notEqual(invoke(['--cleanup', session, ...ctxArgs(wrong)]).status, 0)
      assert.ok(existsSync(session))
    }
    const omitted = invoke(['--directory', outside, ...ctxArgs({ owner: 'omit', roots: [repoA] })], { cwd: repoB })
    assert.notEqual(omitted.status, 0)
    assert.match(omitted.stderr, /omits|current/)
  } finally { cleanup(session) }
})

test('symlink and traversal rejection preserves neighboring user data', () => {
  const session = allocate()
  const userData = join(outside, 'user-data.txt')
  writeFileSync(userData, 'keep')
  try {
    const alias = join(outside, 'alias-' + Date.now())
    symlinkSync(session, alias, process.platform === 'win32' ? 'junction' : 'dir')
    assert.notEqual(invoke(['--cleanup', alias, ...ctxArgs()]).status, 0)
    const checkoutAlias = join(session, 'checkout')
    symlinkSync(repoB, checkoutAlias, process.platform === 'win32' ? 'junction' : 'dir')
    const env = environment({ CIRNET_ARTIFACTS_SESSION: session, CIRNET_ARTIFACTS_CONTEXT: JSON.stringify(context) })
    assert.notEqual(invoke(['checkout/forbidden'], { env }).status, 0)
    assert.notEqual(invoke(['../escape'], { env }).status, 0)
    assert.notEqual(invoke(['--cleanup', outside, ...ctxArgs()]).status, 0)
  } finally { cleanup(session) }
  assert.equal(readFileSync(userData, 'utf8'), 'keep')
})

test('Git enumeration failures fail closed before creating an artifact path', () => {
  const badRoot = join(root, 'broken-git')
  mkdirSync(badRoot)
  writeFileSync(join(badRoot, '.git'), 'gitdir: unavailable-admin\n')
  const parent = join(outside, 'not-created')
  const result = invoke(['--directory', parent, ...ctxArgs({ owner: 'broken', roots: [badRoot] })])
  assert.notEqual(result.status, 0)
  assert.equal(existsSync(parent), false)
})

test('denied worktree enumeration cannot silently reduce a valid protected rootset', () => {
  // Permission ACL mutation is not portable; inject only this OS boundary failure.
  // Root discovery and every other Git operation still execute the actual tool.
  const parent = join(outside, 'denied-must-not-create')
  const program = `
    import childProcess from 'node:child_process';
    import { syncBuiltinESMExports } from 'node:module';
    const original = childProcess.execFileSync;
    childProcess.execFileSync = (command, args, options) => {
      if (command === 'git' && args.includes('worktree')) throw Object.assign(new Error('permission denied enumerating Git worktrees'), { code: 'EACCES' });
      return original(command, args, options);
    };
    syncBuiltinESMExports();
    const api = await import(${JSON.stringify(pathToFileURL(cli).href)});
    api.getArtifactDirectory('', { directory: ${JSON.stringify(parent)}, context: ${JSON.stringify(context)} });
  `
  const result = spawnSync(process.execPath, ['--input-type=module', '-e', program], { cwd: outside, env: environment(), encoding: 'utf8', windowsHide: true })
  assert.notEqual(result.status, 0)
  assert.match(result.stderr, /permission denied/)
  assert.equal(existsSync(parent), false)
})

test('Git location environment overrides cannot substitute a different protected checkout', () => {
  const result = invoke(['--validate', join(repoA, 'forbidden'), ...ctxArgs({ owner: 'physical-root', roots: [repoA] })], {
    env: environment({ GIT_DIR: join(repoB, '.git'), GIT_WORK_TREE: repoB }),
  })
  assert.notEqual(result.status, 0)
  assert.match(result.stderr, /outside/)
  assert.equal(existsSync(join(repoA, 'forbidden')), false)
})

test('destination discovery rejects an unlisted independent checkout and its Git directory', () => {
  const selected = { owner: 'source-only', roots: [source] }
  for (const target of [join(unlisted, 'not-created'), join(unlisted, '.git', 'not-created')]) {
    const result = invoke(['--validate', target, ...ctxArgs(selected)], { cwd: source })
    assert.notEqual(result.status, 0, target)
    assert.match(result.stderr, /outside|Git/)
    assert.equal(existsSync(target), false)
  }
})

test('destination discovery rejects an unlisted bare Git directory', () => {
  const target = join(bare, 'objects', 'not-created')
  const result = invoke(['--validate', target, ...ctxArgs({ owner: 'source-only', roots: [source] })], { cwd: source })
  assert.notEqual(result.status, 0)
  assert.match(result.stderr, /outside|Git/)
  assert.equal(existsSync(target), false)
})

test('damaged unlisted Git metadata is not misclassified as a known non-Git destination', () => {
  const broken = join(root, 'unlisted-broken-git')
  mkdirSync(broken)
  writeFileSync(join(broken, '.git'), 'gitdir: unavailable-administration\n')
  const target = join(broken, 'not-created')
  const result = invoke(['--validate', target, ...ctxArgs({ owner: 'source-only', roots: [source] })], { cwd: source })
  assert.notEqual(result.status, 0)
  assert.equal(existsSync(target), false)
})

test('denied destination Git discovery fails closed without creating output', () => {
  const target = join(outside, 'denied-destination-must-not-create')
  const program = `
    import childProcess from 'node:child_process';
    import { syncBuiltinESMExports } from 'node:module';
    const original = childProcess.spawnSync;
    childProcess.spawnSync = (command, args, options) => {
      if (command === 'git' && args.includes('--is-inside-git-dir')) return { status: 128, stdout: '', stderr: 'fatal: permission denied reading destination Git administration' };
      return original(command, args, options);
    };
    syncBuiltinESMExports();
    const api = await import(${JSON.stringify(pathToFileURL(cli).href)});
    api.getArtifactDirectory('', { directory: ${JSON.stringify(target)}, context: ${JSON.stringify(context)} });
  `
  const result = spawnSync(process.execPath, ['--input-type=module', '-e', program], { cwd: outside, env: environment(), encoding: 'utf8', windowsHide: true })
  assert.notEqual(result.status, 0)
  assert.match(result.stderr, /permission denied/)
  assert.equal(existsSync(target), false)
})

test('legacy v1 sessions remain usable only for their original single repository context', () => {
  const legacy = mkdtempSync(join(outside, 'netcircuit-'))
  writeFileSync(join(legacy, '.netcircuit-artifact-session.json'), JSON.stringify({ version: 1, directory: realpathSync(legacy), repository: realpathSync(repoA), id: 'legacy' }))
  const result = invoke(['--cleanup', legacy], { cwd: repoA })
  assert.equal(result.status, 0, result.stderr)
  assert.equal(existsSync(legacy), false)
})

test('same-process cache cannot reuse a session belonging to another context', async () => {
  const api = await import(pathToFileURL(cli))
  const saved = { ...process.env }
  for (const key of Object.keys(process.env)) if (/^(?:CIRNET|NETCIRCUIT)_ARTIFACTS_/.test(key)) delete process.env[key]
  const firstContext = { owner: 'first', roots: [repoA, source] }
  const secondContext = { owner: 'second', roots: [repoA, source] }
  let first, second
  try {
    first = api.getArtifactDirectory('', { directory: outside, context: firstContext })
    second = api.getArtifactDirectory('', { directory: outside, context: secondContext })
    assert.notEqual(first, second)
  } finally {
    if (first) api.cleanupArtifactSession(first, { context: firstContext })
    if (second) api.cleanupArtifactSession(second, { context: secondContext })
    for (const key of Object.keys(process.env)) if (/^(?:CIRNET|NETCIRCUIT)_ARTIFACTS_/.test(key)) delete process.env[key]
    Object.assign(process.env, saved)
  }
})


test('canonical configuration wins while old v2 sessions remain valid and ownership checked', () => {
  const canonical = allocate()
  const legacy = mkdtempSync(join(outside, 'netcircuit-'))
  const owned = JSON.parse(readFileSync(join(canonical, '.cirnet-artifact-session.json'), 'utf8'))
  writeFileSync(join(legacy, '.netcircuit-artifact-session.json'), JSON.stringify({ ...owned, directory: realpathSync(legacy) }))
  const env = environment({ CIRNET_ARTIFACTS_SESSION: canonical, CIRNET_ARTIFACTS_CONTEXT: JSON.stringify(owned.context),
    NETCIRCUIT_ARTIFACTS_SESSION: legacy, NETCIRCUIT_ARTIFACTS_CONTEXT: 'invalid old JSON' })
  try {
    const child = invoke(['preferred'], { env })
    assert.equal(child.status, 0, child.stderr)
    assert.equal(child.stdout.trim(), join(canonical, 'preferred'))
    const oldChild = invoke(['legacy'], { env: environment({NETCIRCUIT_ARTIFACTS_SESSION: legacy, NETCIRCUIT_ARTIFACTS_CONTEXT: JSON.stringify(owned.context)}) })
    assert.equal(oldChild.status, 0, oldChild.stderr)
    assert.equal(oldChild.stdout.trim(), join(legacy, 'legacy'))
    const denied = invoke(['--cleanup', legacy, ...ctxArgs({owner:'wrong', roots:context.roots})])
    assert.notEqual(denied.status, 0)
    assert.ok(existsSync(legacy))
  } finally { cleanup(canonical); cleanup(legacy) }
})

test('conflicting ownership markers cannot authorize cleanup', () => {
  const session = allocate()
  const owned = JSON.parse(readFileSync(join(session, '.cirnet-artifact-session.json'), 'utf8'))
  writeFileSync(join(session, '.netcircuit-artifact-session.json'), JSON.stringify({...owned, context:{...owned.context, owner:'wrong'}}))
  const result = invoke(['--cleanup', session, ...ctxArgs()])
  assert.notEqual(result.status, 0)
  assert.ok(existsSync(session))
  writeFileSync(join(session, '.netcircuit-artifact-session.json'), JSON.stringify(owned))
  cleanup(session)
})


test('implicit reuse synchronizes both environment prefixes for child processes', async () => {
  const api = await import(pathToFileURL(cli))
  const session = allocate()
  const owned = JSON.parse(readFileSync(join(session, '.cirnet-artifact-session.json'), 'utf8'))
  const saved = { ...process.env }
  try {
    for (const key of Object.keys(process.env)) if (/^(?:CIRNET|NETCIRCUIT)_ARTIFACTS_/.test(key)) delete process.env[key]
    process.env.NETCIRCUIT_ARTIFACTS_SESSION = session
    process.env.NETCIRCUIT_ARTIFACTS_CONTEXT = JSON.stringify(owned.context)
    api.getArtifactDirectory('reused')
    assert.equal(process.env.CIRNET_ARTIFACTS_SESSION, session)
    assert.equal(process.env.NETCIRCUIT_ARTIFACTS_SESSION, session)
    assert.equal(process.env.CIRNET_ARTIFACTS_CONTEXT, process.env.NETCIRCUIT_ARTIFACTS_CONTEXT)
    assert.match(session, /[\\/]cirnet-[A-Za-z0-9]+$/)
  } finally {
    cleanup(session)
    for (const key of Object.keys(process.env)) if (/^(?:CIRNET|NETCIRCUIT)_ARTIFACTS_/.test(key)) delete process.env[key]
    Object.assign(process.env, saved)
  }
})
