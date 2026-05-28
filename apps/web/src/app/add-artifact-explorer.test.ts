/**
 * Focused tests for mapping and inference logic inside the ArtifactExplorer component.
 * Verifies mapping structure from the NestJS/Rust /api/artifacts backend schema
 * into the client UI Artifact representation.
 */

import { mapMetadataToArtifact } from './add-artifact-explorer';

function assert(condition: boolean, message: string): void {
  if (!condition) {
    throw new Error(`Assertion failed: ${message}`);
  }
}

// ── test mapMetadataToArtifact — extension types ─────────────────────────────
function testExtensionMapping(): void {
  const seedMeta = {
    id: '12345',
    name: 'test_seed_input.bin',
    createdAt: '2026-05-28T22:00:00Z',
    sizeBytes: 1024,
  };
  const seedArtifact = mapMetadataToArtifact(seedMeta);
  assert(seedArtifact.type === 'seed', 'Should map .bin to seed');
  assert(seedArtifact.size === 1024, 'Should preserve sizeBytes');
  assert(seedArtifact.updatedAt === '2026-05-28T22:00:00Z', 'Should preserve createdAt');

  const xdrMeta = { ...seedMeta, name: 'mutant.xdr' };
  assert(mapMetadataToArtifact(xdrMeta).type === 'seed', 'Should map .xdr to seed');

  const logMeta = { ...seedMeta, name: 'fuzzer_run_output.log' };
  assert(mapMetadataToArtifact(logMeta).type === 'log', 'Should map .log to log');

  const txtMeta = { ...seedMeta, name: 'raw_stderr.txt' };
  assert(mapMetadataToArtifact(txtMeta).type === 'log', 'Should map .txt to log');

  const traceMeta = { ...seedMeta, name: 'execution_steps.json' };
  assert(mapMetadataToArtifact(traceMeta).type === 'trace', 'Should map .json to trace');

  const coverageMeta = { ...seedMeta, name: 'lcov_report.zip' };
  assert(mapMetadataToArtifact(coverageMeta).type === 'coverage', 'Should map .zip to coverage');

  const htmlMeta = { ...seedMeta, name: 'index.html' };
  assert(mapMetadataToArtifact(htmlMeta).type === 'coverage', 'Should map .html to coverage');

  const bundleMeta = { ...seedMeta, name: 'release_archive.tar.gz' };
  assert(mapMetadataToArtifact(bundleMeta).type === 'bundle', 'Should map .tar.gz to bundle');

  console.log('PASS: testExtensionMapping');
}

// ── test mapMetadataToArtifact — runId parsing ───────────────────────────────
function testRunIdParsing(): void {
  const run1 = mapMetadataToArtifact({
    id: 'a',
    name: 'seed-run-1035-mutant.bin',
    createdAt: '2026-05-28T22:00:00Z',
    sizeBytes: 45,
  });
  assert(run1.runId === 'run-1035', 'Should extract hyphenated run-XXXX id');

  const run2 = mapMetadataToArtifact({
    id: 'b',
    name: 'logs_run_9999.log',
    createdAt: '2026-05-28T22:00:00Z',
    sizeBytes: 900,
  });
  assert(run2.runId === 'run_9999', 'Should extract underscored run_XXXX id');

  const noRun = mapMetadataToArtifact({
    id: 'c',
    name: 'orphan_seed.bin',
    createdAt: '2026-05-28T22:00:00Z',
    sizeBytes: 12,
  });
  assert(noRun.runId === undefined, 'Should be undefined if not match');

  console.log('PASS: testRunIdParsing');
}

// ── test mapMetadataToArtifact — hash parsing ────────────────────────────────
function testHashParsing(): void {
  const hashMeta = {
    id: 'h1',
    name: 'seed_run-1002_a1b2c3d4e5f60718.bin',
    createdAt: '2026-05-28T22:00:00Z',
    sizeBytes: 4096,
  };
  const artifact = mapMetadataToArtifact(hashMeta);
  assert(artifact.content_hash === 'a1b2c3d4e5f60718', 'Should extract hex content hash');

  const shortName = { ...hashMeta, name: 'seed_run-1002.bin' };
  assert(mapMetadataToArtifact(shortName).content_hash === undefined, 'Should not extract hash if not hex match');

  console.log('PASS: testHashParsing');
}

// ── run all tests ────────────────────────────────────────────────────────────
function runAllTests(): void {
  console.log('Running Artifact Explorer component mapping tests...\n');
  try {
    testExtensionMapping();
    testRunIdParsing();
    testHashParsing();
    console.log('\n✅ All mapping tests passed successfully!');
  } catch (error) {
    console.error('\n❌ Mapping tests failed:', error);
    process.exit(1);
  }
}

if (typeof require !== 'undefined' && require.main === module) {
  runAllTests();
}

export {
  testExtensionMapping,
  testRunIdParsing,
  testHashParsing,
  runAllTests,
};
