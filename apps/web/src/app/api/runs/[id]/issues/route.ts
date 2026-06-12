import { NextRequest, NextResponse } from 'next/server';
import { buildMockRuns } from '@/app/mockRuns';
import type { RunIssueLink } from '@/app/types';

// In-memory store keyed by run ID (persists for the lifetime of the process)
const issueStore = new Map<string, RunIssueLink[]>();

function getIssues(id: string): RunIssueLink[] {
  if (issueStore.has(id)) {
    return issueStore.get(id)!;
  }
  const run = buildMockRuns().find((r) => r.id === id);
  const initial = run?.associatedIssues ?? [];
  issueStore.set(id, [...initial]);
  return issueStore.get(id)!;
}

/**
 * GET /api/runs/[id]/issues
 * Returns the current issue links for a run.
 */
export async function GET(
  _request: NextRequest,
  { params }: { params: Promise<{ id: string }> },
) {
  const { id } = await params;
  const run = buildMockRuns().find((r) => r.id === id);
  if (!run) {
    return NextResponse.json({ error: 'Run not found' }, { status: 404 });
  }
  return NextResponse.json({ runId: id, issues: getIssues(id) });
}

/**
 * POST /api/runs/[id]/issues
 * Appends a new issue link. Body: { label: string; href: string }
 */
export async function POST(
  request: NextRequest,
  { params }: { params: Promise<{ id: string }> },
) {
  const { id } = await params;
  const run = buildMockRuns().find((r) => r.id === id);
  if (!run) {
    return NextResponse.json({ error: 'Run not found' }, { status: 404 });
  }

  let body: unknown;
  try {
    body = await request.json();
  } catch {
    return NextResponse.json({ error: 'Invalid JSON body' }, { status: 400 });
  }

  const { label, href } = (body ?? {}) as Record<string, unknown>;

  if (typeof label !== 'string' || !label.trim()) {
    return NextResponse.json({ error: 'label is required' }, { status: 400 });
  }

  if (typeof href !== 'string') {
    return NextResponse.json({ error: 'href is required' }, { status: 400 });
  }

  try {
    const parsed = new URL(href);
    if (!['http:', 'https:'].includes(parsed.protocol)) {
      return NextResponse.json({ error: 'href must use http or https' }, { status: 400 });
    }
  } catch {
    return NextResponse.json({ error: 'href is not a valid URL' }, { status: 400 });
  }

  const issues = getIssues(id);

  if (issues.some((link) => link.href === href)) {
    return NextResponse.json({ error: 'Issue link already exists' }, { status: 409 });
  }

  const newLink: RunIssueLink = { label: label.trim(), href };
  issues.push(newLink);

  return NextResponse.json({ runId: id, issues }, { status: 201 });
}

/**
 * DELETE /api/runs/[id]/issues
 * Removes an issue link by href. Body: { href: string }
 */
export async function DELETE(
  request: NextRequest,
  { params }: { params: Promise<{ id: string }> },
) {
  const { id } = await params;
  const run = buildMockRuns().find((r) => r.id === id);
  if (!run) {
    return NextResponse.json({ error: 'Run not found' }, { status: 404 });
  }

  let body: unknown;
  try {
    body = await request.json();
  } catch {
    return NextResponse.json({ error: 'Invalid JSON body' }, { status: 400 });
  }

  const { href } = (body ?? {}) as Record<string, unknown>;
  if (typeof href !== 'string') {
    return NextResponse.json({ error: 'href is required' }, { status: 400 });
  }

  const issues = getIssues(id);
  const index = issues.findIndex((link) => link.href === href);
  if (index === -1) {
    return NextResponse.json({ error: 'Issue link not found' }, { status: 404 });
  }

  issues.splice(index, 1);
  return NextResponse.json({ runId: id, issues });
}
