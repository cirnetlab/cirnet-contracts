import { execFileSync } from 'node:child_process'
import { createHash } from 'node:crypto'
import { readFile, readdir, realpath } from 'node:fs/promises'
import { join } from 'node:path'

const ignored = new Set(['.git', '.superpowers', '.build', 'node_modules', 'target', 'dist'])
const digest = bytes => createHash('sha256').update(bytes).digest('hex')
const compare = (left, right) => left < right ? -1 : left > right ? 1 : 0

export async function sourceIdentity(root) {
  const files = []
  async function collect(directory, prefix = '') {
    const entries = await readdir(directory, { withFileTypes: true })
    for (const entry of entries) {
      if (ignored.has(entry.name)) continue
      const path = prefix + entry.name
      const absolute = join(directory, entry.name)
      if (entry.isDirectory()) {
        await collect(absolute, path + '/')
      } else {
        if (!entry.isFile()) throw Error('Expected a regular source file: ' + path)
        files.push({ path, sha256: digest(await readFile(absolute)) })
      }
    }
  }
  await collect(root)
  files.sort((left, right) => compare(left.path, right.path))

  const git = (...args) => execFileSync('git', args, {
    cwd: root, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'], windowsHide: true,
  })
  let revision = null
  let sourceDirty = null
  let gitRoot
  try {
    gitRoot = git('rev-parse', '--show-toplevel').trim()
  } catch (error) {
    if (error.status !== 128 && error.code !== 'ENOENT') throw error
  }
  if (gitRoot && await realpath(gitRoot) === await realpath(root)) {
    revision = git('rev-parse', 'HEAD').trim()
    sourceDirty = Boolean(git('status', '--porcelain'))
  }
  return { revision, sourceDirty, sourceSha256: digest(JSON.stringify(files)), files }
}
