import { loadEnvConfig } from '@next/env';
import { down } from '../index';

async function main() {
  loadEnvConfig(process.cwd());

  const { env } = await import('@konobangu/env');

  await down(env);
}

main();
