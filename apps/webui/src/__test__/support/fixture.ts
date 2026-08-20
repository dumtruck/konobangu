import type { SecuritydeptInjectorTrait } from "@securitydept/client";
import type { ParsedLocation } from "@tanstack/react-router";
import { onTestFinished, vi } from "vitest";
import { OIDC_CLIENT_KEY } from "@/app/auth/setup";
import { tokenSetRegistryFromInjector } from "@/app/auth/tokens";
import {
  type CreateKonobanguEnvironmentForTestOptions,
  createKonobanguEnvironmentForTest,
} from "@/app/environment/test";
import { AuthService } from "@/domains/auth/auth.service";

export type TestRequestHandler = (
  request: Request,
) => Response | Promise<Response>;

export function principalPayload(
  subject = "A",
  issuer = "https://idp.example/",
  authType = "oidc",
) {
  return { subject, displayName: subject, authType, issuer };
}

/** A JSON response; an omitted body serializes as `null`, like the API does. */
export function response(status: number, body: unknown = null): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { "Content-Type": "application/json" },
  });
}

/** The identity key the application derives for a verified principal. */
export function identityKey(
  subject = "A",
  issuer = "https://idp.example/",
  authType = "oidc",
) {
  return JSON.stringify([authType, issuer, subject]);
}

export function parsedLocation(href = "/subscriptions"): ParsedLocation {
  return {
    href,
    publicHref: href,
    pathname: href,
    search: {},
    searchStr: "",
    state: { __TSR_index: 0 },
    hash: "",
    external: false,
  };
}

export interface KonobanguFixtureOptions
  extends Omit<CreateKonobanguEnvironmentForTestOptions, "fetch"> {
  /** Answers every request of the environment; unauthenticated by default. */
  handler?: TestRequestHandler;
  /** Protocol suites exercise real SDK tokens instead of an authorized boundary. */
  realSdkAuthorization?: boolean;
}

/**
 * The one way a unit test obtains the application: the test host environment
 * behind a request handler, released when the test finishes.
 */
export function createKonobanguFixture(options: KonobanguFixtureOptions = {}) {
  const {
    handler = () => response(401),
    realSdkAuthorization,
    providers,
    ...environmentOptions
  } = options;
  const fetch = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) =>
    handler(new Request(input, init)),
  );
  const environment = createKonobanguEnvironmentForTest({
    ...environmentOptions,
    // Normalized so assertions read one (url, init) shape for every caller.
    fetch: async (input, init) => {
      const request = new Request(input, init);
      return fetch(request.url, {
        method: request.method,
        headers: request.headers,
        credentials: request.credentials,
        signal: request.signal,
        body:
          request.method === "GET" || request.method === "HEAD"
            ? undefined
            : await request.text(),
      });
    },
    providers,
  });
  onTestFinished(() => environment[Symbol.dispose]());
  const { injector } = environment;
  const auth = injector.get(AuthService);
  // Domain suites start from an authorized SDK boundary.
  if (!realSdkAuthorization)
    vi.spyOn(auth, "authorize").mockResolvedValue({
      header: "Bearer fixture",
      assertCurrent: () => undefined,
    });
  return {
    environment,
    injector,
    api: environment.api,
    router: environment.appRouter,
    auth,
    fetch,
    /** Requests seen so far, narrowed by API-relative path and method. */
    requests(filter: { path?: string; method?: string } = {}) {
      return fetch.mock.calls.filter(
        ([input, init]) =>
          (filter.path === undefined ||
            String(input) === environment.api.url(filter.path)) &&
          (filter.method === undefined ||
            (init?.method ?? "GET") === filter.method),
      );
    },
  };
}

/**
 * An application whose principal the test controls. `handler` answers every
 * request except the principal read, which follows the current subject.
 */
export async function createSignedInFixture(
  options: KonobanguFixtureOptions & { subject?: string | null } = {},
) {
  const { subject: initial = "A", handler, ...runtimeOptions } = options;
  let subject = initial;
  const principal = () =>
    subject === null ? response(401) : response(200, principalPayload(subject));
  const app = createKonobanguFixture({
    ...runtimeOptions,
    handler: (request) =>
      new URL(request.url).pathname.endsWith("/auth/user-info")
        ? principal()
        : (handler?.(request) ?? response(404)),
  });
  await app.auth.refresh();
  return {
    ...app,
    principal,
    subject: () => subject,
    async identity(next: string | null) {
      subject = next;
      await app.auth.refresh();
    },
  };
}

/** The initialized SDK client record, for suites that drive real token state. */
export function oidcClientRecord(app: {
  readonly injector: SecuritydeptInjectorTrait;
}) {
  return tokenSetRegistryFromInjector(app.injector).clientRecordFor(
    OIDC_CLIENT_KEY,
    { initialize: true },
  );
}

/**
 * A reply the test completes later, to observe the state while a request is
 * in flight. `requested` settles once the code under test actually asked for
 * it, so no case polls for a captured resolver.
 */
export function deferredReply<T>() {
  const reply = Promise.withResolvers<T>();
  const asked = Promise.withResolvers<void>();
  return {
    request: () => {
      asked.resolve();
      return reply.promise;
    },
    requested: asked.promise,
    resolve: reply.resolve,
    reject: reply.reject,
  };
}
