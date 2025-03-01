import type { Injector } from '@outposts/injection-js';
import {
  // Link,
  Outlet,
  createRootRouteWithContext,
} from '@tanstack/react-router';
import { TanStackRouterDevtools } from '@tanstack/router-devtools';
import type { OidcSecurityService } from 'oidc-client-rx';

export type RouterContext =
  | {
      isAuthenticated: false;
      injector: Injector;
      oidcSecurityService: OidcSecurityService;
    }
  | {
      isAuthenticated: true;
      injector?: Injector;
      oidcSecurityService?: OidcSecurityService;
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
