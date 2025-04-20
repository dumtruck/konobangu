import type { RouteStateDataOption, RouterContext } from '@/traits/router';
import { Outlet, createRootRouteWithContext } from '@tanstack/react-router';
import { Home } from 'lucide-react';

export const Route = createRootRouteWithContext<RouterContext>()({
  component: Outlet,
  staticData: {
    breadcrumb: {
      icon: Home,
    },
  } satisfies RouteStateDataOption,
});
