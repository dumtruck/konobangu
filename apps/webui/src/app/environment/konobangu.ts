import {
  type CreateFoundationEnvironmentOptions,
  createBaseTransportForStdFetch,
  createSecuritydeptDestroyRef,
  type FoundationEnvironment,
  INJECTOR_TOKEN,
  ROUTER_TRAIT_TOKEN,
  type RouterTrait,
  SecuritydeptDestroyRef,
  type SecuritydeptInjectorTrait,
  type SecuritydeptProvider,
  UriString,
} from "@securitydept/client";
import { createRouterForTanStackRouter } from "@securitydept/client-react/tanstack-router";
import type { AnyRouter } from "@tanstack/react-router";
import { provideAuth } from "@/app/auth/context";
import type { AuthSetup } from "@/app/auth/setup";
import { ApiTarget, provideRuntimeConfig } from "@/app/config/context";
import type { RuntimeConfig } from "@/app/config/runtime";
import { APP_LIFETIME, AppLifetime } from "@/app/lifetime/scope";
import { provideResourceService } from "@/app/resource/resource-grants";
import { AuthService } from "@/domains/auth/auth.service";
import { provideRecorder } from "@/domains/recorder/context";
import { provideGraphql } from "@/infra/graphql/context";
import { HTTP_CLIENT, HttpClient } from "@/infra/http/http-client";
import { provideIntl } from "@/infra/intl/context";
import { providePlatform } from "@/infra/platform/context";
import { provideStyles } from "@/infra/styles/context";
import {
  APP_ROUTER,
  DOCUMENT,
  FETCH,
  type FetchTrait,
  LOCALE,
  type MediaQueryTrait,
  MOBILE_VIEWPORT,
  PAGE_VISIBILITY,
  type PageVisibilityTrait,
  PLATFORM_INFO,
  type PlatformInfoTrait,
  RESOURCE_CRYPTO,
  type ResourceCryptoTrait,
  SYSTEM_DARK_THEME,
  UI_PREFERENCE_STORAGE,
  UI_SESSION_STORAGE,
  type UiStorageTrait,
  VIEWPORT,
  type ViewportTrait,
} from "./traits";

/** The traits a host resolves; everything else here is host-independent. */
export interface KonobanguTraits {
  document: Document;
  fetch: FetchTrait;
  locale: string;
  platformInfo: PlatformInfoTrait;
  systemDarkTheme: MediaQueryTrait;
  mobileViewport: MediaQueryTrait;
  uiPreferenceStorage: UiStorageTrait;
  uiSessionStorage: UiStorageTrait;
  pageVisibility: PageVisibilityTrait;
  viewport: ViewportTrait;
  resourceCrypto: ResourceCryptoTrait;
}

/**
 * The single environment of this application. Platform differences reach
 * services only through it, so a host is described by the options of its
 * `createKonobanguEnvironmentFor{Host}` and nothing else. It is also the unique
 * owner: disposing it ends the application lifetime and the SDK resources,
 * while the services it lends never expose disposal.
 */
export interface KonobanguEnvironment
  extends FoundationEnvironment,
    KonobanguTraits,
    Disposable {
  router: RouterTrait;
  api: ApiTarget;
  documentOrigin: string;
  appRouter: AnyRouter;
}

type SdkOptions = Omit<
  CreateFoundationEnvironmentOptions,
  | "transport"
  | "transportForStdFetchCreateOptions"
  | "router"
  | "persistentStorage"
>;

export interface CreateKonobanguEnvironmentOptions<
  TTraits extends KonobanguTraits = KonobanguTraits,
> extends SdkOptions {
  /** The SDK host stack this application layer is created over. */
  createBaseEnvironment: (
    options: CreateFoundationEnvironmentOptions,
  ) => FoundationEnvironment;
  traits: TTraits;
  config: RuntimeConfig;
  auth: AuthSetup;
  documentOrigin: string;
  appRouter: AnyRouter;
}

