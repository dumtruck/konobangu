import { createGraphiQLFetcher, type Fetcher } from "@graphiql/toolkit";
import { createLazyFileRoute } from "@tanstack/react-router";
import { GraphiQL } from "graphiql";
import { useCallback } from "react";
import { ApiTarget } from "@/app/config/context";
import { AuthService } from "@/domains/auth/auth.service";
import { useInject } from "@/infra/di/inject";
import { HTTP_CLIENT } from "@/infra/http/http-client";
// Registers the editor, JSON and GraphQL workers through Vite's worker imports.
// It must be evaluated before the editor initializes, and it stays inside this
// lazy route so the editor chunk is still deferred.
import "graphiql/setup-workers/vite";
import "graphiql/style.css";

export const Route = createLazyFileRoute("/_app/playground/graphql-api")({
  component: PlaygroundGraphQLApiRouteComponent,
});

function PlaygroundGraphQLApiRouteComponent() {
  const api = useInject(ApiTarget);
  const http = useInject(HTTP_CLIENT);
  const auth = useInject(AuthService);
  const url = api.url("graphql");
  const fetcher: Fetcher = useCallback(
    async (props) => {
      return createGraphiQLFetcher({
        url,
        fetch: http.apolloFetch(auth.identitySignal),
      })(props);
    },
    [url, http, auth],
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
