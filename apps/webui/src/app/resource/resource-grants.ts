import {
  inject,
  SecuritydeptInjectionToken,
  type SecuritydeptProvider,
  TIME_TRAIT_TOKEN,
  type TimeTrait,
} from "@securitydept/client";
import { ArkErrors, type } from "arktype";
import { NetworkError, TimeoutError } from "ky";
import { Observable } from "rxjs";
import { ApiTarget } from "@/app/config/context";
import {
  PAGE_VISIBILITY,
  type PageVisibilityTrait,
  RESOURCE_CRYPTO,
  type ResourceCryptoTrait,
  VIEWPORT,
  type ViewportTrait,
} from "@/app/environment/traits";
import { APP_LIFETIME } from "@/app/lifetime/scope";
import { AuthService } from "@/domains/auth/auth.service";
import { withinScope } from "@/infra/http/cancellation";
import { HTTP_CLIENT, HttpResponseError } from "@/infra/http/http-client";
import { ImageBinding, ResourceBinding } from "./resource-binding";

const GrantReply = type({
  "+": "reject",
  grant: "string",
  issuedAt: "string",
  expiresAt: "string",
  childTtlSeconds: "300",
});
const Parent = type({
  "+": "reject",
  v: "2",
  "i?": "string | null",
  "i64?": "string | null",
  "l?": "null",
  "l64?": "null",
  "s?": "null",
  c: "unknown[]",
  s64: "string",
});
const Identifier = type({
  "+": "reject",
  v: "1",
  kid: "string",
  audience: '"urn:konobangu:native-resources"',
  subscriberId: "number.integer > 0",
  scope: '"native-resource:read"',
  issuedAt: "number.integer",
  expiresAt: "number.integer",
  childTtlSeconds: "300",
});
const encoder = new TextEncoder();
const decoder = new TextDecoder("utf-8", { fatal: true });
export function encodeBase64Url(bytes: Uint8Array): string {
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary)
    .replaceAll("+", "-")
    .replaceAll("/", "_")
    .replaceAll("=", "");
}
export function decodeBase64Url(value: string): Uint8Array {
  if (!/^[A-Za-z0-9_-]+$/.test(value)) throw new Error("Invalid base64url");
  const bytes = Uint8Array.from(
    atob(value.replaceAll("-", "+").replaceAll("_", "/")),
    (c) => c.charCodeAt(0),
  );
  if (encodeBase64Url(bytes) !== value)
    throw new Error("Noncanonical base64url");
  return bytes;
}
export interface ResourceAuth {
  epoch(): number;
  readonly identitySignal: AbortSignal;
  readonly lifetimeSignal: AbortSignal;
  readonly identityChanges: Observable<unknown>;
}
export type NativeResource = {
  readonly url: string;
  readonly expiresAt: number;
};
/** Borrowed application service; only its factory can end its ownership. */
export interface ResourceService {
  isPrivate(input: string): boolean;
  acquire(input: string, renewal?: "renew"): Promise<NativeResource>;
  activate(): Disposable;
  /** A connectable address for a view; public locators need no grant. */
  bind(input: string): ResourceBinding;
  /** A connectable address for one image element. */
  bindImage(input: string): ImageBinding;
  nearViewport(element: Element, run: () => void): () => void;
}
export const RESOURCE_SERVICE = new SecuritydeptInjectionToken<ResourceService>(
  "RESOURCE_SERVICE",
);

