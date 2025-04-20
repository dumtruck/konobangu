import { beforeLoadGuard } from '@/auth/guard';
import { AppAside } from '@/components/layout/app-layout';
import { Outlet, createFileRoute } from '@tanstack/react-router';

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
