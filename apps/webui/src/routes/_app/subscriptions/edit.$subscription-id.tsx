import { createFileRoute } from '@tanstack/solid-router';
import type { RouteStateDataOption } from '~/traits/router';

export const Route = createFileRoute(
  '/_app/subscriptions/edit/$subscription-id'
)({
  component: RouteComponent,
  staticData: {
    breadcrumb: { label: 'Edit' },
  } satisfies RouteStateDataOption,
});

function RouteComponent() {
  return <div>Hello "/subscriptions/edit/$subscription-id"!</div>;
}
