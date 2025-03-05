import { Outlet, createRootRouteWithContext } from '@tanstack/solid-router';
import type { RouterContext } from '../auth/context';

export const Route = createRootRouteWithContext<RouterContext>()({
  component: () => {
    return <Outlet />;
  },
});
