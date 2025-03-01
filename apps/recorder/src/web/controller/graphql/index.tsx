import { type Fetcher, createGraphiQLFetcher } from '@graphiql/toolkit';
import { createFileRoute } from '@tanstack/react-router';
import GraphiQL from 'graphiql';
import { useMemo } from 'react';
import { beforeLoadGuard } from '../../../auth/guard';
import 'graphiql/graphiql.css';
import { firstValueFrom } from 'rxjs';
import { useAuth } from '../../../auth/hooks';

export const Route = createFileRoute('/graphql/')({
  component: RouteComponent,
  beforeLoad: beforeLoadGuard,
});

function RouteComponent() {
  const { oidcSecurityService } = useAuth();

  const fetcher = useMemo(
    (): Fetcher => async (props) => {
      const accessToken = oidcSecurityService
        ? await firstValueFrom(oidcSecurityService.getAccessToken())
        : undefined;
      return createGraphiQLFetcher({
        url: '/api/graphql',
        headers: accessToken
          ? {
              Authorization: `Bearer ${accessToken}`,
            }
          : undefined,
      })(props);
    },
    [oidcSecurityService]
  );

  return <GraphiQL fetcher={fetcher} className="h-svh" />;
}
