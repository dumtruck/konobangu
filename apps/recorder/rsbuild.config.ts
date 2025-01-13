import { defineConfig } from '@rsbuild/core';
import { pluginReact } from '@rsbuild/plugin-react';
import { TanStackRouterRspack } from '@tanstack/router-plugin/rspack';

export default defineConfig({
  plugins: [pluginReact()],
  html: {
    favicon: './public/assets/favicon.ico',
    tags: [
      {
        tag: 'script',
        attrs: { src: 'https://cdn.tailwindcss.com' },
      },
    ],
  },
  tools: {
    rspack: {
      plugins: [TanStackRouterRspack()],
    },
  },
  source: {
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
  dev: {
    client: {
      path: '/api/playground/rsbuild-hmr',
    },
    setupMiddlewares: [
      (middlewares) => {
        middlewares.unshift((req, res, next) => {
          if (process.env.AUTH_TYPE === 'basic') {
            res.setHeader('WWW-Authenticate', 'Basic realm="konobangu"');

            const authorization =
              (req.headers.authorization || '').split(' ')[1] || '';
            const [user, password] = Buffer.from(authorization, 'base64')
              .toString()
              .split(':');

            if (
              user !== process.env.BASIC_USER ||
              password !== process.env.BASIC_PASSWORD
            ) {
              res.statusCode = 401;
              res.write('Unauthorized');
              res.end();
              return;
            }
          }
          next();
        });
        return middlewares;
      },
    ],
  },
  server: {
    base: '/api/playground/',
    host: '0.0.0.0',
    port: 5002,
  },
});
