import type { RouteStateDataOption } from '@/infra/routes/traits';
import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/_app/tasks/detail/$id')({
  component: TaskDetailRouteComponent,
  staticData: {
    breadcrumb: { label: 'Detail' },
  } satisfies RouteStateDataOption,
});

function TaskDetailRouteComponent() {
  return <div>Hello "/_app/tasks/detail/$id"!</div>;
}
