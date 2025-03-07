import { type Fetcher, createGraphiQLFetcher } from '@graphiql/toolkit';
import { createLazyFileRoute } from '@tanstack/solid-router';
import GraphiQL from 'graphiql';
import { createElement } from 'react';
import { createRoot } from 'react-dom/client';
import { onCleanup, onMount } from 'solid-js';
import { isOidcAuth } from '~/auth/config';
import { useAuth } from '~/auth/hooks';
import 'graphiql/graphiql.css';
import { firstValueFrom } from 'rxjs';

export const Route = createLazyFileRoute('/_app/playground/graphql-api')({
  component: PlaygroundGraphQLApiRouteComponent,
});

function PlaygroundGraphQLApiRouteComponent() {
  const auth = useAuth();
  let containerRef: HTMLDivElement | undefined;

  onMount(() => {
    if (containerRef) {
      const reactRoot = createRoot(containerRef);
      if (reactRoot) {
        const fetcher: Fetcher = async (props) => {
          const accessToken = isOidcAuth
            ? await firstValueFrom(auth.oidcSecurityService!.getAccessToken())
            : undefined;
          return createGraphiQLFetcher({
            url: '/api/graphql',
            headers: accessToken
              ? {
                  Authorization: `Bearer ${accessToken}`,
                }
              : undefined,
          })(props);
        };
        const graphiql = createElement(GraphiQL, {
          fetcher,
        });
        reactRoot.render(graphiql);

        onCleanup(() => reactRoot.unmount());
      }
    }
  });

  return (
    <div
      data-id="graphiql-playground-container"
      class="h-full overflow-hidden rounded-lg"
      ref={containerRef}
    />
  );
}
