import { createFileRoute } from '@tanstack/solid-router';

export const Route = createFileRoute('/auth/sign-in')({
  component: RouteComponent,
});

function RouteComponent() {
  return <div>Hello "/auth/sign-in"!</div>;
}
