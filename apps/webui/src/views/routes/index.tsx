import { AppAside } from '@/views/components/layout/app-layout';
import { AppSkeleton } from '@/views/components/layout/app-skeleton';
import { createFileRoute } from '@tanstack/react-router';

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
