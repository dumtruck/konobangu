import '@abraham/reflection';
import { type Injector, ReflectiveInjector } from '@outposts/injection-js';
import { RouterProvider, createRouter } from '@tanstack/react-router';
import {
  OidcSecurityService,
  provideAuth,
  withDefaultFeatures,
} from 'oidc-client-rx';
import {
  InjectorContextVoidInjector,
  InjectorProvider,
} from 'oidc-client-rx/adapters/react';
import { withTanstackRouter } from 'oidc-client-rx/adapters/tanstack-router';
import React from 'react';
import ReactDOM from 'react-dom/client';
import { buildOidcConfig, isBasicAuth } from './auth/config';
import { withCheckAuthResultEvent } from './auth/event';
import { useAuth } from './auth/hooks';
import { routeTree } from './routeTree.gen';
import './main.css';

const router = createRouter({
  routeTree,
  basepath: '/api/playground',
  defaultPreload: 'intent',
  context: {
    isAuthenticated: isBasicAuth,
    injector: InjectorContextVoidInjector,
    oidcSecurityService: {} as OidcSecurityService,
  },
});

// Register things for typesafety
declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router;
  }
}

const injector: Injector = isBasicAuth
  ? ReflectiveInjector.resolveAndCreate([])
  : ReflectiveInjector.resolveAndCreate(
      provideAuth(
        {
          config: buildOidcConfig(),
        },
        withDefaultFeatures({
          router: { enabled: false },
          securityStorage: { type: 'local-storage' },
        }),
        withTanstackRouter(router),
        withCheckAuthResultEvent()
      )
    );

// if needed, check when init
let oidcSecurityService: OidcSecurityService | undefined;
if (!isBasicAuth) {
  oidcSecurityService = injector.get(OidcSecurityService);
  oidcSecurityService.checkAuth().subscribe();
}

const AppWithBasicAuth = () => {
  return <RouterProvider router={router} />;
};

const AppWithOidcAuth = () => {
  const { isAuthenticated, oidcSecurityService, injector } = useAuth();
  return (
    <RouterProvider
      router={router}
      context={{
        isAuthenticated,
        oidcSecurityService,
        injector,
      }}
    />
  );
};

const App = isBasicAuth ? AppWithBasicAuth : AppWithOidcAuth;

const rootEl = document.getElementById('root');

if (rootEl) {
  rootEl.classList.add('min-h-svh');
  const root = ReactDOM.createRoot(rootEl);

  root.render(
    <React.StrictMode>
      <InjectorProvider injector={injector}>
        <App />
      </InjectorProvider>
    </React.StrictMode>
  );
}
