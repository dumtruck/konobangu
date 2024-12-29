import {
  buildAuth,
  getAuthMigrations,
} from '@konobangu/auth/better-auth.config';
import { buildDatabase, buildMigrator } from '@konobangu/database/builder';

export const up = async (envVars: Record<string, any>) => {
  const { db, pool } = buildDatabase(envVars.DATABASE_URL);

  const authConfig = buildAuth({ pool, baseURL: envVars.NEXT_PUBLIC_APP_URL });

  const { toBeAdded, toBeCreated, runMigrations } = await getAuthMigrations(
    authConfig.options
  );

  if (!toBeAdded.length && !toBeCreated.length) {
    // biome-ignore lint/suspicious/noConsole: <explanation>
    console.info('No auth migrations needed.');
    process.exit(0);
  }

  // biome-ignore lint/suspicious/noConsole: <explanation>
  console.info('Migrating auth...');

  await runMigrations();

  // biome-ignore lint/suspicious/noConsole: <explanation>
  console.info('Auth migration was completed successfully!');

  // biome-ignore lint/suspicious/noConsole: <explanation>
  console.info('Migrating webui...');

  const { error, results } = await buildMigrator(db).migrateUp();

  for (const it of results || []) {
    if (it.status === 'Success') {
      // biome-ignore lint/suspicious/noConsole: <explanation>
      // biome-ignore lint/suspicious/noConsoleLog: <explanation>
      console.log(
        `migration up "${it.migrationName}" was executed successfully`
      );
    } else if (it.status === 'Error') {
      // biome-ignore lint/suspicious/noConsole: <explanation>
      console.error(`failed to execute migration up "${it.migrationName}"`);
    }
  }

  if (error) {
    // biome-ignore lint/suspicious/noConsole: <explanation>
    console.error('failed to migrate up');
    // biome-ignore lint/suspicious/noConsole: <explanation>
    console.error(error);

    process.exit(1);
  }
};

export const down = async (envVars: Record<string, any>) => {
  const { db } = buildDatabase(envVars.DATABASE_URL);

  const { error, results } = await buildMigrator(db).migrateDown();

  for (const it of results || []) {
    if (it.status === 'Success') {
      // biome-ignore lint/suspicious/noConsoleLog: <explanation>
      // biome-ignore lint/suspicious/noConsole: <explanation>
      console.log(
        `migration down "${it.migrationName}" was executed successfully`
      );
    } else if (it.status === 'Error') {
      // biome-ignore lint/suspicious/noConsole: <explanation>
      console.error(`failed to execute migration down "${it.migrationName}"`);
    }
  }

  if (error) {
    // biome-ignore lint/suspicious/noConsole: <explanation>
    console.error('failed to migrate down');
    // biome-ignore lint/suspicious/noConsole: <explanation>
    console.error(error);
    process.exit(1);
  }
};
