import type { Kysely } from 'kysely';
import type { Database } from 'schema/database';

export async function up(db: Kysely<Database>): Promise<void> {
  await db.schema
    .createTable('page')
    .ifNotExists()
    .addColumn('id', 'serial', (cb) => cb.primaryKey())
    .addColumn('name', 'text', (cb) => cb.notNull())
    .execute();
}

export async function down(db: Kysely<Database>): Promise<void> {
  await db.schema.dropTable('page').ifExists().execute();
}
