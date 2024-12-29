import { promises as fs } from 'node:fs';
import path from 'node:path';
import {
  FileMigrationProvider,
  Kysely,
  Migrator,
  PostgresDialect,
  // biome-ignore lint/nursery/noExportedImports: <explanation>
  type PostgresPool,
} from 'kysely';
import { Pool } from 'pg';
import type { Database } from './schema/database';

export type { PostgresPool };

export const buildPool = (connectionString: string): PostgresPool => {
  return new Pool({
    connectionString,
    max: 10,
  });
};

export const buildDatabase = (connectionString: string) => {
  const pool = buildPool(connectionString);

  const dialect = new PostgresDialect({
    pool,
  });

  return {
    db: new Kysely<Database>({
      dialect,
    }),
    pool,
    dialect,
  };
};

export const buildMigrator = (db: Kysely<Database>) =>
  new Migrator({
    db,
    provider: new FileMigrationProvider({
      fs,
      path,
      migrationFolder: path.resolve(__dirname, 'migrations'),
    }),
  });
