import type { RouteStateDataOption } from '@/infra/routes/traits';
import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/_app/tasks/manage')({
  component: TaskManageRouteComponent,
  staticData: {
    breadcrumb: { label: 'Manage' },
  } satisfies RouteStateDataOption,
});

function TaskManageRouteComponent() {
  return <div>Hello "/_app/tasks/manage"!</div>;
}
