export interface ArtifactContext {
  version?: 1
  owner: string
  roots: string[]
}
export interface ArtifactOptions {
  directory?: string
  context?: ArtifactContext
}
export function getArtifactContext(options?: { context?: ArtifactContext }): ArtifactContext & { version: 1 }
export function getArtifactDirectory(subdirectory?: string, options?: ArtifactOptions): string
export function validateArtifactDirectory(directory: string, options?: { context?: ArtifactContext }): string
export function cleanupArtifactSession(directory: string, options?: { context?: ArtifactContext }): void
export function runArtifactCli(arguments_?: string[]): void
