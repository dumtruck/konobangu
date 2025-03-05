import { createFileRoute } from '@tanstack/solid-router'

export const Route = createFileRoute('/auth/oidc/callback')({
  component: RouteComponent,
})

function RouteComponent() {
  return <div>Hello "/auth/oidc/callback"!</div>
}
