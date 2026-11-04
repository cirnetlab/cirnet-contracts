// Compatibility entrypoint; policy is owned by cirnet_artifact_paths.
export * from '../cirnet_artifact_paths/artifact-paths.mjs'
import { runArtifactCli } from '../cirnet_artifact_paths/artifact-paths.mjs'
import { resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) runArtifactCli()
