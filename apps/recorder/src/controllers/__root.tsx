import {
  Link,
  Outlet,
  createRootRouteWithContext,
} from '@tanstack/react-router';
import { TanStackRouterDevtools } from '@tanstack/router-devtools';
import type { UserManager } from 'oidc-client-ts';
import type { AuthContextProps } from 'react-oidc-context';

export type RouterContext =
  | {
      isAuthenticated: false;
      auth: AuthContextProps;
      userManager: UserManager;
    }
  | {
      isAuthenticated: true;
      auth?: AuthContextProps;
      userManager?: UserManager;
    };

export const Route = createRootRouteWithContext<RouterContext>()({
  component: RootComponent,
});

function RootComponent() {
  return (
    <>
      {/* <div className="flex gap-2 p-2 text-lg ">
        <Link
          to="/"
          activeProps={{
            className: 'font-bold',
          }}
        >
          Home
        </Link>{' '}
        <Link
          to="/graphql"
          activeProps={{
            className: 'font-bold',
          }}
        >
          GraphQL
        </Link>
      </div> */}
      {/* <hr /> */}
      <Outlet />
      <TanStackRouterDevtools position="bottom-right" />
    </>
  );
}
