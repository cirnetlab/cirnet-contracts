/** Shared verification output allocator. Never writes evidence into a checkout. */
import { execFileSync, spawnSync } from 'node:child_process'
import { existsSync, realpathSync, lstatSync, mkdirSync, mkdtempSync, writeFileSync, readFileSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { resolve, dirname, join, relative, isAbsolute, basename, parse } from 'node:path'
import { fileURLToPath } from 'node:url'
import { createHash, randomUUID } from 'node:crypto'

const marker = '.cirnet-artifact-session.json'
const legacyMarker = '.netcircuit-artifact-session.json'
function getenv(name) {
  const current = process.env[name]
  const legacy = process.env[name.replace(/^CIRNET_/, 'NETCIRCUIT_')]
  if (process.env.CIRNET_LOCAL_STORAGE_ROOT !== undefined && current !== undefined && legacy !== undefined && current !== legacy) {
    throw new Error('Conflicting artifact configuration for ' + name + '.')
  }
  return current ?? legacy
}
function setenv(name, value) { process.env[name] = value; process.env[name.replace(/^CIRNET_/, 'NETCIRCUIT_')] = value }
function selectedTaskKey() {
  const key = process.env.CIRNET_ARTIFACTS_TASK_KEY
  if (key !== undefined && !/^[a-f0-9]{32}$/.test(key)) throw new Error('Invalid CIRNET_ARTIFACTS_TASK_KEY.')
  return key
}
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
    if (['GIT_DIR', 'GIT_WORK_TREE', 'GIT_COMMON_DIR', 'GIT_INDEX_FILE', 'GIT_OBJECT_DIRECTORY',
      'GIT_ALTERNATE_OBJECT_DIRECTORIES', 'GIT_CONFIG_COUNT', 'GIT_CONFIG_PARAMETERS',
      'GIT_CEILING_DIRECTORIES', 'GIT_DISCOVERY_ACROSS_FILESYSTEM', 'GIT_CONFIG'].includes(key.toUpperCase())) delete environment[key]
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
  const protectedRoots = process.env.CIRNET_LOCAL_STORAGE_ROOT === undefined ? protectedPaths(context) : roots
  if (current && !protectedRoots.includes(current)) {
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
function unaliasedPath(directory) {
  if (typeof directory !== 'string' || !isAbsolute(directory) ||
      (process.platform === 'win32' && !/^[A-Z]:[\\/]/i.test(directory)) ||
      directory.slice(parse(directory).root.length).split(process.platform === 'win32' ? /[\\/]/ : '/')
        .some(part => part === '..' || part === '.' || (process.platform === 'win32' && /[:]|[. ]$/.test(part)))) {
    throw new Error('Expected an absolute local artifact path without traversal or aliases: ' + directory)
  }
  const absolute = resolve(directory)
  for (let part = absolute; ; part = dirname(part)) {
    const stat = lstatSync(part, { throwIfNoEntry: false })
    if (stat?.isSymbolicLink()) throw new Error('Artifact path traverses a symbolic link or junction: ' + part)
    if (dirname(part) === part) break
  }
  const actual = canonical(absolute)
  const same = process.platform === 'win32' ? absolute.toLowerCase() === actual.toLowerCase() : absolute === actual
  if (!same) throw new Error('Artifact path uses a filesystem alias: ' + directory)
  return actual
}

const canonicalRepos = /^cirnet-(?:rules|contracts|remote|interface|simulator|hardware|embedded-hardware|cli)$/

function configuredArtifactParent(context, protection) {
  const value = process.env.CIRNET_LOCAL_STORAGE_ROOT
  if (value === undefined) return undefined
  const base = unaliasedPath(value)
  if (!lstatSync(base, { throwIfNoEntry: false })?.isDirectory()) throw new Error('CIRNET_LOCAL_STORAGE_ROOT is unavailable: ' + base)
  // The candidate has one integration owner even while commands run in other checkouts.
  const ownerId = context.owner === 'cirnet-native-candidate' ? 'cirnet-remote' : context.owner.replace(/-ci$/, '')
  const ownerRoots = context.owner.startsWith('git:')
    ? context.roots.filter(root => root === context.owner.slice(4))
    : context.roots
  const ids = context.roots.map(root => {
    let urls
    try { urls = git(root, 'config', '--local', '--no-includes', '--null', '--get-all', 'remote.origin.url').split('\0') }
    catch { throw new Error('Cannot establish a canonical CIRNETLAB origin for ' + root) }
    const match = urls.length === 2 && urls[0].match(/^(?:https:\/\/github\.com\/|git@github\.com:|ssh:\/\/git@github\.com\/)cirnetlab\/([a-z0-9-]+?)(?:\.git)?$/i)
    return match && match[1].toLowerCase()
  })
  const id = context.owner.startsWith('git:') ? ids[context.roots.indexOf(ownerRoots[0])] : ownerId
  if (!id || !canonicalRepos.test(id) || ids.some(value => !value || !canonicalRepos.test(value)) ||
      (!context.owner.startsWith('git:') && ids.filter(value => value === id).length !== 1)) {
    throw new Error('CIRNET_LOCAL_STORAGE_ROOT requires a canonical GitHub origin for the artifact owner.')
  }
  // An otherwise valid override must not suppress a broken configured root.
  validateDirectory(base, context, protection)
  return validateDirectory(join(base, '.cirnet-build-output', id), context, protection)
}

function validateDirectory(directory, context, protection = protectedPaths(context)) {
  if (!isAbsolute(directory)) throw new Error('Artifact directory must be absolute and outside the repository.')
  const result = process.env.CIRNET_LOCAL_STORAGE_ROOT === undefined ? canonical(directory) : unaliasedPath(directory)
  for (const protectedPath of protection) {
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
function readSession(directory, context, protection) {
  const actual = validateDirectory(directory, context, protection)
  if (resolve(directory) !== actual || lstatSync(actual).isSymbolicLink()) throw new Error('Artifact session cannot be a symlink alias.')
  const markers = [marker, legacyMarker].filter(name => existsSync(join(actual, name)))
  if (!markers.length) throw new Error('Refusing an unowned artifact session.')
  const markerBytes = markers.map(name => {
    const path = join(actual, name)
    if (lstatSync(path).isSymbolicLink() || !lstatSync(path).isFile()) throw new Error('Artifact marker must be a regular owned file.')
    return readFileSync(path)
  })
  const records = markerBytes.map(bytes => JSON.parse(bytes))
  if (records.some(record => JSON.stringify(record) !== JSON.stringify(records[0]))) throw new Error('Conflicting artifact ownership markers.')
  const ownership = records[0]
  const compatibleLegacy = ownership.version === 1 && context.roots.length === 1 &&
    ownership.repository === context.roots[0] && context.owner === `git:${context.roots[0]}`
  const ownedContext = ownership.version === 2 && JSON.stringify(ownership.context) === JSON.stringify(context)
  if (ownership.taskKey !== selectedTaskKey()) throw new Error('Artifact session belongs to a different task.')
  if ((!compatibleLegacy && !ownedContext) || ownership.directory !== actual || !/^(?:cirnet|netcircuit)-[A-Za-z0-9]+$/.test(basename(actual))) {
    throw new Error('Refusing an unowned artifact session.')
  }
  return { directory: actual, context, markerSha256: createHash('sha256').update(markerBytes[0]).digest('hex') }
}

function selectArtifactParent(context, protection, explicitParent, inheritedSession) {
  const configured = configuredArtifactParent(context, protection)
  const inheritedParent = getenv('CIRNET_ARTIFACTS_DIR')
  const selectedParent = inheritedParent === undefined ? undefined : validateDirectory(inheritedParent, context, protection)
  if (configured && selectedParent && !inside(configured, selectedParent) && explicitParent === undefined && !inheritedSession) {
    throw new Error('Select a new alternate parent explicitly with --directory. CIRNET_ARTIFACTS_DIR is outside the configured repository namespace.')
  }
  const parent = explicitParent === undefined
    ? selectedParent ?? configured ?? validateDirectory(tmpdir(), context, protection)
    : validateDirectory(explicitParent, context, protection)
  if (configured && explicitParent !== undefined && selectedParent && selectedParent !== parent) {
    throw new Error('Explicit artifact parent conflicts with CIRNET_ARTIFACTS_DIR.')
  }
  return { parent, configured, inheritedParent }
}

export function getArtifactParent(options = {}) {
  const context = getArtifactContext(options)
  const protection = protectedPaths(context)
  const inherited = getenv('CIRNET_ARTIFACTS_SESSION')
  const { parent, configured } = selectArtifactParent(context, protection, options.directory, inherited)
  if (configured && inherited !== undefined && dirname(readSession(inherited, context, protection).directory) !== parent) {
    throw new Error('Inherited artifact session does not belong to the selected artifact parent.')
  }
  return parent
}

function validateSubdirectory(subdirectory) {
  if (typeof subdirectory !== 'string' || isAbsolute(subdirectory) ||
      subdirectory.split(/[\\/]/).some(part => part === '..' || part.includes('\0') ||
        (process.platform === 'win32' && (/[:*?"<>|]|[. ]$/.test(part) || /^(?:CON|PRN|AUX|NUL|COM[1-9¹²³]|LPT[1-9¹²³])(?:\.|$)/i.test(part))))) {
    throw new Error('Artifact subdirectory must stay inside its session.')
  }
}

export function inspectArtifactSession(options = {}) {
  const context = getArtifactContext(options)
  const inherited = getenv('CIRNET_ARTIFACTS_SESSION')
  if (!inherited) throw new Error('Set CIRNET_ARTIFACTS_SESSION to an existing owned session.')
  const protection = protectedPaths(context)
  const session = readSession(inherited, context, protection)
  if (process.env.CIRNET_LOCAL_STORAGE_ROOT !== undefined) {
    const { parent } = selectArtifactParent(context, protection, undefined, inherited)
    if (dirname(session.directory) !== parent) {
      throw new Error('Inherited artifact session does not belong to the selected artifact parent.')
    }
  } else if (getenv('CIRNET_ARTIFACTS_DIR')) {
    validateDirectory(getenv('CIRNET_ARTIFACTS_DIR'), context, protection)
  }
  return session
}

export function getExistingArtifactDirectory(subdirectory = '', options = {}) {
  validateSubdirectory(subdirectory)
  const session = inspectArtifactSession(options).directory
  const directory = join(session, subdirectory)
  if (!lstatSync(directory, { throwIfNoEntry: false })?.isDirectory() || realpathSync(directory) !== directory) {
    throw new Error('Expected an existing ordinary artifact directory: ' + directory)
  }
  rejectGitDestination(directory)
  return directory
}

export function getArtifactDirectory(subdirectory = '', options = {}) {
  validateSubdirectory(subdirectory)
  const taskKey = selectedTaskKey()
  const context = getArtifactContext(options)
  const protection = protectedPaths(context)
  const explicitParent = options.directory
  const inheritedSession = getenv('CIRNET_ARTIFACTS_SESSION')
  const legacyReuse = process.env.CIRNET_LOCAL_STORAGE_ROOT === undefined && !explicitParent && inheritedSession
  const { parent, configured, inheritedParent } = legacyReuse
    ? { inheritedParent: getenv('CIRNET_ARTIFACTS_DIR') }
    : selectArtifactParent(context, protection, explicitParent, inheritedSession)
  let session
  if (legacyReuse || (configured && inheritedSession !== undefined)) {
    if (legacyReuse && inheritedParent) validateDirectory(inheritedParent, context, protection)
    session = readSession(inheritedSession, context, protection).directory
    if (configured && dirname(session) !== parent) {
      throw new Error('Inherited artifact session does not belong to the selected artifact parent.')
    }
  } else {
    const cacheKey = JSON.stringify([parent, context, taskKey])
    session = sessions.get(cacheKey)
    if (session && !existsSync(session)) { sessions.delete(cacheKey); session = undefined }
    if (session) readSession(session, context, protection)
    if (!session) {
      mkdirSync(parent, { recursive: true })
      // Recheck after creation; a pre-existing symlink must never redirect output.
      const checked = validateDirectory(parent, context, protection)
      session = realpathSync(mkdtempSync(join(checked, 'cirnet-')))
      writeFileSync(join(session, marker), JSON.stringify({ version: 2, directory: session, context, taskKey, id: randomUUID(), createdAt: new Date().toISOString() }) + '\n', { flag: 'wx' })
      sessions.set(cacheKey, session)
    }
  }
  if (explicitParent === undefined) {
    setenv('CIRNET_ARTIFACTS_SESSION', session)
    setenv('CIRNET_ARTIFACTS_CONTEXT', JSON.stringify(context))
    if (inheritedParent !== undefined) setenv('CIRNET_ARTIFACTS_DIR', inheritedParent)
  }
  const output = validateDirectory(join(session, subdirectory), context, protection)
  if (!inside(session, output)) throw new Error('Artifact subdirectory escapes its session.')
  mkdirSync(output, { recursive: true })
  if (!inside(session, validateDirectory(output, context, protection))) throw new Error('Artifact output changed while creating it.')
  return output
}
export function cleanupArtifactSession(directory, options = {}) {
  const session = readSession(directory, getArtifactContext(options)).directory
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
    } else if (args[0] === '--session-info') {
      if (args.length !== 1) throw new Error('Usage: --session-info')
      console.log(JSON.stringify(inspectArtifactSession(options)))
    } else if (args[0] === '--parent-info') {
      if (args.length !== 1) throw new Error('Usage: --parent-info')
      console.log(getArtifactParent(options))
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
