import { defineConfig } from '@rsbuild/core';
import { pluginBabel } from '@rsbuild/plugin-babel';
import { pluginSolid } from '@rsbuild/plugin-solid';
import { TanStackRouterRspack } from '@tanstack/router-plugin/rspack';

export default defineConfig({
  html: {
    favicon: './public/assets/favicon.ico',
  },
  plugins: [
    pluginBabel({
      include: /\.(?:jsx|tsx)$/,
    }),
    pluginSolid(),
  ],
  tools: {
    rspack: {
      plugins: [
        TanStackRouterRspack({ target: 'solid', autoCodeSplitting: true }),
      ],
    },
  },
  source: {
    entry: {
      index: './src/main.tsx',
    },
    define: {
      'process.env.AUTH_TYPE': JSON.stringify(process.env.AUTH_TYPE),
      'process.env.OIDC_CLIENT_ID': JSON.stringify(process.env.OIDC_CLIENT_ID),
      'process.env.OIDC_CLIENT_SECRET': JSON.stringify(
        process.env.OIDC_CLIENT_SECRET
      ),
      'process.env.OIDC_ISSUER': JSON.stringify(process.env.OIDC_ISSUER),
      'process.env.OIDC_AUDIENCE': JSON.stringify(process.env.OIDC_AUDIENCE),
      'process.env.OIDC_EXTRA_SCOPES': JSON.stringify(
        process.env.OIDC_EXTRA_SCOPES
      ),
    },
  },
  server: {
    host: '0.0.0.0',
    port: 5000,
  },
});
