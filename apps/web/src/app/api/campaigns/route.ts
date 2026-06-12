import { NextRequest, NextResponse } from 'next/server';

export async function POST(request: NextRequest) {
    const body = await request.json() as Record<string, unknown>;

    const campaign = {
        id: `campaign-${Date.now()}`,
        status: 'queued',
        createdAt: new Date().toISOString(),
        ...body,
    };

    return NextResponse.json({ campaign }, { status: 201 });
}
