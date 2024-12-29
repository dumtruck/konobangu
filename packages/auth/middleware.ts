import { env } from '@konobangu/env';
import { createAuthClient } from 'better-auth/client';
import { type NextRequest, NextResponse } from 'next/server';

const { getSession } = createAuthClient({
  baseURL: env.NEXT_PUBLIC_APP_URL
});

const isProtectedRoute = (request: NextRequest) => {
  return request?.url?.startsWith?.('/dashboard'); // change this to your protected route
};

export const authMiddleware = async (request: NextRequest) => {
  const session = await getSession(request as any);

  if (isProtectedRoute(request) && !session) {
    return NextResponse.redirect(new URL('/sign-in', request.url));
  }

  return NextResponse.next();
};