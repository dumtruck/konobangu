import { createFileRoute } from '@tanstack/solid-router';
import type { RouteStateDataOption } from '~/traits/router';

export const Route = createFileRoute('/_app/subscriptions/create')({
  component: SubscriptionCreateRouteComponent,
  staticData: {
    breadcrumb: { label: 'Create' },
  } satisfies RouteStateDataOption,
});

function SubscriptionCreateRouteComponent() {
  return <div>Hello "/subscriptions/create"!</div>;
}
