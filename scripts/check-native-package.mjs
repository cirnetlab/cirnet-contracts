import assert from 'node:assert/strict'
import { createHash } from 'node:crypto'
import { execFileSync, spawnSync } from 'node:child_process'
import { readFile, readdir } from 'node:fs/promises'
import { basename, join, resolve } from 'node:path'

const manifestName = 'contracts-native-manifest.json'
const binaryName = 'bin/cirnet-contracts.exe'
const root = resolve(process.argv[2] ?? '')
const crate = process.argv[3]
if (process.argv.length < 3 || process.argv.length > 4) throw Error('Usage: node scripts/check-native-package.mjs <bundle> [crate]')

async function inventory(directory, prefix = '') {
  const files = []
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const path = prefix + entry.name
    if (entry.isDirectory()) files.push(...await inventory(join(directory, entry.name), path + '/'))
    else if (entry.isFile()) files.push(path)
    else throw Error('Unexpected packaged entry: ' + path)
  }
  return files
}

const manifest = JSON.parse(await readFile(join(root, manifestName), 'utf8'))
assert.equal(manifest.schemaVersion, 1)
assert.equal(manifest.component, 'cirnet-contracts')
assert.equal(manifest.target, 'x86_64-pc-windows-msvc')
assert.equal(manifest.crt, 'static')
assert.match(manifest.sourceSha256, /^[0-9a-f]{64}$/)
if (manifest.revision !== null) assert.match(manifest.revision, /^[0-9a-f]{40}$/)
assert.match(manifest.version, /^\d+\.\d+\.\d+-dev\.\d+$/)
assert.match(manifest.cargoLockSha256, /^[0-9a-f]{64}$/)
assert.match(manifest.rustc, /release: \d+\.\d+\.\d+/)
assert.match(manifest.cargo, /^cargo \d+\.\d+\.\d+/)
assert.equal(manifest.crate.filename, `cirnet-contracts-native-${manifest.version}.crate`)
assert(Number.isSafeInteger(manifest.crate.size) && manifest.crate.size > 0)
assert.match(manifest.crate.sha256, /^[0-9a-f]{64}$/)
if (crate) {
  const bytes = await readFile(crate)
  assert.equal(basename(crate), manifest.crate.filename)
  assert.equal(bytes.length, manifest.crate.size)
  assert.equal(createHash('sha256').update(bytes).digest('hex'), manifest.crate.sha256)
  const members = execFileSync('tar', ['-tzf', '-'], { input: bytes, encoding: 'utf8', windowsHide: true }).trim().split(/\r?\n/)
  assert(members.length > 1)
  assert(!members.some(path => /\.(py|pyc|pyd|whl)$|(^|\/)node_modules\//i.test(path)))
}

const paths = manifest.files.map(file => file.path)
assert.deepEqual((await inventory(root)).sort(), [...paths, manifestName].sort())
assert(!paths.includes('resources/contracts/compatibility/2026-10-16/hardware/v2/rpc_v2.proto'))
for (const required of [binaryName, 'LICENSE', 'resources/contracts/manifest.json', 'resources/contracts/api/openapi.json', 'resources/contracts/profiles.json']) {
  assert(paths.includes(required), 'Missing packaged file: ' + required)
}
for (const { path, size, sha256 } of manifest.files) {
  assert(path === binaryName || path === 'LICENSE' || /^resources\/(?:contracts|device-library)\/(?:[a-zA-Z0-9_.-]+\/)*[a-zA-Z0-9_.-]+\.(?:json|proto)$/.test(path))
  assert(!path.split('/').some(part => part === '.' || part === '..' || part === 'fixtures'))
  const bytes = await readFile(join(root, path))
  assert.equal(bytes.length, size, path)
  assert.equal(createHash('sha256').update(bytes).digest('hex'), sha256, path)
}

const binary = join(root, binaryName)
const options = { cwd: root, windowsHide: true, timeout: 20_000 }
const health = { status: 'ok', service: 'cirnet-api', hardware_mode: 'simulation' }
const validate = ['validate', join(root, 'resources'), 'HealthResponse', 'transport']
assert.deepEqual(JSON.parse(execFileSync(binary, validate, { ...options, input: JSON.stringify(health) })), health)
const invalid = spawnSync(binary, validate, { ...options, input: '{"status":false}', encoding: 'utf8' })
assert.equal(invalid.status, 2, invalid.stderr)
assert.equal(typeof JSON.parse(invalid.stderr), 'object')

const graph = await readFile(new URL('../contracts/circuit-schema/examples/physical-hc08-led.json', import.meta.url), 'utf8')
const validatedGraph = execFileSync(binary, ['validate', join(root, 'resources'), 'CircuitGraph', 'transport'], { ...options, input: graph })
assert.equal(JSON.parse(validatedGraph).circuit_id, JSON.parse(graph).circuit_id)

const canonical = execFileSync(binary, ['descriptor'], options)
assert(canonical.includes(Buffer.from('cirnet.hardware.v2')))
assert.equal(createHash('sha256').update(canonical).digest('hex'), manifest.rpcDescriptorSha256)
const retired = spawnSync(binary, ['descriptor', 'legacy'], options)
assert.equal(retired.status, 1)
assert.equal(retired.stdout.length, 0)
console.log('CONTRACTS_NATIVE_PACKAGE_OK ' + manifest.version + ' ' + manifest.files.length + ' files')
