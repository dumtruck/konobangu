import type { RouteStateDataOption } from '@/infra/routes/traits';
import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/_app/subscriptions/edit/$id')({
  component: RouteComponent,
  staticData: {
    breadcrumb: { label: 'Edit' },
  } satisfies RouteStateDataOption,
});

function RouteComponent() {
  const { id } = Route.useParams();
  return <div>Hello "/subscriptions/edit/$id"!</div>;
}
