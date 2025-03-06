import { createFileRoute } from '@tanstack/solid-router';
import type { RouteStateDataOption } from '~/traits/router';

export const Route = createFileRoute('/_app/subscriptions/manage')({
  component: SubscriptionManage,
  staticData: {
    breadcrumb: { label: 'Manage' },
  } satisfies RouteStateDataOption,
});

function SubscriptionManage() {
  return <div>Hello "/subscriptions/manage"!</div>;
}
