import type { RouteStateDataOption } from '@/traits/router';
import { createFileRoute } from '@tanstack/react-router';

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
