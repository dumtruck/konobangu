import {
  ApolloClient,
  ApolloLink,
  HttpLink,
  InMemoryCache,
  Observable,
} from "@apollo/client";
import { createSignal, inject, readonlySignal } from "@securitydept/client";
import { ApiTarget } from "@/app/config/context";
import type { AppLifetime } from "@/app/lifetime/scope";
import { AuthService } from "@/domains/auth/auth.service";
import { HTTP_CLIENT } from "@/infra/http/http-client";

/**
 * Owns the Apollo client and its cache. An identity change upstream only tells
 * this service that its cached data belongs to nobody anymore; it then retires
 * the client and publishes a new one. Who may send a request, and with which
 * credential, stays with the HTTP client underneath.
 */
export class GraphQLService {
  private readonly api = inject(ApiTarget);
  private readonly http = inject(HTTP_CLIENT);
  private retirement = new AbortController();
  private ended = false;
  private readonly clientState = createSignal(this.createClient());
  /** The only thing consumers observe: this domain's own current client. */
  readonly client = readonlySignal(this.clientState);
  private readonly subscription = inject(
    AuthService,
  ).identityKeyChanges.subscribe(() => this.replaceClient());

  static createOwned(lifetime: AppLifetime): GraphQLService {
    lifetime.signal.throwIfAborted();
    const service = new GraphQLService();
    return lifetime.own(service, (value) => value.end());
  }

  get _apollo(): ApolloClient {
    return this.clientState.get();
  }
  get query(): ApolloClient["query"] {
    return this._apollo.query.bind(this._apollo);
  }
  get mutate(): ApolloClient["mutate"] {
    return this._apollo.mutate.bind(this._apollo);
  }
  get watchQuery(): ApolloClient["watchQuery"] {
    return this._apollo.watchQuery.bind(this._apollo);
  }

  private createClient(): ApolloClient {
    const retired = this.retirement.signal;
    // A reply that arrives after this client was retired must not reach its
    // observers or cache; cancelling the request alone cannot cover a reply
    // that had already been received.
    const whileCurrent = new ApolloLink(
      (operation, forward) =>
        new Observable((observer) => {
          const subscription = forward(operation).subscribe({
            next: (result) => {
              if (!retired.aborted) observer.next(result);
            },
            error: (error) => {
              if (!retired.aborted) observer.error(error);
            },
            complete: () => observer.complete(),
          });
          return () => subscription.unsubscribe();
        }),
    );
    const createHttp = (replay: "safe" | "never") =>
      new HttpLink({
        uri: this.api.url("graphql"),
        fetch: (input, init) =>
          this.http.apolloFetch(retired, replay)(input, init),
      });
    const http = ApolloLink.split(
      (operation) => operation.operationType === "query",
      createHttp("safe"),
      createHttp("never"),
    );
    return new ApolloClient({
      link: whileCurrent.concat(http),
      cache: new InMemoryCache(),
      defaultOptions: {
        watchQuery: {
          fetchPolicy: "cache-and-network",
          nextFetchPolicy: "network-only",
          errorPolicy: "all",
          refetchWritePolicy: "overwrite",
          initialFetchPolicy: "cache-and-network",
        },
        query: { fetchPolicy: "network-only", errorPolicy: "all" },
        mutate: { errorPolicy: "none" },
      },
      devtools: { enabled: import.meta.env.DEV },
    });
  }

  private retire(): void {
    this.retirement.abort();
    this.clientState.get().stop();
  }

  private replaceClient(): void {
    if (this.ended) return;
    this.retire();
    this.retirement = new AbortController();
    this.clientState.set(this.createClient());
  }

  private end(): void {
    this.ended = true;
    this.subscription.unsubscribe();
    this.retire();
  }
}
