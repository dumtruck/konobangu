import { Outlet, createRootRouteWithContext } from '@tanstack/solid-router';
import { Home } from 'lucide-solid';
import type { RouteStateDataOption, RouterContext } from '~/traits/router';

export const Route = createRootRouteWithContext<RouterContext>()({
  component: Outlet,
  staticData: {
    breadcrumb: {
      icon: Home,
    },
  } satisfies RouteStateDataOption,
});
