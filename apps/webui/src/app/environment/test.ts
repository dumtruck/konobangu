import {
  type CreateFoundationEnvironmentOptions,
  createTimeForStd,
  type SecuritydeptProvider,
  type TimeTrait,
} from "@securitydept/client";
import { createEnvironmentForTest } from "@securitydept/client/test";
import { createEnvironmentForReact } from "@securitydept/client-react";
import { injectConfigProjectionIntoRealm } from "@securitydept/token-set-context-client/frontend-oidc-mode";
import {
  type AnyRouter,
  createMemoryHistory,
  createRootRoute,
  createRouter,
} from "@tanstack/react-router";
import {
  type AuthSetup,
  callbackUrlForOrigin,
  OIDC_CLIENT_KEY,
} from "@/app/auth/setup";
import { parseRuntimeConfig, type RuntimeConfig } from "@/app/config/runtime";
import {
  type CreateKonobanguBootstrapEnvironmentOptions,
  createKonobanguBootstrapEnvironment,
  type KonobanguBootstrapEnvironment,
} from "./bootstrap";
import { createResourceCrypto } from "./crypto";
import {
  type CreateKonobanguEnvironmentOptions,
  createKonobanguEnvironment,
  type KonobanguEnvironment,
  type KonobanguTraits,
} from "./konobangu";
import {
  createUiStorageForMemory,
  type FetchTrait,
  type MediaQueryTrait,
  MissingEnvironmentTraitError,
  type PageVisibilityTrait,
  type ViewportTrait,
} from "./traits";

