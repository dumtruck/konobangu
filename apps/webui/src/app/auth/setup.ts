import type { PublicResourceConfig } from "@/app/config/auth-config";

/**
 * The WebUI is hosted at its origin root, so the callback, the runtime config
 * sidecar, the router, the assets and the private virtual paths share one
 * location. The server validates the same path.
 */
export const AUTH_CALLBACK_PATH = "/auth/callback";

/** One registry entry per deployment; also the config-projection realm key. */
export const OIDC_CLIENT_KEY = "konobangu-webui";

/**
 * What bootstrap hands the composition root. For OIDC the validated projection
 * is already injected into `realm`, so the client factory resolves it from the
 * Realm source without a second network read.
 */
export type AuthSetup = {
  readonly clientKey: string;
  readonly resource: PublicResourceConfig;
  readonly realm: object;
};

export class AuthPolicyError extends Error {
  constructor(reason: string) {
    super(`Rejected the published OIDC configuration: ${reason}`);
    this.name = "AuthPolicyError";
  }
}

/**
 * The SDK schema permits a client secret, a claims-check script and disabled
 * PKCE, so passing structural validation is not the same as passing this host's
 * safety policy. The server enforces these too; a browser that trusted the
 * response alone would accept whatever answered the API address.
 */
export function assertHostAuthPolicy(
  projection: Record<string, unknown>,
  options: { callbackUrl: string },
): void {
  if (projection.clientSecret !== undefined)
    throw new AuthPolicyError("a browser OIDC client must be public");
  if (projection.claimsCheckScript !== undefined)
    throw new AuthPolicyError("an executable claims-check script is refused");
  if (projection.pkceEnabled !== true)
    throw new AuthPolicyError("PKCE is required for the browser code flow");
  for (const field of ["clientId", "issuerUrl"] as const) {
    const value = projection[field];
    if (typeof value !== "string" || value.trim().length === 0)
      throw new AuthPolicyError(`${field} must be a non-empty string`);
  }
  if (projection.redirectUrl !== options.callbackUrl)
    throw new AuthPolicyError(
      `the published callback must be ${options.callbackUrl}`,
    );
}

export function callbackUrlForOrigin(origin: string): string {
  return new URL(AUTH_CALLBACK_PATH, origin).toString();
}
