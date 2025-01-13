import { RouterProvider, createRouter } from '@tanstack/react-router';
import type { UserManager } from 'oidc-client-ts';
import { useMemo } from 'react';
import { AuthProvider, useAuth } from 'react-oidc-context';
import { buildUserManager } from '../auth/config';
import { routeTree } from '../routeTree.gen';

// Set up a Router instance
const router = createRouter({
  routeTree,
  basepath: '/api/playground',
  defaultPreload: 'intent',
  context: {
    isAuthenticated: process.env.AUTH_TYPE === 'basic',
    auth: undefined!,
    userManager: undefined!,
  },
});

// Register things for typesafety
declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router;
  }
}

const AppWithBasicAuth = () => {
  return <RouterProvider router={router} />;
};

const AppWithOidcAuthInner = ({
  userManager,
}: { userManager: UserManager }) => {
  const auth = useAuth();
  return (
    <RouterProvider
      router={router}
      context={{ isAuthenticated: auth.isAuthenticated, auth, userManager }}
    />
  );
};

const AppWithOidcAuth = () => {
  const userManager = useMemo(() => buildUserManager(), []);
  return (
    <AuthProvider userManager={userManager}>
      <AppWithOidcAuthInner userManager={userManager} />
    </AuthProvider>
  );
};

export const App =
  process.env.AUTH_TYPE === 'oidc' ? AppWithOidcAuth : AppWithBasicAuth;
