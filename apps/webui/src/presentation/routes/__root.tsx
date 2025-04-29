import type {
  RouteStateDataOption,
  RouterContext,
} from '@/infra/routes/traits';
import { Outlet, createRootRouteWithContext } from '@tanstack/react-router';
import { Home } from 'lucide-react';
import { memo } from 'react';
import { Toaster } from 'sonner';

export const RootRouteComponent = memo(() => {
  return (
    <>
      <Outlet />
      <Toaster position="top-right" />
    </>
  );
});

export const Route = createRootRouteWithContext<RouterContext>()({
  component: RootRouteComponent,
  staticData: {
    breadcrumb: {
      icon: Home,
    },
  } satisfies RouteStateDataOption,
});
