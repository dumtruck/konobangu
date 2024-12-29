import { database } from '@konobangu/database';

export const POST = async () => {
  const newPage = await database
    .insertInto('page')
    .values([
      {
        name: 'cron-temp',
      },
    ])
    .returning('id')
    .executeTakeFirstOrThrow();

  await database.deleteFrom('page').where('id', '=', newPage.id);

  return new Response('OK', { status: 200 });
};
