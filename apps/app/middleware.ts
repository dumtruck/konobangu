import { authMiddleware } from '@konobangu/auth/middleware';
import {
  noseconeConfig,
  noseconeMiddleware,
} from '@konobangu/security/middleware';
import { NextRequest } from 'next/server';

const securityHeaders = noseconeMiddleware(noseconeConfig);

export async function middleware (_request: NextRequest) {
  const response = await securityHeaders();
  return authMiddleware(response as any);
}

export const config = {
  matcher: [
    // Skip Next.js internals and all static files, unless found in search params
    '/((?!_next|[^?]*\\.(?:html?|css|js(?!on)|jpe?g|webp|png|gif|svg|ttf|woff2?|ico|csv|docx?|xlsx?|zip|webmanifest)).*)',
    // Always run for API routes
    '/(api|trpc)(.*)',
  ],
};
