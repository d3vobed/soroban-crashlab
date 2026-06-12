import * as assert from 'node:assert/strict';
import { simulateSeedReplay } from './replay';
import {
  createReplayPlaceholderRun,
  getReplayButtonLabel,
  type ReplayActionData,
  type ReplayButtonStatus,
} from './replay-ui-utils';

function testGetReplayButtonLabel() {
  assert.equal(getReplayButtonLabel('idle'), 'Replay');
  assert.equal(getReplayButtonLabel('loading'), 'Replaying...');
  assert.equal(getReplayButtonLabel('success'), 'Replay queued');
  assert.equal(getReplayButtonLabel('error'), 'Retry replay');

  const statuses: ReplayButtonStatus[] = ['idle', 'loading', 'success', 'error'];
  const labels = statuses.map((status) => getReplayButtonLabel(status));
  assert.deepEqual(labels, ['Replay', 'Replaying...', 'Replay queued', 'Retry replay']);
}

function testCreateReplayPlaceholderRun() {
  const data: ReplayActionData = { id: 'replay-run-1', status: 'running' };
  const run = createReplayPlaceholderRun(data);

  assert.equal(run.id, 'replay-run-1');
  assert.equal(run.status, 'running');
  assert.equal(run.area, 'state');
  assert.equal(run.severity, 'medium');

  const defaultRun = createReplayPlaceholderRun({ id: 'replay-run-2', status: 'running' });
  assert.equal(defaultRun.duration, 0);
  assert.equal(defaultRun.seedCount, 0);
  assert.equal(defaultRun.cpuInstructions, 0);
  assert.equal(defaultRun.memoryBytes, 0);
  assert.equal(defaultRun.minResourceFee, 0);
  assert.equal(defaultRun.crashDetail, null);
}

async function testReplayServiceMapping() {
  const originalFetch = globalThis.fetch;
  const fetchMock = async (input: RequestInfo | URL, init?: RequestInit) => {
    assert.ok(String(input).includes('/api/runs/run-42/replay'));
    assert.equal(init?.method, 'POST');

    return new Response(
      JSON.stringify({
        ok: true,
        runId: 'run-42',
        newRunId: 'replay-run-42-abc12345',
        command: 'cargo',
        args: ['run'],
        stdout: '',
        stderr: '',
        exitCode: 0,
        bundleJson: '{}',
      }),
      {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      },
    );
  };

  globalThis.fetch = fetchMock as typeof globalThis.fetch;

  try {
    const replayResult = await simulateSeedReplay('run-42');
    const run = createReplayPlaceholderRun({
      id: replayResult.newRunId,
      status: 'running',
    });

    assert.ok(run.id.startsWith('replay-run-42-'));
    assert.equal(run.status, 'running');
    assert.equal(run.crashDetail, null);
    assert.equal(run.area, 'state');
  } finally {
    globalThis.fetch = originalFetch;
  }
}

async function main() {
  testGetReplayButtonLabel();
  testCreateReplayPlaceholderRun();
  await testReplayServiceMapping();
  console.log('replay-ui-utils.test.ts: all assertions passed');
}

void main();