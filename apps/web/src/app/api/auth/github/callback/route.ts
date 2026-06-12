import { NextRequest, NextResponse } from 'next/server';
import { logger } from '@/lib/logger';

/**
 * GET /api/auth/github/callback
 * GitHub OAuth 2.0 callback route.
 *
 * This route handles the redirection from GitHub after a user has authorized the application.
 * It expects 'code' and 'state' as query parameters.
 */
export async function GET(request: NextRequest) {
  const { searchParams } = new URL(request.url);
  const code = searchParams.get('code');
  const state = searchParams.get('state');

  // 1. Validate the presence of required parameters
  if (!code) {
    return NextResponse.json(
      { error: 'Missing "code" parameter from GitHub callback.' },
      { status: 400 }
    );
  }

  if (!state) {
    logger.warn('GET /api/auth/github/callback: missing state parameter');
  }

  try {
    // 2. Simulate exchange of 'code' for an access token
    logger.info('GET /api/auth/github/callback: exchanging code for access token', { code });

    // Simulate API latency
    await new Promise((resolve) => setTimeout(resolve, 500));

    // 3. Simulate fetching user profile with the access token
    const mockUser = {
      id: 123456,
      login: 'octocat',
      email: 'octocat@github.com',
      name: 'The Octocat',
    };

    logger.info('GET /api/auth/github/callback: authenticated user', { login: mockUser.login });

    // 4. Redirect user back to the dashboard with a success indicator
    return NextResponse.redirect(new URL('/', request.url), {
      status: 302,
    });
  } catch (error) {
    logger.error('GET /api/auth/github/callback failed', { error });
    return NextResponse.json(
      { error: 'An internal error occurred while processing the GitHub authentication.' },
      { status: 500 }
    );
  }
}