export interface TestMediaQueryTrait extends MediaQueryTrait {
  set(matches: boolean): void;
  readonly listenerCount: number;
}
export function createMediaQueryForTest(initial = false): TestMediaQueryTrait {
  let matches = initial;
  const listeners = new Set<(value: boolean) => void>();
  return {
    get matches() {
      return matches;
    },
    changes(listener) {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    set(next) {
      matches = next;
      for (const listener of listeners) listener(next);
    },
    get listenerCount() {
      return listeners.size;
    },
  };
}

export interface TestPageVisibilityTrait extends PageVisibilityTrait {
  set(visible: boolean): void;
}
export function createPageVisibilityForTest(
  initial = true,
): TestPageVisibilityTrait {
  let visible = initial;
  const listeners = new Set<() => void>();
  return {
    get visible() {
      return visible;
    },
    changes(listener) {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    set(next) {
      visible = next;
      for (const listener of listeners) listener();
    },
  };
}

export interface TestViewportTrait extends ViewportTrait {
  /** Reports every observed element, or only the given one, as near. */
  approach(element?: Element): void;
  readonly observedCount: number;
}
export function createViewportForTest(): TestViewportTrait {
  const watchers = new Map<Element, () => void>();
  return {
    nearViewport(element, run) {
      watchers.set(element, run);
      return () => watchers.delete(element);
    },
    approach(element) {
      if (element) watchers.get(element)?.();
      else for (const run of [...watchers.values()]) run();
    },
    get observedCount() {
      return watchers.size;
    },
  };
}

export function createRouterForMemory(initialPath = "/"): AnyRouter {
  return createRouter({
    routeTree: createRootRoute(),
    history: createMemoryHistory({ initialEntries: [initialPath] }),
  });
}

/**
 * Builds the OIDC realm the way bootstrap does, with explicit endpoints so no
 * test performs provider discovery.
 */
export function createOidcAuthSetupForTest(documentOrigin: string): AuthSetup {
  const realm = {};
  injectConfigProjectionIntoRealm({
    clientKey: OIDC_CLIENT_KEY,
    projection: {
      clientId: "konobangu-webui",
      issuerUrl: "https://idp.example",
      authorizationEndpoint: "https://idp.example/auth",
      tokenEndpoint: "https://idp.example/token",
      jwksUri: "https://idp.example/jwks",
      redirectUrl: callbackUrlForOrigin(documentOrigin),
      scopes: ["openid", "profile", "konobangu:api"],
      requiredScopes: ["konobangu:api"],
      pkceEnabled: true,
      generatedAt: 1,
    },
    realm,
  });
  return {
    clientKey: OIDC_CLIENT_KEY,
    resource: {
      audience: "urn:konobangu:api",
      requiredScopes: ["konobangu:api"],
    },
    realm,
  };
}

export interface KonobanguTestTraits extends KonobanguTraits {
  systemDarkTheme: TestMediaQueryTrait;
  mobileViewport: TestMediaQueryTrait;
  pageVisibility: TestPageVisibilityTrait;
  viewport: TestViewportTrait;
}
export type KonobanguTestEnvironment = KonobanguEnvironment &
  KonobanguTestTraits;

export interface CreateKonobanguEnvironmentForTestOptions
  extends Partial<
    Pick<CreateKonobanguEnvironmentOptions, "appRouter" | "documentOrigin">
  > {
  config?: RuntimeConfig;
  auth?: AuthSetup;
  fetch?: FetchTrait;
  time?: TimeTrait;
  document?: Document;
  locale?: string;
  dark?: boolean;
  mobile?: boolean;
  initialPath?: string;
  providers?: readonly SecuritydeptProvider[];
}

/**
 * The test host: deterministic traits over the SDK test environment. A test
 * replaces a unit through `providers`, never by rebuilding this composition.
 */
export function createKonobanguEnvironmentForTest(
  options: CreateKonobanguEnvironmentForTestOptions = {},
): KonobanguTestEnvironment {
  const document = options.document ?? globalThis.document;
  if (!document) throw new MissingEnvironmentTraitError("document");
  // Mirror the real entry: the API base resolves against the document origin.
  const documentOrigin = options.documentOrigin ?? document.location.origin;
  const auth = options.auth ?? createOidcAuthSetupForTest(documentOrigin);
  return createKonobanguEnvironment<KonobanguTestTraits>({
    createBaseEnvironment: (baseOptions) =>
      createEnvironmentForTest({
        ...baseOptions,
        createBaseEnvironment: (
          reactOptions: CreateFoundationEnvironmentOptions,
        ) => createEnvironmentForReact(reactOptions),
      }),
    traits: {
      document,
      fetch:
        options.fetch ??
        (async () => {
          throw new Error("Test environment fetch was not provided.");
        }),
      locale: options.locale ?? "en-US",
      platformInfo: { userAgent: "konobangu-test" },
      systemDarkTheme: createMediaQueryForTest(options.dark),
      mobileViewport: createMediaQueryForTest(options.mobile),
      uiPreferenceStorage: createUiStorageForMemory(),
      uiSessionStorage: createUiStorageForMemory(),
      pageVisibility: createPageVisibilityForTest(),
      viewport: createViewportForTest(),
      resourceCrypto: createResourceCrypto(globalThis.crypto),
    },
    config:
      options.config ??
      parseRuntimeConfig(
        { version: 1, apiBaseUrl: "/api/" },
        { origin: documentOrigin, allowInsecureLoopback: true },
      ),
    auth,
    documentOrigin,
    appRouter: options.appRouter ?? createRouterForMemory(options.initialPath),
    // Real time unless a test supplies a controllable clock.
    time: options.time ?? createTimeForStd(),
    sessionStorage: createUiStorageForMemory(),
    providers: options.providers,
  });
}

export function createKonobanguBootstrapEnvironmentForTest(
  options: Partial<CreateKonobanguBootstrapEnvironmentOptions> = {},
): KonobanguBootstrapEnvironment {
  return createKonobanguBootstrapEnvironment({
    documentOrigin: options.documentOrigin ?? "http://localhost",
    fetch: options.fetch ?? (async () => new Response(null, { status: 503 })),
    ensurePolyfills: options.ensurePolyfills ?? (async () => undefined),
    deadlineMs: options.deadlineMs ?? 10_000,
    allowInsecureLoopback: options.allowInsecureLoopback ?? true,
  });
}
