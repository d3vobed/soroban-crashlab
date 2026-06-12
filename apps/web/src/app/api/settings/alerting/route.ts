import { NextRequest, NextResponse } from 'next/server';
import {
  createDefaultAlertingSettingsSnapshot,
  readAlertingSettingsSnapshot,
  serializeAlertingSettingsSnapshot,
  validateAlertingSettingsSnapshot,
  type AlertingSettingsSnapshot,
} from '@/app/alerting-settings-page-utils';

// In-memory store (persists for the lifetime of the process)
let store: AlertingSettingsSnapshot | null = null;

function getSnapshot(): AlertingSettingsSnapshot {
  if (!store) {
    store = createDefaultAlertingSettingsSnapshot();
  }
  return store;
}

/**
 * GET /api/settings/alerting
 * Returns the current alerting settings snapshot.
 */
export async function GET() {
  return NextResponse.json(getSnapshot());
}

/**
 * PUT /api/settings/alerting
 * Replaces the alerting settings snapshot. Body: AlertingSettingsSnapshot JSON.
 */
export async function PUT(request: NextRequest) {
  let body: string;
  try {
    body = await request.text();
  } catch {
    return NextResponse.json({ error: 'Failed to read request body' }, { status: 400 });
  }

  const result = readAlertingSettingsSnapshot(body);
  if (result.status === 'error' || !result.snapshot) {
    return NextResponse.json(
      { error: result.error ?? 'Invalid alerting settings payload' },
      { status: 422 },
    );
  }

  const validationError = validateAlertingSettingsSnapshot(result.snapshot);
  if (validationError) {
    return NextResponse.json({ error: validationError }, { status: 422 });
  }

  store = {
    ...result.snapshot,
    lastUpdated: new Date().toISOString(),
  };

  return NextResponse.json(store);
}