function createSdkRouter(appRouter: AnyRouter, documentOrigin: string) {
  return createRouterForTanStackRouter({
    router: {
      get state() {
        return appRouter.state;
      },
      navigate(options) {
        const absolute =
          typeof options.to === "string"
            ? UriString.tryParse(options.to)
            : null;
        if (absolute) {
          const target = absolute.toURL();
          if (
            target.origin !== documentOrigin ||
            target.username ||
            target.password
          )
            throw new Error(
              "Internal SDK navigation must remain on the configured WebUI origin",
            );
          return appRouter.navigate({
            ...options,
            to: `${target.pathname}${target.search}${target.hash}`,
          });
        }
        return appRouter.navigate(options);
      },
    },
    get currentUrl() {
      return new URL(appRouter.state.location.href, documentOrigin).toString();
    },
  });
}

export function createKonobanguEnvironment<TTraits extends KonobanguTraits>(
  options: CreateKonobanguEnvironmentOptions<TTraits>,
): KonobanguEnvironment & TTraits {
  const {
    createBaseEnvironment,
    traits,
    config,
    auth,
    documentOrigin,
    appRouter,
    providers = [],
    ...sdkOptions
  } = options;
  // A failed construction releases whatever its providers already acquired.
  using rollback = new DisposableStack();
  const destroy = rollback.use(createSecuritydeptDestroyRef());
  const lifetime = rollback.use(new AppLifetime());
  const api = new ApiTarget(config, documentOrigin);
  // Protocol requests use the SDK authentication boundary.
  const protocolFetch: FetchTrait = (input, init) => {
    const request = new Request(input, init);
    return traits.fetch(request, {
      credentials: "omit",
      redirect: "error",
      referrerPolicy: "origin",
    });
  };
  const sdkRouter = createSdkRouter(appRouter, documentOrigin);
  const environmentProviders: SecuritydeptProvider[] = [
    { provide: SecuritydeptDestroyRef, useValue: destroy },
    { provide: APP_LIFETIME, useValue: lifetime },
    { provide: DOCUMENT, useValue: traits.document },
    { provide: FETCH, useValue: traits.fetch },
    { provide: LOCALE, useValue: traits.locale },
    { provide: PLATFORM_INFO, useValue: traits.platformInfo },
    {
      provide: SYSTEM_DARK_THEME,
      useValue: traits.systemDarkTheme,
    },
    { provide: MOBILE_VIEWPORT, useValue: traits.mobileViewport },
    {
      provide: UI_PREFERENCE_STORAGE,
      useValue: traits.uiPreferenceStorage,
    },
    {
      provide: UI_SESSION_STORAGE,
      useValue: traits.uiSessionStorage,
    },
    { provide: PAGE_VISIBILITY, useValue: traits.pageVisibility },
    { provide: VIEWPORT, useValue: traits.viewport },
    { provide: RESOURCE_CRYPTO, useValue: traits.resourceCrypto },
    {
      provide: HTTP_CLIENT,
      useFactory: (
        lifetime: AppLifetime,
        fetch: FetchTrait,
        injector: SecuritydeptInjectorTrait,
      ) =>
        new HttpClient({
          api,
          network: fetch,
          rootSignal: lifetime.signal,
          authorization: () => injector.get(AuthService, null),
        }),
      deps: [APP_LIFETIME, FETCH, INJECTOR_TOKEN],
    },
    {
      provide: ROUTER_TRAIT_TOKEN,
      useValue: sdkRouter,
    },
    {
      provide: APP_ROUTER,
      useFactory: (router: RouterTrait) => router,
      deps: [ROUTER_TRAIT_TOKEN],
    },
    ...provideRuntimeConfig(config, api),
    ...providePlatform(),
    ...provideAuth(api, auth, documentOrigin),
    ...provideStyles(),
    ...provideGraphql(),
    ...provideRecorder(),
    ...provideIntl(),
    provideResourceService(),
  ];
  const base = createBaseEnvironment({
    ...sdkOptions,
    transport: createBaseTransportForStdFetch({ fetch: protocolFetch }),
    // Tokens and configuration never outlive the browsing session.
    persistentStorage: null,
    // Caller providers come last so a host or a test can replace any unit.
    providers: [...environmentProviders, ...providers],
  });
  const router = base.injector.get(APP_ROUTER);
  const ownership = rollback.move();
  return {
    ...base,
    ...traits,
    router,
    api,
    documentOrigin,
    appRouter,
    [Symbol.dispose]: () => ownership.dispose(),
  };
}
