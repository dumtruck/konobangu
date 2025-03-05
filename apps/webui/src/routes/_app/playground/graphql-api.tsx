import { createFileRoute } from '@tanstack/solid-router'

export const Route = createFileRoute('/_app/playground/graphql-api')({
  component: RouteComponent,
})

function RouteComponent() {
  return <div>Hello "/_app/playground/group-api"!</div>
}
