import { createFileRoute } from '@tanstack/solid-router';

export const Route = createFileRoute('/_app/explore')({
  component: RouteComponent,
});

function RouteComponent() {
  return <div>Hello "/_app/explore"!</div>;
}
