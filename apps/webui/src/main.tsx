import '@abraham/reflection';
import { type Injector, ReflectiveInjector } from '@outposts/injection-js';
import { RouterProvider, createRouter } from '@tanstack/react-router';
import {
  InjectorContextVoidInjector,
  InjectorProvider,
} from 'oidc-client-rx/adapters/react';
import { Suspense } from 'react';
import { createRoot } from 'react-dom/client';
import { provideAuth, setupAuthContext } from './auth/context';
import { AppNotFoundComponent } from './components/layout/app-not-found';
import { providePlatform } from './platform/context';
import { routeTree } from './routeTree.gen';
import { provideStorages } from './storage/context';
import { provideStyles } from './styles/context';
import './app.css';

// Create a new router instance
const router = createRouter({
  routeTree,
  defaultPreload: 'intent',
  defaultStaleTime: 5000,
  scrollRestoration: true,
  defaultNotFoundComponent: AppNotFoundComponent,
  notFoundMode: 'root',
  context: {
    injector: InjectorContextVoidInjector,
  },
});

// Register the router instance for type safety
declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router;
  }
}

const injector: Injector = ReflectiveInjector.resolveAndCreate([
  ...providePlatform(),
  ...provideStorages(),
  ...provideAuth(router),
  ...provideStyles(),
]);

setupAuthContext(injector);

const rootElement = document.getElementById('root');

const App = () => {
  return (
    <Suspense>
      <InjectorProvider injector={injector}>
        <RouterProvider
          router={router}
          context={{
            injector,
          }}
        />
      </InjectorProvider>
    </Suspense>
  );
};

if (rootElement) {
  const root = createRoot(rootElement);
  root.render(<App />);
}
