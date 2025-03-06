import '@abraham/reflection';
import {
  ColorModeProvider,
  ColorModeScript,
  createLocalStorageManager,
} from '@kobalte/core';
import { type Injector, ReflectiveInjector } from '@outposts/injection-js';
import { RouterProvider, createRouter } from '@tanstack/solid-router';
import {
  OidcSecurityService,
  provideAuth,
  withCheckAuthResultEvent,
  withDefaultFeatures,
} from 'oidc-client-rx';
import { withTanstackRouter } from 'oidc-client-rx/adapters/@tanstack/solid-router';
import {
  InjectorContextVoidInjector,
  InjectorProvider,
} from 'oidc-client-rx/adapters/solid-js';
import { Dynamic, render } from 'solid-js/web';
import { buildOidcConfig, isBasicAuth, isOidcAuth } from './auth/config';
import { isAuthenticated } from './auth/context';
import { useAuth } from './auth/hooks';
import { routeTree } from './routeTree.gen';
import './app.css';
import { AppNotFoundComponent } from './components/layout/app-not-found';

// Create a new router instance
const router = createRouter({
  routeTree,
  defaultPreload: 'intent',
  defaultStaleTime: 5000,
  scrollRestoration: true,
  defaultNotFoundComponent: AppNotFoundComponent,
  notFoundMode: 'root',
  context: {
    isAuthenticated: isAuthenticated,
    injector: InjectorContextVoidInjector,
    oidcSecurityService: {} as OidcSecurityService,
  },
});

// Register the router instance for type safety
declare module '@tanstack/solid-router' {
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
if (isOidcAuth) {
  oidcSecurityService = injector.get(OidcSecurityService);
  oidcSecurityService.checkAuth().subscribe();
}

// Render the app
const rootElement = document.getElementById('root');

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

const App = () => {
  const storageManager = createLocalStorageManager('color-theme');
  return (
    <>
      <ColorModeScript storageType={storageManager.type} />
      <ColorModeProvider storageManager={storageManager}>
        <InjectorProvider injector={injector}>
          <Dynamic
            component={isBasicAuth ? AppWithBasicAuth : AppWithOidcAuth}
          />
        </InjectorProvider>
      </ColorModeProvider>
    </>
  );
};

if (rootElement && !rootElement.innerHTML) {
  render(App, rootElement);
}
