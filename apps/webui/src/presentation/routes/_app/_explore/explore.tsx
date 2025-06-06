import type { RouteStateDataOption } from '@/infra/routes/traits';
import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/_app/_explore/explore')({
  component: ExploreRouteComponent,
  staticData: {
    breadcrumb: {
      label: 'Explore',
    },
  } satisfies RouteStateDataOption,
});

function ExploreRouteComponent() {
  return <div>Hello "/_app/explore"!</div>;
}
