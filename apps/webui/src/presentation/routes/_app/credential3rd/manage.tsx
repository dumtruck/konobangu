import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/_app/credential3rd/manage')({
  component: CredentialManageRouteComponent,
});

function CredentialManageRouteComponent() {
  return <div>Hello "/_app/credential/manage"!</div>;
}
