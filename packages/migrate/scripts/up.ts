import { loadEnvConfig } from '@next/env';
import { up } from '../index';

async function main() {
  loadEnvConfig(process.cwd());

  const { env } = await import('@konobangu/env');

  await up(env);
}

main();
