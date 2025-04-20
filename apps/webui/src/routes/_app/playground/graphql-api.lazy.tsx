import { useAuth } from '@/auth/hooks';
import { type Fetcher, createGraphiQLFetcher } from '@graphiql/toolkit';
import { createLazyFileRoute } from '@tanstack/react-router';
import GraphiQL from 'graphiql';
import { useCallback } from 'react';
import 'graphiql/graphiql.css';
import { AuthMethodEnum } from '@/auth/config';
import { firstValueFrom } from 'rxjs';

export const Route = createLazyFileRoute('/_app/playground/graphql-api')({
  component: PlaygroundGraphQLApiRouteComponent,
});

function PlaygroundGraphQLApiRouteComponent() {
  const { authContext } = useAuth();

  const fetcher: Fetcher = useCallback(
    async (props) => {
      const accessToken =
        authContext.type === AuthMethodEnum.OIDC
          ? await firstValueFrom(
              authContext.oidcSecurityService.getAccessToken()
            )
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
    [
      authContext.type,
      // @ts-ignore
      authContext?.oidcSecurityService?.getAccessToken,
    ]
  );

  return (
    <div
      data-id="graphiql-playground-container"
      className="h-full overflow-hidden rounded-lg"
    >
      <GraphiQL fetcher={fetcher} />
    </div>
  );
}
