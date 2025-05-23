import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/_app/credential3rd/create')({
  component: CredentialCreateRouteComponent,
});

function CredentialCreateRouteComponent() {
  return <div>Hello "/_app/credential/create"!</div>;
}
