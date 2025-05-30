import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_app/credential3rd/detail/$id')({
  component: RouteComponent,
})

function RouteComponent() {
  return <div>Hello "/_app/credential3rd/detail/$id"!</div>
}
