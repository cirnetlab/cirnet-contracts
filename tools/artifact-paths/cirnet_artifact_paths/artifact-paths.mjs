/** Shared verification output allocator. Never writes evidence into a checkout. */
import { execFileSync, spawnSync } from 'node:child_process'
import { existsSync, realpathSync, lstatSync, mkdirSync, mkdtempSync, writeFileSync, readFileSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { resolve, dirname, join, relative, isAbsolute, basename } from 'node:path'
import { fileURLToPath } from 'node:url'
import { randomUUID } from 'node:crypto'

const marker = '.cirnet-artifact-session.json'
const legacyMarker = '.netcircuit-artifact-session.json'
function getenv(name) { return process.env[name] ?? process.env[name.replace(/^CIRNET_/, 'NETCIRCUIT_')] }
function setenv(name, value) { process.env[name] = value; process.env[name.replace(/^CIRNET_/, 'NETCIRCUIT_')] = value }
const sessions = new Map()
function canonical(path) {
  const absolute = resolve(path)
  if (existsSync(absolute)) return realpathSync(absolute)
  const parent = dirname(absolute)
  if (parent === absolute) throw new Error(`Cannot resolve artifact path: ${absolute}`)
  return join(canonical(parent), basename(absolute))
}
function inside(parent, child) {
  const path = relative(parent, child)
  return path === '' || (!isAbsolute(path) && path !== '..' && !path.startsWith('..' + (process.platform === 'win32' ? '\\' : '/')))
}
function gitEnvironment() {
  const environment = { ...process.env, LC_ALL: 'C', LANG: 'C' }
  for (const key of Object.keys(environment)) {
    if (['GIT_DIR', 'GIT_WORK_TREE', 'GIT_COMMON_DIR', 'GIT_INDEX_FILE', 'GIT_OBJECT_DIRECTORY', 'GIT_ALTERNATE_OBJECT_DIRECTORIES'].includes(key.toUpperCase())) delete environment[key]
  }
  return environment
}
function knownNonGitDiscovery(stderr) {
  return /^fatal: not a git repository \(or any of the parent directories\): \.git(?:\r?\n|$)/.test(stderr)
}
function git(repository, ...args) {
  return execFileSync('git', ['-C', repository, ...args], { encoding: 'utf8', windowsHide: true, stdio: ['ignore', 'pipe', 'pipe'], env: gitEnvironment() }).trim()
}
function invocationRepository() {
  const result = spawnSync('git', ['-C', process.cwd(), 'rev-parse', '--show-toplevel'], { encoding: 'utf8', windowsHide: true, env: gitEnvironment() })
  if (result.error) throw result.error
  if (result.status === 0) return canonical(result.stdout.trim())
  if (knownNonGitDiscovery(result.stderr)) return undefined
  throw new Error(`Cannot enumerate current Git context: ${result.stderr.trim()}`)
}
export function getArtifactContext(options = {}) {
  const current = invocationRepository()
  let supplied = options.context
  if (supplied === undefined && getenv('CIRNET_ARTIFACTS_CONTEXT')) {
    try { supplied = JSON.parse(getenv('CIRNET_ARTIFACTS_CONTEXT')) }
    catch { throw new Error('Invalid inherited artifact context JSON.') }
  }
  if (supplied === undefined) {
    if (!current) throw new Error('Artifact context is required outside a Git checkout; supply owner and involved roots.')
    supplied = { owner: `git:${current}`, roots: [current] }
  }
  if (!supplied || typeof supplied !== 'object' || typeof supplied.owner !== 'string' || !supplied.owner.trim() ||
      !Array.isArray(supplied.roots) || !supplied.roots.length || (supplied.version !== undefined && supplied.version !== 1)) {
    throw new Error('Artifact context requires a nonempty owner and nonempty absolute Git roots.')
  }
  const roots = [...new Set(supplied.roots.map(root => {
    if (typeof root !== 'string' || !isAbsolute(root)) throw new Error('Artifact context roots must be absolute Git checkout paths.')
    return canonical(git(canonical(root), 'rev-parse', '--show-toplevel'))
  }))].sort()
  const context = { version: 1, owner: supplied.owner, roots }
  const protectedRoots = protectedPaths(context)
  if (current && !protectedRoots.some(root => root === current)) {
    throw new Error('Artifact context omits the current Git repository; include every involved root.')
  }
  return context
}
function protectedPaths(context) {
  const paths = []
  for (const repository of context.roots) {
    // Every query must succeed; permission failures must never reduce protection.
    const worktrees = git(repository, 'worktree', 'list', '--porcelain', '-z').split('\0').filter(line => line.startsWith('worktree ')).map(line => line.slice(9))
    paths.push(repository, git(repository, 'rev-parse', '--absolute-git-dir'), resolve(repository, git(repository, 'rev-parse', '--git-common-dir')), ...worktrees)
  }
  return [...new Set(paths.map(canonical))]
}
function rejectGitDestination(directory) {
  let ancestor = directory
  while (!existsSync(ancestor)) {
    const parent = dirname(ancestor)
    if (parent === ancestor) throw new Error(`Cannot inspect artifact destination ancestor: ${directory}`)
    ancestor = parent
  }
  const result = spawnSync('git', ['-C', ancestor, 'rev-parse', '--is-inside-work-tree', '--is-inside-git-dir', '--is-bare-repository'], {
    encoding: 'utf8', windowsHide: true, env: gitEnvironment(),
  })
  if (result.error) throw result.error
  if (result.status === 0) {
    if (result.stdout.trim().split(/\s+/).includes('true')) {
      throw new Error(`Artifact directory must be outside every repository/worktree and Git directory: ${directory}`)
    }
    throw new Error(`Cannot establish a non-Git artifact destination: ${directory}`)
  }
  if (!knownNonGitDiscovery(result.stderr)) {
    throw new Error(`Cannot enumerate artifact destination Git context: ${result.stderr.trim()}`)
  }
}
function validateDirectory(directory, context) {
  if (!isAbsolute(directory)) throw new Error('Artifact directory must be absolute and outside the repository.')
  const result = canonical(directory)
  for (const protectedPath of protectedPaths(context)) {
    if (inside(protectedPath, result)) throw new Error(`Artifact directory must be outside every repository/worktree and Git directory: ${directory}`)
  }
  // Independently discover unrelated checkouts/bare Git at the destination too.
  // A caller's incomplete rootset must never authorize output inside another Git tree.
  rejectGitDestination(result)
  return result
}
export function validateArtifactDirectory(directory, options = {}) {
  return validateDirectory(directory, getArtifactContext(options))
}
function readSession(directory, context) {
  const actual = validateDirectory(directory, context)
  if (resolve(directory) !== actual || lstatSync(actual).isSymbolicLink()) throw new Error('Artifact session cannot be a symlink alias.')
  const markers = [marker, legacyMarker].filter(name => existsSync(join(actual, name)))
  if (!markers.length) throw new Error('Refusing an unowned artifact session.')
  const records = markers.map(name => {
    const path = join(actual, name)
    if (lstatSync(path).isSymbolicLink() || !lstatSync(path).isFile()) throw new Error('Artifact marker must be a regular owned file.')
    return JSON.parse(readFileSync(path, 'utf8'))
  })
  if (records.some(record => JSON.stringify(record) !== JSON.stringify(records[0]))) throw new Error('Conflicting artifact ownership markers.')
  const ownership = records[0]
  const compatibleLegacy = ownership.version === 1 && context.roots.length === 1 &&
    ownership.repository === context.roots[0] && context.owner === `git:${context.roots[0]}`
  const ownedContext = ownership.version === 2 && JSON.stringify(ownership.context) === JSON.stringify(context)
  if ((!compatibleLegacy && !ownedContext) || ownership.directory !== actual || !/^(?:cirnet|netcircuit)-[A-Za-z0-9]+$/.test(basename(actual))) {
    throw new Error('Refusing an unowned artifact session.')
  }
  return actual
}
export function getArtifactDirectory(subdirectory = '', options = {}) {
  const context = getArtifactContext(options)
  if (getenv('CIRNET_ARTIFACTS_DIR')) validateDirectory(getenv('CIRNET_ARTIFACTS_DIR'), context)
  let session
  if (!options.directory && getenv('CIRNET_ARTIFACTS_SESSION')) {
    session = readSession(getenv('CIRNET_ARTIFACTS_SESSION'), context)
  } else {
    const parent = validateDirectory(options.directory ?? getenv('CIRNET_ARTIFACTS_DIR') ?? tmpdir(), context)
    const cacheKey = JSON.stringify([parent, context])
    session = sessions.get(cacheKey)
    if (session && !existsSync(session)) { sessions.delete(cacheKey); session = undefined }
    if (!session) {
      mkdirSync(parent, { recursive: true })
      // Recheck after creation; a pre-existing symlink must never redirect output.
      const checked = validateDirectory(parent, context)
      session = realpathSync(mkdtempSync(join(checked, 'cirnet-')))
      writeFileSync(join(session, marker), JSON.stringify({ version: 2, directory: session, context, id: randomUUID(), createdAt: new Date().toISOString() }) + '\n', { flag: 'wx' })
      sessions.set(cacheKey, session)
    }
  }
  if (!options.directory) {
    setenv('CIRNET_ARTIFACTS_SESSION', session)
    setenv('CIRNET_ARTIFACTS_CONTEXT', JSON.stringify(context))
    if (getenv('CIRNET_ARTIFACTS_DIR') !== undefined) setenv('CIRNET_ARTIFACTS_DIR', getenv('CIRNET_ARTIFACTS_DIR'))
  }
  if (isAbsolute(subdirectory) || subdirectory.split(/[\\/]/).includes('..')) throw new Error('Artifact subdirectory must stay inside its session.')
  const output = validateDirectory(join(session, subdirectory), context)
  if (!inside(session, output)) throw new Error('Artifact subdirectory escapes its session.')
  mkdirSync(output, { recursive: true })
  if (!inside(session, validateDirectory(output, context))) throw new Error('Artifact output changed while creating it.')
  return output
}
export function cleanupArtifactSession(directory, options = {}) {
  const session = readSession(directory, getArtifactContext(options))
  // rm does not follow directory symlinks within the owned session.
  rmSync(session, { recursive: true, force: false })
  for (const [key, value] of sessions) if (value === session) sessions.delete(key)
  for (const key of ['CIRNET_ARTIFACTS_SESSION', 'NETCIRCUIT_ARTIFACTS_SESSION']) if (process.env[key] === session) delete process.env[key]
}
export function runArtifactCli(arguments_ = process.argv.slice(2)) {
  try {
    const args = [...arguments_]
    const contextIndex = args.indexOf('--context')
    let context
    if (contextIndex >= 0) {
      const selected = args.splice(contextIndex, 2)
      if (selected.length !== 2) throw new Error('Usage: --context JSON')
      context = JSON.parse(selected[1])
    }
    const options = { context }
    if (args[0] === '--validate') {
      if (args.length !== 2) throw new Error('Usage: --validate EXTERNAL_DIRECTORY')
      console.log(validateArtifactDirectory(args[1], options))
    } else if (args[0] === '--cleanup') {
      if (args.length !== 2) throw new Error('Usage: --cleanup SESSION')
      cleanupArtifactSession(args[1], options)
    } else if (args[0] === '--context-info') {
      if (args.length !== 1) throw new Error('Usage: --context-info')
      console.log(JSON.stringify(getArtifactContext(options)))
    } else {
      const index = args.indexOf('--directory')
      const directory = index >= 0 ? args.splice(index, 2)[1] : undefined
      if (args.length > 1 || (index >= 0 && !directory)) throw new Error('Usage: [subdirectory] [--directory EXTERNAL_PARENT]')
      console.log(getArtifactDirectory(args[0], { directory, context }))
    }
  } catch (error) {
    console.error(`artifact-paths: ${error.message}`)
    process.exitCode = 1
  }
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) runArtifactCli()
