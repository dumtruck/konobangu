import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_app/tasks/cron/detail/$id')({
  component: RouteComponent,
})

function RouteComponent() {
  return <div>Hello "/_app/tasks/cron/detail/$id"!</div>
}
