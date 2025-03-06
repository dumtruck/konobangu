import { Outlet, createFileRoute } from '@tanstack/solid-router';
import { beforeLoadGuard } from '~/auth/guard';
import { AppAside } from '~/components/layout/app-layout';

export const Route = createFileRoute('/_app')({
  component: AppLayoutRoute,
  beforeLoad: beforeLoadGuard,
});

function AppLayoutRoute() {
  return (
    <AppAside extractBreadcrumbFromRoutes>
      <Outlet />
    </AppAside>
  );
}
