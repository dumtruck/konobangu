import { createFileRoute } from '@tanstack/solid-router';
import type { RouteStateDataOption } from '~/traits/router';

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
