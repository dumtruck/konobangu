import type { RouteStateDataOption } from '@/infra/routes/traits';
import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/_app/_explore/feed')({
  component: FeedRouteComponent,
  staticData: {
    breadcrumb: {
      label: 'Feed',
    },
  } satisfies RouteStateDataOption,
});

function FeedRouteComponent() {
  return <div>Hello "/_app/feed"!</div>;
}
