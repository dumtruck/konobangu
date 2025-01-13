import { createFileRoute, redirect } from '@tanstack/react-router';
import { useEffect } from 'react';
import { useAuth } from 'react-oidc-context';
import { PostLoginRedirectUriKey } from '../../auth/config';

export const Route = createFileRoute('/oidc/callback')({
  component: RouteComponent,
  beforeLoad: ({ context }) => {
    if (!context.auth) {
      throw redirect({
        to: '/',
      });
    }
  },
});

function RouteComponent() {
  const auth = useAuth();

  useEffect(() => {
    if (!auth?.isLoading && auth?.isAuthenticated) {
      try {
        const redirectUri = sessionStorage.getItem(PostLoginRedirectUriKey);
        if (redirectUri) {
          history.replaceState(null, '', redirectUri);
        }
        // biome-ignore lint/suspicious/noEmptyBlockStatements: <explanation>
      } catch {}
    }
  }, [auth]);

  if (auth?.isLoading) {
    return <div>Loading...</div>;
  }

  return (
    <div>
      OpenID Connect Auth Callback Result:{' '}
      {auth.error ? auth.error?.message : 'unknown'}
    </div>
  );
}
