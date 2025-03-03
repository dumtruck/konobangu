import {
  ColorModeProvider,
  ColorModeScript,
  createLocalStorageManager,
} from '@kobalte/core';
import { RouterProvider, createRouter } from '@tanstack/solid-router';
import { render } from 'solid-js/web';
import './app.css';

// Import the generated route tree
import { routeTree } from './routeTree.gen';

// Create a new router instance
const router = createRouter({
  routeTree,
  defaultPreload: 'intent',
  defaultStaleTime: 5000,
  scrollRestoration: true,
});

// Register the router instance for type safety
declare module '@tanstack/solid-router' {
  interface Register {
    router: typeof router;
  }
}

// Render the app
const rootElement = document.getElementById('root');

const App = () => {
  const storageManager = createLocalStorageManager('vite-ui-theme');
  return (
    <>
      <ColorModeScript storageType={storageManager.type} />
      <ColorModeProvider storageManager={storageManager}>
        <RouterProvider router={router} />
      </ColorModeProvider>
    </>
  );
};

if (rootElement && !rootElement.innerHTML) {
  render(() => <App />, rootElement);
}
