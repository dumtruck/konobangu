import type { RouteStateDataOption } from '@/traits/router';
import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/_app/subscriptions/create')({
  component: SubscriptionCreateRouteComponent,
  staticData: {
    breadcrumb: { label: 'Create' },
  } satisfies RouteStateDataOption,
});

function SubscriptionCreateRouteComponent() {
  return <div>Hello "/subscriptions/create"!</div>;
}
