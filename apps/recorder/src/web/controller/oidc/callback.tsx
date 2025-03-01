import { createFileRoute, redirect } from '@tanstack/react-router';
import { EventTypes } from 'oidc-client-rx';
import { useAuth } from '../../../auth/hooks';

export const Route = createFileRoute('/oidc/callback')({
  component: RouteComponent,
  beforeLoad: ({ context }) => {
    if (!context.oidcSecurityService) {
      throw redirect({
        to: '/',
      });
    }
  },
});

function RouteComponent() {
  const auth = useAuth();

  if (!auth.checkAuthResultEvent) {
    return <div>Loading...</div>;
  }

  return (
    <div>
      OpenID Connect Auth Callback:{' '}
      {auth.checkAuthResultEvent?.type ===
      EventTypes.CheckingAuthFinishedWithError
        ? auth.checkAuthResultEvent.value
        : 'success'}
    </div>
  );
}
