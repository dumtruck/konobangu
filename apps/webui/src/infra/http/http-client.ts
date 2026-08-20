import {
  type CancellationTokenTrait,
  ClientError,
  ClientErrorKind,
  cancellationTokenToAbortSignal,
  SecuritydeptInjectionToken,
} from "@securitydept/client";
import ky, { isNetworkError } from "ky";
import { PUBLIC_AUTH_CONFIG_PATH } from "@/app/config/auth-config";
import type { ApiTarget } from "@/app/config/context";
import { withinScope } from "./cancellation";

export interface AuthorizationPolicy {
  /** Aborts when the identity that started a request ends. */
  readonly identitySignal: AbortSignal;
  authorize(
    url: URL,
    signal: AbortSignal,
    policy: "principal-probe" | "protected",
  ): Promise<RequestAuthorization>;
  renew(signal: AbortSignal): Promise<void>;
}
export interface RequestAuthorization {
  readonly header: string | undefined;
  /** Checks the borrowed SDK material immediately before using its header. */
  assertCurrent(): void;
}
export interface JsonSchema<T> {
  readonly "~standard": {
    readonly validate: (
      input: unknown,
    ) =>
      | { readonly value: T }
      | { readonly issues: readonly unknown[] }
      | Promise<
          { readonly value: T } | { readonly issues: readonly unknown[] }
        >;
  };
}
export type RequestPolicy = "public" | "principal-probe" | "protected";
export interface HttpRequest {
  readonly endpoint: string;
  readonly method?: string;
  readonly headers?: HeadersInit;
  readonly json?: unknown;
  readonly body?: BodyInit | null;
  readonly signal?: AbortSignal;
  readonly replay?: "never" | "safe";
  readonly policy?: RequestPolicy;
}
export interface JsonRequest<T> extends HttpRequest {
  readonly schema: JsonSchema<T>;
  readonly cancellationToken?: CancellationTokenTrait;
}
export const HTTP_CLIENT = new SecuritydeptInjectionToken<HttpClient>(
  "HTTP_CLIENT",
);
/** HTTP status stays available even when a server supplies its own error code. */
export class HttpResponseError extends ClientError {
  constructor(
    readonly status: number,
    body: unknown,
  ) {
    const mapped = ClientError.fromHttpResponse({ status, body });
    super({
      kind: mapped.kind,
      message: mapped.message,
      code: mapped.code,
      recovery: mapped.recovery,
      retryable: mapped.retryable,
      presentation: mapped.presentation,
      source: mapped.source,
      cause: mapped,
    });
  }
}

/**
 * The request policy controls authorization and the bounded retry budget:
 *
 * | policy          | authorization        | 401                             |
 * | --------------- | -------------------- | ------------------------------- |
 * | public          | none                 | an error like any other status  |
 * | principal-probe | Bearer if available  | returned to the caller          |
 * | protected       | Bearer required      | renew once if replay-safe       |
 */
interface RequestScope {
  readonly url: URL;
  readonly policy: RequestPolicy;
  readonly auth: AuthorizationPolicy | null;
  /** Root, caller and identity lifetimes: the request ends with the first. */
  readonly signal: AbortSignal;
}

/** Borrowed API client; every request ends with its root, caller or identity. */
export class HttpClient {
  private readonly execute;
  constructor(
    private readonly options: {
      api: ApiTarget;
      network: typeof fetch;
      rootSignal: AbortSignal;
      authorization: () => AuthorizationPolicy | null;
    },
  ) {
    // Replay is decided here per request policy, never by the engine.
    this.execute = ky.create({
      fetch: options.network,
      timeout: 15_000,
      retry: 0,
      throwHttpErrors: false,
    });
  }

  async json<T>(options: JsonRequest<T>): Promise<T> {
    const { cancellationToken, schema, ...request } = options;
    cancellationToken?.throwIfCancellationRequested();
    using caller = cancellationTokenToAbortSignal(cancellationToken);
    const scope = this.scope(request, caller?.signal);
    const response = await this.send(request, scope);
    const result = await withinScope(
      (async () => schema["~standard"].validate(await response.json()))(),
      scope.signal,
    );
    if ("issues" in result)
      throw new ClientError({
        kind: ClientErrorKind.Configuration,
        code: "api.response.invalid_schema",
        source: "webui",
        message: "The API response failed schema validation",
        cause: { endpoint: options.endpoint, issues: result.issues },
      });
    return result.value;
  }

