import { createFileRoute } from '@tanstack/solid-router';

export const Route = createFileRoute('/_app/subscriptions/manage')({
  component: SubscriptionDashboard,
});

function SubscriptionDashboard() {
  return <div>Hello "/subscriptions/manage"!</div>;
}
