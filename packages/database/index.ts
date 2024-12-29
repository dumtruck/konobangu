import { env } from '@konobangu/env';
import { buildDatabase } from './builder';

// Database interface is passed to Kysely's constructor, and from now on, Kysely
// knows your database structure.
// Dialect is passed to Kysely's constructor, and from now on, Kysely knows how
// to communicate with your database.
const { db, dialect, pool } = buildDatabase(env.DATABASE_URL);

export const database = db;

export { db, dialect, pool };
