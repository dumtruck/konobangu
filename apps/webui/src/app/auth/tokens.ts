import {
  SecuritydeptInjectionToken,
  type SecuritydeptInjectorTrait,
} from "@securitydept/client";
import type { BaseOidcModeClient } from "@securitydept/token-set-context-client/orchestration";
import {
  TOKEN_SET_CLIENT_REGISTRY,
  type TokenSetClientRegistry,
  type TokenSetClientRegistryEntry,
} from "@securitydept/token-set-context-client/registry";
import type { AuthSetup } from "./setup";

/** The published mode this composition root was built for. */
export const AUTH_SETUP = new SecuritydeptInjectionToken<AuthSetup>(
  "AUTH_SETUP",
);

/**
 * Rebuilds the single OIDC registry entry. Signing out unregisters the client
 * so the registry disposes it, then registers the same lazy entry again.
 */
export type OidcClientEntryBuilder =
  () => TokenSetClientRegistryEntry<BaseOidcModeClient>;

export const OIDC_CLIENT_ENTRY =
  new SecuritydeptInjectionToken<OidcClientEntryBuilder>("OIDC_CLIENT_ENTRY");

export function tokenSetRegistryFromInjector(
  injector: SecuritydeptInjectorTrait,
): TokenSetClientRegistry {
  // `TOKEN_SET_CLIENT_REGISTRY` is typed `any` in v0.3.0, so the generic on
  // `get` carries the type instead of a cast on the result. Upstream typing
  // gap: the token should be a `SecuritydeptInjectionToken<TokenSetClientRegistry>`.
  return injector.get<TokenSetClientRegistry>(TOKEN_SET_CLIENT_REGISTRY);
}
