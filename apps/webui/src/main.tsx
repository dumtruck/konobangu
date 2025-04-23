import '@abraham/reflection';
import { provideAuth, setupAuthContext } from '@/app/auth/context';
import { providePlatform } from '@/infra/platform/context';
import { provideStorages } from '@/infra/storage/context';
import { provideStyles } from '@/infra/styles/context';
import { type Injector, ReflectiveInjector } from '@outposts/injection-js';
import { RouterProvider, createRouter } from '@tanstack/react-router';
import {
  InjectorContextVoidInjector,
  InjectorProvider,
} from 'oidc-client-rx/adapters/react';
import { Suspense } from 'react';
import { createRoot } from 'react-dom/client';
import { AppNotFoundComponent } from './components/layout/app-not-found';
import { routeTree } from './routeTree.gen';
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