type ParsedGrant = {
  readonly token: {
    readonly i?: string | null;
    readonly i64?: string | null;
    readonly s64: string;
  };
  readonly identity: typeof Identifier.infer;
  readonly expiresAt: number;
};
function isTransient(error: unknown): boolean {
  if (error instanceof HttpResponseError)
    return error.status >= 500 && error.status <= 599;
  return error instanceof NetworkError || error instanceof TimeoutError;
}
export function createResourceService(options: {
  api: ApiTarget;
  auth: ResourceAuth;
  time: TimeTrait;
  visibility: PageVisibilityTrait;
  viewport: ViewportTrait;
  crypto: ResourceCryptoTrait;
  issue(signal: AbortSignal): Promise<typeof GrantReply.infer>;
}): ResourceService & Disposable {
  const { api, auth, time, visibility: page, viewport, crypto } = options;
  const after = (delayMs: number, run: () => void) => {
    const timer = time.setTimeout(run, delayMs);
    return () => time.clearTimeout(timer);
  };
  const owner = new AbortController();
  let parent: ParsedGrant | undefined;
  let pending: Promise<ParsedGrant> | undefined;
  let offset = 0;
  let active = 0;
  let stopTimer: (() => void) | undefined;
  let failures = 0;
  let activity = new AbortController();
  let visibilityScope = new AbortController();
  if (!page.visible)
    visibilityScope.abort(new DOMException("Page hidden", "AbortError"));
  const now = () => time.now() + offset;
  const stop = () => {
    stopTimer?.();
    stopTimer = undefined;
  };
  const signal = () =>
    AbortSignal.any([
      owner.signal,
      auth.lifetimeSignal,
      auth.identitySignal,
      visibilityScope.signal,
      ...(active ? [activity.signal] : []),
    ]);
  const parseResource = (input: string) => {
    const url = new URL(input, api.baseUrl);
    const base = new URL(api.url("static/subscribers/"));
    const feedBase = new URL(api.url("feeds/rss/"));
    const feed = url.pathname.startsWith(feedBase.pathname);
    if (
      url.origin !== base.origin ||
      (!url.pathname.startsWith(base.pathname) && !feed)
    )
      return null;
    if (
      url.username ||
      url.password ||
      url.hash ||
      [...url.searchParams.keys()].some((key) => feed || key !== "optimize")
    )
      throw new Error("Invalid native resource URL");
    if (feed) {
      const token = decodeURIComponent(
        url.pathname.slice(feedBase.pathname.length),
      );
      if (!/^[A-Za-z0-9_-]{1,128}$/.test(token))
        throw new Error("Invalid feed resource path");
      return { url, path: `feeds/rss/${token}`, subscriberId: undefined };
    }
    // URL normalizes dot segments; reject them before that normalization can
    // silently turn a borrowed path into a different resource capability.
    if (
      input
        .split(/[?#]/, 1)[0]
        .split("/")
        .some((segment) => [".", ".."].includes(decodeURIComponent(segment)))
    )
      throw new Error("Invalid native image resource path");
    const parts = url.pathname
      .slice(base.pathname.length)
      .split("/")
      .map((part) => decodeURIComponent(part));
    if (
      parts.length < 4 ||
      !/^[1-9]\d*$/.test(parts[0]) ||
      parts[1] !== "image" ||
      parts.some(
        (part) =>
          !part ||
          part === "." ||
          part === ".." ||
          part.includes("\0") ||
          /[\\/:%]/.test(part),
      )
    )
      throw new Error("Invalid native image resource path");
    const subscriberId = Number(parts.shift());
    if (!Number.isSafeInteger(subscriberId))
      throw new Error("Invalid resource subscriber");
    if (
      url.username ||
      url.password ||
      url.hash ||
      [...url.searchParams.keys()].some((key) => key !== "optimize")
    )
      throw new Error("Invalid native image resource URL");
    return { url, subscriberId, path: parts.join("/") };
  };
  const schedule = () => {
    stop();
    if (
      owner.signal.aborted ||
      auth.lifetimeSignal.aborted ||
      !page.visible ||
      !active ||
      !parent
    )
      return;
    const delay = failures
      ? Math.min(60_000, 5000 * 2 ** (failures - 1))
      : Math.max(0, parent.expiresAt - now() - 300_000);
    if (failures >= 3) return;
    stopTimer = after(delay, () => {
      stopTimer = undefined;
      void obtain(true).catch((error: unknown) => {
        if (!isTransient(error)) return;
        failures++;
        schedule();
      });
    });
  };
  const wait = (delay: number, currentSignal: AbortSignal) =>
    new Promise<void>((resolve, reject) => {
      currentSignal.throwIfAborted();
      const cancel = after(delay, () => {
        cleanup();
        resolve();
      });
      const abort = () => {
        cleanup();
        reject(currentSignal.reason);
      };
      const cleanup = () => {
        cancel();
        currentSignal.removeEventListener("abort", abort);
      };
      currentSignal.addEventListener("abort", abort, { once: true });
      if (currentSignal.aborted) abort();
    });
  const issue = async (currentSignal: AbortSignal) => {
    for (let attempt = 0; ; attempt++) {
      try {
        return await withinScope(options.issue(currentSignal), currentSignal);
      } catch (error) {
        currentSignal.throwIfAborted();
        if (
          parent ||
          !active ||
          !page.visible ||
          attempt >= 3 ||
          !isTransient(error)
        )
          throw error;
        await wait(5000 * 2 ** attempt, currentSignal);
      }
    }
  };
  const obtain = (renew = false): Promise<ParsedGrant> => {
    const currentSignal = signal();
    currentSignal.throwIfAborted();
    if (!renew && parent && parent.expiresAt > now() + 1000)
      return Promise.resolve(parent);
    if (pending) return pending;
    const epoch = auth.epoch();
    const operation = issue(currentSignal)
      .then((reply) => {
        currentSignal.throwIfAborted();
        if (epoch !== auth.epoch())
          throw new Error("Resource grant identity changed");
        const validated = GrantReply(reply);
        if (validated instanceof ArkErrors) throw new Error(validated.summary);
        const issuedAt = Temporal.Instant.from(
          validated.issuedAt,
        ).epochMilliseconds;
        const expiresAt = Temporal.Instant.from(
          validated.expiresAt,
        ).epochMilliseconds;
        const token = Parent(JSON.parse(validated.grant));
        if (
          token instanceof ArkErrors ||
          token.c.length !== 0 ||
          !!token.i64 === !!token.i
        )
          throw new Error("Invalid parent resource grant");
        const encodedId = token.i64
          ? decoder.decode(decodeBase64Url(token.i64))
          : token.i;
        if (!encodedId) throw new Error("Missing resource grant identifier");
        const identity = Identifier(JSON.parse(encodedId));
        if (
          identity instanceof ArkErrors ||
          decodeBase64Url(token.s64).length !== 32 ||
          identity.expiresAt * 1000 !== expiresAt ||
          identity.issuedAt * 1000 !== issuedAt ||
          expiresAt <= issuedAt ||
          expiresAt - issuedAt > 3_600_000
        )
          throw new Error("Invalid resource grant contract");
        offset = issuedAt - time.now();
        parent = { token, identity, expiresAt };
        failures = 0;
        schedule();
        return parent;
      })
      .finally(() => {
        if (pending === operation) pending = undefined;
      });
    pending = operation;
    return operation;
  };
  const identity = auth.identityChanges.subscribe(() => {
    parent = undefined;
    pending = undefined;
    offset = 0;
    failures = 0;
    stop();
  });
  const visibility = page.changes(() => {
    if (!page.visible) {
      visibilityScope.abort(new DOMException("Page hidden", "AbortError"));
      pending = undefined;
      stop();
    } else {
      visibilityScope = new AbortController();
      failures = 0;
      schedule();
    }
  });
  /** A bounded address kept fresh while something is connected to it. */
  const observe = (input: string) =>
    new Observable<NativeResource | undefined>((subscriber) => {
      const lease = service.activate();
      const lifetime = new AbortController();
      let derivation = new AbortController();
      let timer: ReturnType<TimeTrait["setTimeout"]> | undefined;
      const clear = () => {
        derivation.abort();
        if (timer !== undefined) time.clearTimeout(timer);
        timer = undefined;
      };
      const refresh = () => {
        clear();
        subscriber.next(undefined);
        if (!page.visible || lifetime.signal.aborted) return;
        derivation = new AbortController();
        const current = AbortSignal.any([
          lifetime.signal,
          derivation.signal,
          owner.signal,
          auth.lifetimeSignal,
        ]);
        void withinScope(service.acquire(input), current)
          .then((resource) => {
            current.throwIfAborted();
            subscriber.next(resource);
            current.throwIfAborted();
            timer = time.setTimeout(
              refresh,
              Math.max(0, resource.expiresAt - time.now() - 15_000),
            );
          })
          .catch((error: unknown) => {
            if (!current.aborted) subscriber.error(error);
          });
      };
      const end = AbortSignal.any([owner.signal, auth.lifetimeSignal]);
      const complete = () => subscriber.complete();
      end.addEventListener("abort", complete, { once: true });
      const changed = auth.identityChanges.subscribe(refresh);
      const visible = page.changes(refresh);
      refresh();
      return () => {
        lifetime.abort();
        clear();
        changed.unsubscribe();
        visible();
        end.removeEventListener("abort", complete);
        lease[Symbol.dispose]();
      };
    });
  const service: ResourceService & Disposable = {
    isPrivate(input) {
      return parseResource(input) !== null;
    },
    async acquire(input, renewal) {
      const resource = parseResource(input);
      if (!resource)
        throw new Error(
          "Resource does not belong to a configured native endpoint",
        );
      const currentSignal = signal();
      let grant: ParsedGrant;
      try {
        grant = await obtain(
          renewal === "renew" ||
            (!!parent && parent.expiresAt - now() <= 300_000),
        );
      } catch (error) {
        currentSignal.throwIfAborted();
        if (!isTransient(error) || !parent || parent.expiresAt - now() < 30_000)
          throw error;
        grant = parent;
      }
      currentSignal.throwIfAborted();
      if (
        resource.subscriberId !== undefined &&
        resource.subscriberId !== grant.identity.subscriberId
      )
        throw new Error("Resource belongs to another subscriber");
      const issuedAt = Math.floor(now() / 1000);
      const expiresAt = Math.min(issuedAt + 300, grant.identity.expiresAt);
      const predicates = [
        `resource = ${JSON.stringify({ subscriberId: grant.identity.subscriberId, path: resource.path })}`,
        `window = ${JSON.stringify({ issuedAt, expiresAt })}`,
      ];
      let signature = decodeBase64Url(grant.token.s64);
      for (const predicate of predicates)
        signature = await crypto.hmac(signature, encoder.encode(predicate));
      currentSignal.throwIfAborted();
      const child = {
        v: 2,
        ...(grant.token.i64 ? { i64: grant.token.i64 } : { i: grant.token.i }),
        c: predicates.map((i) => ({ i64: encodeBase64Url(encoder.encode(i)) })),
        s64: encodeBase64Url(signature),
      };
      resource.url.searchParams.set(
        "resource_token",
        encodeBase64Url(encoder.encode(JSON.stringify(child))),
      );
      return {
        url: resource.url.toString(),
        expiresAt: expiresAt * 1000 - offset,
      };
    },
    activate() {
      owner.signal.throwIfAborted();
      auth.lifetimeSignal.throwIfAborted();
      if (!active) activity = new AbortController();
      active++;
      failures = 0;
      schedule();
      return {
        [Symbol.dispose]() {
          active--;
          if (!active) {
            activity.abort(
              new DOMException("No active resources", "AbortError"),
            );
            pending = undefined;
          }
          schedule();
        },
      };
    },
    bindImage(input) {
      return new ImageBinding({
        input,
        private: service.isPrivate(input),
        acquire: (value, renewal) => service.acquire(value, renewal),
        activate: () => service.activate(),
        identityChanges: {
          subscribe: (next) => auth.identityChanges.subscribe(next),
        },
      });
    },
    bind(input) {
      return new ResourceBinding({
        input,
        private: service.isPrivate(input),
        observe,
      });
    },
    nearViewport: (element, run) => viewport.nearViewport(element, run),
    [Symbol.dispose]() {
      owner.abort(new Error("Resource owner ended"));
      stop();
      identity.unsubscribe();
      visibility();
      parent = undefined;
      pending = undefined;
    },
  };
  return service;
}
export function provideResourceService(): SecuritydeptProvider {
  return {
    provide: RESOURCE_SERVICE,
    useFactory: () => {
      const http = inject(HTTP_CLIENT);
      const service = createResourceService({
        api: inject(ApiTarget),
        auth: inject(AuthService),
        time: inject(TIME_TRAIT_TOKEN),
        visibility: inject(PAGE_VISIBILITY),
        viewport: inject(VIEWPORT),
        crypto: inject(RESOURCE_CRYPTO),
        issue: (signal) =>
          http.json({
            endpoint: "auth/resource-grants",
            method: "POST",
            json: {},
            replay: "safe",
            schema: GrantReply,
            signal,
          }),
      });
      const { [Symbol.dispose]: dispose, ...borrowed } = service;
      inject(APP_LIFETIME).own(service, () => dispose());
      return borrowed satisfies ResourceService;
    },
  };
}
