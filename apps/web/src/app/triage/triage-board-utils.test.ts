import * as assert from 'node:assert/strict';
import {
  TRIAGE_COLUMNS,
  getColumnRuns,
  getColumnCounts,
  getRunsWithIssues,
  getIssueCounts,
} from './triage-board-utils';
import type { FuzzingRun } from '../types';
import type { RunIssueLink } from '../types';

const base: FuzzingRun[] = [
  { id: 'r1', status: 'failed',    area: 'auth',   severity: 'high',   duration: 1000, seedCount: 100, cpuInstructions: 0, memoryBytes: 0, minResourceFee: 0, crashDetail: null },
  { id: 'r2', status: 'running',   area: 'state',  severity: 'low',    duration: 2000, seedCount: 200, cpuInstructions: 0, memoryBytes: 0, minResourceFee: 0, crashDetail: null },
  { id: 'r3', status: 'cancelled', area: 'budget', severity: 'medium', duration: 500,  seedCount: 50,  cpuInstructions: 0, memoryBytes: 0, minResourceFee: 0, crashDetail: null },
  { id: 'r4', status: 'failed',    area: 'xdr',    severity: 'critical', duration: 3000, seedCount: 300, cpuInstructions: 0, memoryBytes: 0, minResourceFee: 0, crashDetail: null },
  { id: 'r5', status: 'completed', area: 'auth',   severity: 'low',    duration: 4000, seedCount: 400, cpuInstructions: 0, memoryBytes: 0, minResourceFee: 0, crashDetail: null },
];

// ---------------------------------------------------------------------------
// getColumnRuns
// ---------------------------------------------------------------------------

// getColumnRuns – failed column
assert.deepEqual(
  getColumnRuns(base, TRIAGE_COLUMNS[0]).map((r) => r.id),
  ['r1', 'r4'],
);

// getColumnRuns – active column
assert.deepEqual(
  getColumnRuns(base, TRIAGE_COLUMNS[1]).map((r) => r.id),
  ['r2'],
);

// getColumnRuns – cancelled column
assert.deepEqual(
  getColumnRuns(base, TRIAGE_COLUMNS[2]).map((r) => r.id),
  ['r3'],
);

// ---------------------------------------------------------------------------
// getColumnCounts
// ---------------------------------------------------------------------------

// getColumnCounts
assert.deepEqual(getColumnCounts(base), { failed: 2, active: 1, cancelled: 1 });

// edge case: empty runs
assert.deepEqual(getColumnCounts([]), { failed: 0, active: 0, cancelled: 0 });

// ---------------------------------------------------------------------------
// getRunsWithIssues
// ---------------------------------------------------------------------------

const issueA: RunIssueLink = { label: '#10 Auth bug', href: 'https://github.com/test/10' };
const issueB: RunIssueLink = { label: '#20 XDR crash', href: 'https://github.com/test/20' };
const issueC: RunIssueLink = { label: '#30 Follow-up', href: 'https://github.com/test/30' };

// Merges issues from the map into matching runs
{
  const issueMap = new Map<string, RunIssueLink[]>();
  issueMap.set('r1', [issueA, issueC]);
  issueMap.set('r4', [issueB]);

  const enriched = getRunsWithIssues(base, issueMap);
  assert.equal(enriched.length, base.length, 'enriched array preserves length');
  assert.deepEqual(enriched[0].associatedIssues, [issueA, issueC], 'r1 gets issues from map');
  assert.deepEqual(enriched[3].associatedIssues, [issueB], 'r4 gets issues from map');
}

// Runs not in the map keep their existing associatedIssues
{
  const runsWithExisting: FuzzingRun[] = [
    { ...base[1], associatedIssues: [issueA] },
  ];
  const emptyMap = new Map<string, RunIssueLink[]>();
  const result = getRunsWithIssues(runsWithExisting, emptyMap);
  assert.deepEqual(result[0].associatedIssues, [issueA], 'existing issues preserved when not in map');
}

// Empty map returns runs unchanged
{
  const emptyMap = new Map<string, RunIssueLink[]>();
  const result = getRunsWithIssues(base, emptyMap);
  assert.deepEqual(
    result.map((r) => r.id),
    base.map((r) => r.id),
    'empty map preserves all runs',
  );
  result.forEach((r, i) => {
    assert.equal(r.associatedIssues, base[i].associatedIssues, `run ${r.id} issues unchanged`);
  });
}

// ---------------------------------------------------------------------------
// getIssueCounts
// ---------------------------------------------------------------------------

// Counts total issues across all runs
{
  const runsWithIssues: FuzzingRun[] = [
    { ...base[0], associatedIssues: [issueA, issueC] },
    { ...base[1], associatedIssues: [] },
    { ...base[3], associatedIssues: [issueB] },
  ];
  assert.equal(getIssueCounts(runsWithIssues), 3, 'total issue count is 3');
}

// Returns 0 when no runs have issues
assert.equal(getIssueCounts(base), 0, 'base runs have 0 issues');

// Returns 0 for empty array
assert.equal(getIssueCounts([]), 0, 'empty array has 0 issues');

console.log('triage-board-utils.test.ts: all assertions passed');
