import type { Injector } from '@outposts/injection-js';
import {
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
      <Outlet />
      <TanStackRouterDevtools position="bottom-right" />
    </>
  );
}
