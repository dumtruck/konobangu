import { createFileRoute } from '@tanstack/solid-router';
import { AppAside } from '~/components/layout/app-layout';
import { AppSkeleton } from '~/components/layout/app-skeleton';

export const Route = createFileRoute('/')({
  component: HomeRouteComponent,
});

function HomeRouteComponent() {
  return (
    <AppAside extractBreadcrumbFromRoutes>
      <AppSkeleton />
    </AppAside>
  );
}
