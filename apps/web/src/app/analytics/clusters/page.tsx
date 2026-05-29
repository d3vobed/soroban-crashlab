'use client';

import Link from 'next/link';
import RunClusterVisualization from '../../add-run-cluster-visualization';
import { buildMockRuns } from '../../mockRuns';

export default function ClustersPage() {
  const runs = buildMockRuns();

  return (
    <div className="min-h-screen bg-white dark:bg-zinc-950">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Header */}
        <div className="mb-8">
          <Link
            href="/"
            className="inline-flex items-center gap-1.5 text-sm text-zinc-500 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-100 mb-4 transition"
          >
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 19l-7-7m0 0l7-7m-7 7h18" />
            </svg>
            Back to Dashboard
          </Link>
          <h1 className="text-3xl font-bold text-zinc-900 dark:text-zinc-100 mb-2">
            Run Cluster Visualization
          </h1>
          <p className="text-zinc-600 dark:text-zinc-400 max-w-3xl">
            Analyze fuzzing runs grouped by status, area, severity, performance characteristics,
            or failure signatures. Switch between grid, bubble, timeline, and metrics views to
            explore cluster distributions and identify patterns.
          </p>
        </div>

        {/* Cluster Visualization */}
        <RunClusterVisualization
          runs={runs}
          dataState="success"
          showTimeline={true}
          showMetrics={true}
        />
      </div>
    </div>
  );
}
