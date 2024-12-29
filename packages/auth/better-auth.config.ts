import type { PostgresPool } from '@konobangu/database/builder';
import { betterAuth } from 'better-auth';
import { nextCookies } from 'better-auth/next-js';
import { organization } from 'better-auth/plugins';

export interface BuildAuthProps {
  pool: PostgresPool;
  baseURL: string;
}

export const buildAuth = ({ pool, baseURL }: BuildAuthProps) =>
  betterAuth({
    database: pool,
    plugins: [nextCookies(), organization()],
    baseURL,
  });

export { getMigrations as getAuthMigrations } from 'better-auth/db';
