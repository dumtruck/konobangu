import type { SecuritydeptProvider } from "@securitydept/client";
import {
  createFrontendOidcModeClientFactory,
  FrontendOidcModeConfigProjectionSourceKind,
  resolveFrontendOidcModeConfigProjection,
} from "@securitydept/token-set-context-client/frontend-oidc-mode";
import {
  type BaseOidcModeClient,
  TokenSetRefreshErrorPolicy,
} from "@securitydept/token-set-context-client/orchestration";
import {
  provideTokenSetClientRegistry,
  TokenSetClientInitializationMode,
  type TokenSetClientRegistryEntry,
} from "@securitydept/token-set-context-client/registry";
import { PUBLIC_AUTH_CONFIG_PATH } from "@/app/config/auth-config";
import type { ApiTarget } from "@/app/config/context";
import { APP_LIFETIME, type AppLifetime } from "@/app/lifetime/scope";
import { AuthService } from "@/domains/auth/auth.service";
import { type AuthSetup, callbackUrlForOrigin } from "./setup";
import { AUTH_SETUP, OIDC_CLIENT_ENTRY } from "./tokens";

function oidcProviders(
  api: ApiTarget,
  setup: AuthSetup,
  documentOrigin: string,
): SecuritydeptProvider[] {
  const publicConfigUrl = api.url(PUBLIC_AUTH_CONFIG_PATH);
  const callbackUrl = callbackUrlForOrigin(documentOrigin);
  const buildEntry = (): TokenSetClientRegistryEntry<BaseOidcModeClient> => ({
    meta: {
      clientKey: setup.clientKey,
      // Exactly this API, and never the public configuration: matching it
      // would make reading the login mode initialize a client.
      urlPatterns: [(url: string) => api.owns(url) && url !== publicConfigUrl],
      callbackUrl,
      initialization: TokenSetClientInitializationMode.Lazy,
    },
    clientFactory: createFrontendOidcModeClientFactory({
      config: async ({ environment, cancellationToken }) => {
        // Realm first: bootstrap already read and validated the published
        // projection, so a valid realm never requests it again. Persisted is
        // skipped automatically while persistentStorage is null, leaving no
        // local copy of the configuration.
        const resolved = await resolveFrontendOidcModeConfigProjection({
          clientKey: setup.clientKey,
          environment,
          sources: [
            {
              kind: FrontendOidcModeConfigProjectionSourceKind.Realm,
              realm: setup.realm,
            },
            { kind: FrontendOidcModeConfigProjectionSourceKind.Persisted },
            {
              kind: FrontendOidcModeConfigProjectionSourceKind.Network,
              endpoint: publicConfigUrl,
            },
          ],
          cancellationToken,
        });
        return {
          ...resolved.config,
          // Only an IdP-confirmed invalid grant returns to a local
          // unauthenticated state; no revocation request is ever sent.
          refreshErrorPolicy:
            TokenSetRefreshErrorPolicy.RevokeAsUnauthenticated,
          // No persistent storage exists, so this key never writes; it stays
          // stable so one deployment keeps one client identity.
          persistence: { key: setup.clientKey },
        };
      },
    }),
  });
  return [
    { provide: OIDC_CLIENT_ENTRY, useValue: buildEntry },
    ...provideTokenSetClientRegistry({ clients: [buildEntry()] }),
  ];
}

/**
 * Both bundled and external issuers publish the same SDK projection, so the
 * browser has one TokenSet composition regardless of server deployment.
 */
export function provideAuth(
  api: ApiTarget,
  setup: AuthSetup,
  documentOrigin: string,
): SecuritydeptProvider[] {
  return [
    { provide: AUTH_SETUP, useValue: setup },
    ...oidcProviders(api, setup, documentOrigin),
    {
      provide: AuthService,
      useFactory: (lifetime: AppLifetime) => AuthService.createOwned(lifetime),
      deps: [APP_LIFETIME],
    },
  ];
}
