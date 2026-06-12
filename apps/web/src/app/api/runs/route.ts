import { NextResponse } from 'next/server';
import { buildMockRuns } from '@/app/mockRuns';

export async function GET() {
    const runs = buildMockRuns();
    return NextResponse.json({ runs, total: runs.length });
}