  /**
   * The body stays bound to the request scope: the same signal that was given
   * to the network ends an unfinished read when the identity or caller ends.
   */
  async response(options: HttpRequest): Promise<Response> {
    return this.send(options, this.scope(options));
  }

  private scope(options: HttpRequest, caller?: AbortSignal): RequestScope {
    const url = new URL(this.options.api.url(options.endpoint));
    if (!this.options.api.owns(url))
      throw new Error("Request escapes the configured API destination");
    const policy = options.policy ?? "protected";
    if (
      policy !== "public" &&
      url.toString() === this.options.api.url(PUBLIC_AUTH_CONFIG_PATH)
    )
      throw new Error(
        "Public auth configuration must not initialize authentication",
      );
    const auth = policy === "public" ? null : this.options.authorization();
    if (policy !== "public" && !auth)
      throw new Error("Protected requests require an authorization policy");
    const signal = AbortSignal.any(
      [
        this.options.rootSignal,
        options.signal,
        caller,
        auth?.identitySignal,
      ].filter((value) => value !== undefined),
    );
    return { url, policy, auth, signal };
  }

  /** The header for one attempt, confirmed current right before it is used. */
  private async authorization(
    scope: RequestScope,
  ): Promise<string | undefined> {
    const { auth, policy, signal, url } = scope;
    if (!auth || policy === "public") return undefined;
    const authorization = await withinScope(
      auth.authorize(url, signal, policy),
      signal,
    );
    authorization.assertCurrent();
    if (!authorization.header && policy === "protected")
      throw new Error(
        "Protected OIDC requests require an authorization header",
      );
    return authorization.header;
  }

  private async attempt(
    options: HttpRequest,
    scope: RequestScope,
  ): Promise<Response> {
    const headers = new Headers(options.headers);
    // A caller never supplies the credential; it comes from the policy alone.
    headers.delete("Authorization");
    const header = await this.authorization(scope);
    if (header) headers.set("Authorization", header);
    // A terminal that ignores abort must not hold the caller open.
    return withinScope(
      this.execute(scope.url, {
        method: options.method ?? "GET",
        headers,
        ...(options.json === undefined
          ? { body: options.body }
          : { json: options.json }),
        signal: scope.signal,
        credentials: "omit",
        redirect: "error",
        cache: "no-store",
      }),
      scope.signal,
    );
  }

  private async send(
    options: HttpRequest,
    scope: RequestScope,
  ): Promise<Response> {
    const { auth, policy, signal } = scope;
    signal.throwIfAborted();
    // One replay at most, shared by a network failure and a renewed 401, and
    // only for a request its caller declared safe to repeat.
    let replay = options.replay === "safe" && policy === "protected";
    let response: Response;
    try {
      response = await this.attempt(options, scope);
    } catch (error) {
      if (!replay || !isNetworkError(error) || signal.aborted) throw error;
      replay = false;
      response = await this.attempt(options, scope);
    }
    if (response.status === 401 && policy === "protected" && auth) {
      if (replay) {
        await response.body?.cancel();
        await withinScope(auth.renew(signal), signal);
        response = await this.attempt(options, scope);
      }
    }
    signal.throwIfAborted();
    const tolerated =
      response.ok ||
      [304, 416].includes(response.status) ||
      (policy === "principal-probe" && response.status === 401);
    if (tolerated) return response;
    const text = await response.text();
    let body: unknown = text;
    try {
      body = JSON.parse(text);
    } catch {
      /* Non-JSON error bodies remain diagnostic text. */
    }
    throw new HttpResponseError(response.status, body);
  }

  /** Only protocol adapters receive this function; domains use schema requests. */
  apolloFetch(
    signal: AbortSignal,
    replay: "never" | "safe" = "never",
  ): typeof fetch {
    return async (input, init) => {
      const request = new Request(input, init);
      if (!this.options.api.owns(request.url))
        throw new Error("Apollo destination is outside this API");
      const target = new URL(request.url);
      const endpoint =
        target.pathname.slice(this.options.api.baseUrl.pathname.length) +
        target.search;
      return this.response({
        endpoint,
        method: request.method,
        headers: request.headers,
        body:
          request.method === "GET" || request.method === "HEAD"
            ? undefined
            : await request.text(),
        signal: AbortSignal.any([signal, request.signal]),
        replay,
      });
    };
  }
}
