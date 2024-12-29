import type { ReadonlyHeaders } from 'next/dist/server/web/spec-extension/adapters/headers';
import { headers } from 'next/headers';
import { buildAuth } from './better-auth.config';
import { env } from '@konobangu/env';
import { pool } from '@konobangu/database';

export const auth = buildAuth({
  pool,
  baseURL: env.NEXT_PUBLIC_APP_URL,
});

export const getSessionFromHeaders = async () => {
  const h = await headers();

  const session = await auth.api.getSession({
    headers: h,
  });

  return {
    ...session,
    headers: h,
    userId: session?.user?.id,
    orgId: session?.session?.activeOrganizationId ?? undefined,
  };
};

export const getFullOrganizationFromSession = async (session: {
  orgId?: string;
  headers: ReadonlyHeaders;
}) => {
  const orgId = session?.orgId;

  const fullOrganization = await auth.api.getFullOrganization({
    headers: session.headers,
    query: { organizationId: orgId ?? undefined },
  });

  return {
    fullOrganization,
  };
};
