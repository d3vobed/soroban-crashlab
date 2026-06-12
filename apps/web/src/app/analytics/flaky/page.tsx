'use client';

import Link from 'next/link';
import { buildMockRuns } from '../../mockRuns';
import { FuzzingRun, RunArea, RunSeverity } from '../../types';

interface FlakySignature {
  signature: string;
  category: string;
  area: RunArea;
  severity: RunSeverity;
  occurrences: number;
  firstSeen: string;
  lastSeen: string;
  seedSpread: number;
  flakyScore: number;
  sampleRunId: string;
}

interface FlakySummary {
  totalRuns: number;
  failedRuns: number;
  flakyRuns: number;
  flakyRate: number;
  averageScore: number;
  signatures: FlakySignature[];
}

const severityWeight: Record<RunSeverity, number> = {
  low: 0.7,
  medium: 0.85,
  high: 1,
  critical: 1.15,
};

export default function FlakyAnalyticsPage() {
  const runs = buildMockRuns();
  const summary = buildFlakySummary(runs);
  const leadingSignature = summary.signatures[0];

  return (
    <div className="min-h-screen bg-white dark:bg-zinc-950">
      <div className="mx-auto max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
        <div className="mb-8">
          <Link
            href="/"
            className="mb-4 inline-flex items-center gap-1.5 text-sm text-zinc-500 transition hover:text-zinc-900 dark:text-zinc-400 dark:hover:text-zinc-100"
          >
            <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M10 19l-7-7m0 0l7-7m-7 7h18"
              />
            </svg>
            Back to Dashboard
          </Link>
          <h1 className="mb-2 text-3xl font-bold text-zinc-900 dark:text-zinc-100">
            Flaky Rate Analytics
          </h1>
          <p className="max-w-3xl text-zinc-600 dark:text-zinc-400">
            Track unstable crash signatures across recent fuzzing runs. The flaky rate
            counts failed runs that belong to recurring signatures with varying seed
            counts, which helps separate persistent defects from noise.
          </p>
        </div>

        <section className="mb-6 grid gap-3 md:grid-cols-4">
          <MetricPanel
            label="Flaky rate"
            value={`${summary.flakyRate.toFixed(1)}%`}
            detail={`${summary.flakyRuns} of ${summary.totalRuns} runs`}
            tone={summary.flakyRate >= 25 ? 'red' : summary.flakyRate >= 10 ? 'amber' : 'green'}
          />
          <MetricPanel
            label="Recurring signatures"
            value={String(summary.signatures.length)}
            detail={`${summary.failedRuns} failed runs analyzed`}
            tone="zinc"
          />
          <MetricPanel
            label="Average score"
            value={summary.averageScore.toFixed(0)}
            detail="Severity and recurrence weighted"
            tone={summary.averageScore >= 70 ? 'red' : 'amber'}
          />
          <MetricPanel
            label="Top signal"
            value={leadingSignature ? `${leadingSignature.flakyScore.toFixed(0)}` : '0'}
            detail={leadingSignature?.category ?? 'No recurring failures'}
            tone="blue"
          />
        </section>

        <div className="grid gap-6 xl:grid-cols-[minmax(0,1fr)_360px]">
          <section className="overflow-hidden rounded-2xl border border-zinc-200 bg-zinc-50/70 dark:border-zinc-800 dark:bg-zinc-900/60">
            <div className="border-b border-zinc-200 bg-white px-5 py-4 dark:border-zinc-800 dark:bg-zinc-950">
              <h2 className="text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                Flaky signature breakdown
              </h2>
              <p className="mt-1 text-sm text-zinc-600 dark:text-zinc-400">
                Ranked by recurrence, seed spread, and severity impact.
              </p>
            </div>

            <div className="overflow-x-auto">
              <table className="min-w-full divide-y divide-zinc-200 text-sm dark:divide-zinc-800">
                <thead className="bg-zinc-100/80 text-left text-xs font-semibold uppercase tracking-wider text-zinc-500 dark:bg-zinc-900 dark:text-zinc-400">
                  <tr>
                    <th className="px-5 py-3">Signature</th>
                    <th className="px-5 py-3">Area</th>
                    <th className="px-5 py-3">Occurrences</th>
                    <th className="px-5 py-3">Seed spread</th>
                    <th className="px-5 py-3">Score</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-zinc-200 bg-white dark:divide-zinc-800 dark:bg-zinc-950">
                  {summary.signatures.map((signature) => (
                    <tr key={signature.signature}>
                      <td className="max-w-sm px-5 py-4">
                        <div className="font-medium text-zinc-950 dark:text-zinc-50">
                          {signature.category}
                        </div>
                        <div className="mt-1 truncate font-mono text-xs text-zinc-500 dark:text-zinc-400">
                          {signature.signature}
                        </div>
                      </td>
                      <td className="px-5 py-4">
                        <span className="rounded-full bg-zinc-100 px-2.5 py-1 text-xs font-medium text-zinc-700 dark:bg-zinc-800 dark:text-zinc-200">
                          {signature.area}
                        </span>
                      </td>
                      <td className="px-5 py-4 text-zinc-700 dark:text-zinc-300">
                        {signature.occurrences}
                      </td>
                      <td className="px-5 py-4 text-zinc-700 dark:text-zinc-300">
                        {signature.seedSpread.toLocaleString()}
                      </td>
                      <td className="px-5 py-4">
                        <div className="h-2 w-28 overflow-hidden rounded-full bg-zinc-200 dark:bg-zinc-800">
                          <div
                            className="h-full rounded-full bg-orange-500"
                            style={{ width: `${Math.min(signature.flakyScore, 100)}%` }}
                          />
                        </div>
                        <div className="mt-1 text-xs font-medium text-zinc-600 dark:text-zinc-400">
                          {signature.flakyScore.toFixed(0)}
                        </div>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </section>

          <aside className="rounded-2xl border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/70">
            <p className="text-xs font-semibold uppercase tracking-[0.2em] text-zinc-500">
              Current risk
            </p>
            <h2 className="mt-2 text-2xl font-semibold text-zinc-950 dark:text-zinc-50">
              {riskLabel(summary.flakyRate)}
            </h2>
            <p className="mt-2 text-sm text-zinc-600 dark:text-zinc-400">
              {riskCopy(summary.flakyRate)}
            </p>

            <div className="mt-6 space-y-4">
              {summary.signatures.slice(0, 3).map((signature) => (
                <div
                  key={signature.signature}
                  className="rounded-xl border border-zinc-200 bg-white p-4 dark:border-zinc-800 dark:bg-zinc-950"
                >
                  <div className="flex items-start justify-between gap-3">
                    <div>
                      <div className="font-medium text-zinc-950 dark:text-zinc-50">
                        {signature.category}
                      </div>
                      <div className="mt-1 text-xs text-zinc-500 dark:text-zinc-400">
                        {formatShortDate(signature.firstSeen)} to{' '}
                        {formatShortDate(signature.lastSeen)}
                      </div>
                    </div>
                    <span className="rounded-full bg-orange-100 px-2.5 py-1 text-xs font-semibold text-orange-800 dark:bg-orange-950 dark:text-orange-200">
                      {signature.flakyScore.toFixed(0)}
                    </span>
                  </div>
                  <div className="mt-3 text-xs text-zinc-600 dark:text-zinc-400">
                    Sample: {signature.sampleRunId}
                  </div>
                </div>
              ))}
            </div>
          </aside>
        </div>
      </div>
    </div>
  );
}

function buildFlakySummary(runs: FuzzingRun[]): FlakySummary {
  const failedRuns = runs.filter((run) => run.status === 'failed' && run.crashDetail);
  const bySignature = new Map<string, FuzzingRun[]>();

  for (const run of failedRuns) {
    const signature = run.crashDetail?.signature;
    if (!signature) {
      continue;
    }

    bySignature.set(signature, [...(bySignature.get(signature) ?? []), run]);
  }

  const signatures = Array.from(bySignature.entries())
    .map(([signature, signatureRuns]) => toFlakySignature(signature, signatureRuns))
    .filter((signature): signature is FlakySignature => signature !== null)
    .sort((a, b) => b.flakyScore - a.flakyScore);

  const flakyRuns = signatures.reduce((total, signature) => total + signature.occurrences, 0);
  const averageScore =
    signatures.length === 0
      ? 0
      : signatures.reduce((total, signature) => total + signature.flakyScore, 0) /
        signatures.length;

  return {
    totalRuns: runs.length,
    failedRuns: failedRuns.length,
    flakyRuns,
    flakyRate: runs.length === 0 ? 0 : (flakyRuns / runs.length) * 100,
    averageScore,
    signatures,
  };
}

function toFlakySignature(signature: string, runs: FuzzingRun[]): FlakySignature | null {
  if (runs.length < 2) {
    return null;
  }

  const sortedRuns = [...runs].sort(
    (a, b) => runTimestamp(a).getTime() - runTimestamp(b).getTime(),
  );
  const firstRun = sortedRuns[0];
  const lastRun = sortedRuns[sortedRuns.length - 1];
  const seedCounts = sortedRuns.map((run) => run.seedCount);
  const seedSpread = Math.max(...seedCounts) - Math.min(...seedCounts);
  const recurrenceScore = Math.min(sortedRuns.length * 18, 60);
  const spreadScore = Math.min(seedSpread / 750, 25);
  const weightedScore =
    (recurrenceScore + spreadScore + 15) * severityWeight[lastRun.severity];

  return {
    signature,
    category: lastRun.crashDetail?.failureCategory ?? 'Unknown',
    area: lastRun.area,
    severity: lastRun.severity,
    occurrences: sortedRuns.length,
    firstSeen: runTimestamp(firstRun).toISOString(),
    lastSeen: runTimestamp(lastRun).toISOString(),
    seedSpread,
    flakyScore: Math.min(weightedScore, 100),
    sampleRunId: lastRun.id,
  };
}

function MetricPanel({
  label,
  value,
  detail,
  tone,
}: {
  label: string;
  value: string;
  detail: string;
  tone: 'amber' | 'blue' | 'green' | 'red' | 'zinc';
}) {
  const toneClass = {
    amber: 'border-amber-200 bg-amber-50 text-amber-950 dark:border-amber-900/60 dark:bg-amber-950/20 dark:text-amber-100',
    blue: 'border-blue-200 bg-blue-50 text-blue-950 dark:border-blue-900/60 dark:bg-blue-950/20 dark:text-blue-100',
    green: 'border-emerald-200 bg-emerald-50 text-emerald-950 dark:border-emerald-900/60 dark:bg-emerald-950/20 dark:text-emerald-100',
    red: 'border-red-200 bg-red-50 text-red-950 dark:border-red-900/60 dark:bg-red-950/20 dark:text-red-100',
    zinc: 'border-zinc-200 bg-zinc-50 text-zinc-950 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-100',
  }[tone];

  return (
    <div className={`rounded-2xl border p-4 ${toneClass}`}>
      <div className="text-sm font-medium opacity-80">{label}</div>
      <div className="mt-2 text-3xl font-bold">{value}</div>
      <div className="mt-1 text-sm opacity-75">{detail}</div>
    </div>
  );
}

function runTimestamp(run: FuzzingRun): Date {
  return new Date(run.finishedAt ?? run.startedAt ?? run.queuedAt ?? 0);
}

function formatShortDate(value: string): string {
  return new Intl.DateTimeFormat('en', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(new Date(value));
}

function riskLabel(flakyRate: number): string {
  if (flakyRate >= 25) {
    return 'Elevated flake pressure';
  }

  if (flakyRate >= 10) {
    return 'Watch closely';
  }

  return 'Stable trend';
}

function riskCopy(flakyRate: number): string {
  if (flakyRate >= 25) {
    return 'Recurring crash signatures are taking a meaningful share of recent runs. Prioritize the top signature before broadening the campaign.';
  }

  if (flakyRate >= 10) {
    return 'A few signatures are recurring across different seed counts. Keep them visible during triage and replay validation.';
  }

  return 'Recent runs show low recurrence in failed signatures. Continue monitoring as new campaigns land.';
}
