import { createFileRoute } from '@tanstack/solid-router';
import type { RouteStateDataOption } from '~/traits/router';

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
