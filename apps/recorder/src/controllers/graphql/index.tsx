import { GraphiQLProvider, QueryEditor } from '@graphiql/react';
import { createGraphiQLFetcher } from '@graphiql/toolkit';
import { createFileRoute } from '@tanstack/react-router';
import { useMemo } from 'react';
import { useAuth } from 'react-oidc-context';
import { beforeLoadGuard } from '../../auth/guard';
import '@graphiql/react/dist/style.css';

export const Route = createFileRoute('/graphql/')({
  component: RouteComponent,
  beforeLoad: beforeLoadGuard,
});

function RouteComponent() {
  const auth = useAuth();

  const fetcher = useMemo(
    () =>
      createGraphiQLFetcher({
        url: '/api/graphql',
        headers: auth?.user?.access_token
          ? {
              Authorization: `Bearer ${auth.user.access_token}`,
            }
          : undefined,
      }),
    [auth]
  );

  return (
    <GraphiQLProvider fetcher={fetcher}>
      <div className="graphiql-container h-svh">
        <QueryEditor />
      </div>
    </GraphiQLProvider>
  );
}
