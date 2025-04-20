import type { RouteStateDataOption } from '@/traits/router';
import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/_app/subscriptions/manage')({
  component: SubscriptionManage,
  staticData: {
    breadcrumb: { label: 'Manage' },
  } satisfies RouteStateDataOption,
});

function SubscriptionManage() {
  return <div>Hello "/subscriptions/manage"!</div>;
}
