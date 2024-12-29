import { authMiddleware } from '@konobangu/auth/middleware';
import { env } from '@konobangu/env';
import { parseError } from '@konobangu/observability/error';
import { secure } from '@konobangu/security';
import {
  noseconeConfig,
  noseconeMiddleware,
} from '@konobangu/security/middleware';
import { type NextRequest, NextResponse } from 'next/server';

export const config = {
  // matcher tells Next.js which routes to run the middleware on. This runs the
  // middleware on all routes except for static assets and Posthog ingest
  matcher: ['/((?!_next/static|_next/image|ingest|favicon.ico).*)'],
};

const securityHeaders = noseconeMiddleware(noseconeConfig);

export const middleware = async (request: NextRequest) => {
  const beforeMiddleware = async (request: NextRequest) => {
    if (!env.ARCJET_KEY) {
      return securityHeaders();
    }

    try {
      await secure(
        [
          // See https://docs.arcjet.com/bot-protection/identifying-bots
          'CATEGORY:SEARCH_ENGINE', // Allow search engines
          'CATEGORY:PREVIEW', // Allow preview links to show OG images
          'CATEGORY:MONITOR', // Allow uptime monitoring services
        ],
        request
      );

      return securityHeaders();
    } catch (error) {
      const message = parseError(error);

      return NextResponse.json({ error: message }, { status: 403 });
    }
  };

  const response = await beforeMiddleware(request);

  return authMiddleware(response as any);
};
