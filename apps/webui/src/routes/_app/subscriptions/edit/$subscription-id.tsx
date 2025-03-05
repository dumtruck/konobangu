import { createFileRoute } from '@tanstack/solid-router'

export const Route = createFileRoute('/_app/subscriptions/edit/$subscription-id')({
  component: RouteComponent,
})

function RouteComponent() {
  return <div>Hello "/subscriptions/edit/$subscription-id"!</div>
}
