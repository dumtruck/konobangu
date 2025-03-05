import { createFileRoute } from '@tanstack/solid-router';

export const Route = createFileRoute('/_app/subscriptions/create')({
  component: RouteComponent,
});

function RouteComponent() {
  return <div>Hello "/subscriptions/create"!</div>;
}
