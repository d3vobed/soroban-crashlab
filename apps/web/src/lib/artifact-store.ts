/**
 * Shared in-memory artifact store for Next.js API routes.
 * Production should replace this module with a filesystem or Rust-backed adapter.
 */
export interface StoredArtifact {
  id: string;
  name: string;
  createdAt: string;
  sizeBytes: number;
  buffer: Buffer;
}

const globalForArtifacts = globalThis as unknown as {
  crashlabArtifactStore?: Map<string, StoredArtifact>;
};

export const artifactStore: Map<string, StoredArtifact> =
  globalForArtifacts.crashlabArtifactStore ??
  new Map<string, StoredArtifact>();

if (process.env.NODE_ENV !== "production") {
  globalForArtifacts.crashlabArtifactStore = artifactStore;
}

export function listArtifactMetadata(): Array<{
  id: string;
  name: string;
  createdAt: string;
  sizeBytes: number;
}> {
  return Array.from(artifactStore.values())
    .map(({ id, name, createdAt, sizeBytes }) => ({
      id,
      name,
      createdAt,
      sizeBytes,
    }))
    .sort(
      (a, b) =>
        new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime(),
    );
}
