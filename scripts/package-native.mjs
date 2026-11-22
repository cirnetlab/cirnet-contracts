import { execFileSync } from 'node:child_process'
import { createHash } from 'node:crypto'
import { copyFile, lstat, mkdir, mkdtemp, readFile, rename, writeFile } from 'node:fs/promises'
import { dirname, isAbsolute, join, relative, resolve, sep } from 'node:path'
import { fileURLToPath } from 'node:url'
import { getArtifactDirectory, validateArtifactDirectory } from '../tools/artifact-paths/artifact-paths.mjs'
import { sourceIdentity } from './source-identity.mjs'

const root = fileURLToPath(new URL('../', import.meta.url))
const target = 'x86_64-pc-windows-msvc'
const binary = 'cirnet-contracts.exe'
if (process.argv.length !== 3 || !process.env.CIRNET_ARTIFACTS_SESSION) {
  throw Error('Usage: set CIRNET_ARTIFACTS_SESSION, then run node scripts/package-native.mjs <fresh-output-directory>')
}
if (process.platform !== 'win32' || process.arch !== 'x64') throw Error('Use an x64 Windows MSVC build host')

const session = getArtifactDirectory()
function sessionChild(path) {
  const child = validateArtifactDirectory(resolve(path))
  const rest = relative(session, child)
  if (!rest || rest === '..' || rest.startsWith('..' + sep) || isAbsolute(rest)) {
    throw Error('Expected a child of the owned external session: ' + child)
  }
  return child
}

const output = sessionChild(process.argv[2])
const targetDir = process.env.CARGO_TARGET_DIR
  ? sessionChild(process.env.CARGO_TARGET_DIR)
  : getArtifactDirectory('contracts-native-target')
const cargo = process.env.CIRNET_CARGO ?? 'cargo'
const rustc = process.env.RUSTC ?? 'rustc'
const metadata = { cwd: root, encoding: 'utf8', windowsHide: true }
const source = await sourceIdentity(root)
const pinned = (await readFile(join(root, 'rust-toolchain.toml'), 'utf8')).match(/^channel\s*=\s*"([^"]+)"/m)?.[1]
const cargoManifest = await readFile(join(root, 'Cargo.toml'), 'utf8')
const version = cargoManifest.match(/^version\s*=\s*"([^"]+)"/m)?.[1]
const vendoredProtocVersion = cargoManifest.match(/^protoc-bin-vendored\s*=\s*"=([^"]+)"/m)?.[1]
const rustcVersion = execFileSync(rustc, ['-vV'], metadata).trim()
const cargoVersion = execFileSync(cargo, ['--version'], metadata).trim()
if (!pinned || !version || !vendoredProtocVersion || !rustcVersion.split(/\r?\n/).includes('release: ' + pinned) ||
    !rustcVersion.split(/\r?\n/).includes('host: ' + target) ||
    !cargoVersion.startsWith('cargo ' + pinned + ' ')) {
  throw Error('Use the Rust and Cargo versions selected by rust-toolchain.toml on ' + target)
}

async function requireAbsent(path) {
  try {
    await lstat(path)
  } catch (error) {
    if (error.code === 'ENOENT') return
    throw error
  }
  throw Error('Output already exists: ' + path)
}
await requireAbsent(output)
await mkdir(dirname(output), { recursive: true })
const digest = bytes => createHash('sha256').update(bytes).digest('hex')
const explicitProtoc = process.env.PROTOC
let protoc = { provider: 'cargo-vendored', crateVersion: vendoredProtocVersion }
if (explicitProtoc) {
  if (!isAbsolute(explicitProtoc) || !(await lstat(explicitProtoc)).isFile()) {
    throw Error('PROTOC must be an absolute regular compiler file')
  }
  const sha256 = digest(await readFile(explicitProtoc))
  if (sha256 !== process.env.CIRNET_PROTOC_SHA256) throw Error('PROTOC SHA-256 does not match the selected compiler')
  protoc = { provider: 'explicit', sha256, version: execFileSync(explicitProtoc, ['--version'], metadata).trim() }
}

const cargoEnv = { ...process.env, CARGO_TARGET_DIR: targetDir }
execFileSync(cargo, ['build', '--release', '--locked', '--no-default-features', '--target', target, '--bin', 'cirnet-contracts'], {
  cwd: root,
  stdio: 'inherit',
  windowsHide: true,
  env: {
    ...cargoEnv,
    CARGO_ENCODED_RUSTFLAGS: '-C\x1ftarget-feature=+crt-static',
  },
})

const crateArgs = ['package', '--locked', '--allow-dirty', '--no-verify']
const packagedPaths = execFileSync(cargo, [...crateArgs, '--list'], { ...metadata, env: cargoEnv }).trim().split(/\r?\n/)
if (packagedPaths.some(path => /\.(py|pyc|pyd|whl)$/i.test(path))) throw Error('Python file in native crate inventory')
execFileSync(cargo, crateArgs, { cwd: root, stdio: 'inherit', windowsHide: true, env: cargoEnv })
const crateFilename = `cirnet-contracts-native-${version}.crate`
const cratePath = join(targetDir, 'package', crateFilename)
const crateBytes = await readFile(cratePath)
const nativeBinary = join(targetDir, target, 'release', binary)
const descriptor = execFileSync(nativeBinary, ['descriptor'], { cwd: root, windowsHide: true })

const stage = await mkdtemp(join(dirname(output), '.contracts-native-'))
const files = []
async function include(source, path) {
  const info = await lstat(source)
  if (!info.isFile() || info.isSymbolicLink()) throw Error('Expected a regular build input: ' + source)
  const destination = join(stage, path)
  await mkdir(dirname(destination), { recursive: true })
  await copyFile(source, destination)
  const bytes = await readFile(destination)
  files.push({ path, size: bytes.length, sha256: digest(bytes) })
}

const resources = source.files.filter(({ path }) => {
  return /^(contracts|device-library)\//.test(path) && /\.(?:json|proto)$/.test(path) &&
    !path.split('/').some(part => ['fixtures', 'examples', 'browser'].includes(part)) &&
    path !== 'contracts/compatibility/2026-10-16/hardware/v2/rpc_v2.proto'
})
await include(join(root, 'LICENSE'), 'LICENSE')
await include(nativeBinary, 'bin/' + binary)
for (const { path } of resources) await include(join(root, path), 'resources/' + path)
files.sort((a, b) => a.path < b.path ? -1 : a.path > b.path ? 1 : 0)
if ((await sourceIdentity(root)).sourceSha256 !== source.sourceSha256) {
  throw Error('Contracts source changed during the build')
}
const manifest = {
  schemaVersion: 1,
  component: 'cirnet-contracts',
  version,
  revision: source.revision,
  sourceDirty: source.sourceDirty,
  sourceSha256: source.sourceSha256,
  target,
  rustc: rustcVersion,
  cargo: cargoVersion,
  cargoLockSha256: digest(await readFile(join(root, 'Cargo.lock'))),
  protoc,
  rpcDescriptorSha256: digest(descriptor),
  crate: { filename: crateFilename, size: crateBytes.length, sha256: digest(crateBytes) },
  crt: 'static',
  files,
}
await writeFile(join(stage, 'contracts-native-manifest.json'), JSON.stringify(manifest, null, 2) + '\n')
await requireAbsent(output)
await rename(stage, output)
console.log('CONTRACTS_NATIVE_PACKAGE_PATH ' + output)
console.log('CONTRACTS_NATIVE_CRATE_PATH ' + cratePath)
