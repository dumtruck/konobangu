import { throwValidationClientError } from "@securitydept/client";
import {
  FrontendOidcModeConfigErrorCode,
  FrontendOidcModeConfigErrorSource,
  type FrontendOidcModeConfigProjection,
  injectConfigProjectionIntoRealm,
  validateConfigProjection,
} from "@securitydept/token-set-context-client/frontend-oidc-mode";
import type { PublicAuthConfig } from "@/app/config/auth-config";
import {
  type AuthSetup,
  assertHostAuthPolicy,
  callbackUrlForOrigin,
  OIDC_CLIENT_KEY,
} from "./setup";

/**
 * Runs after the capability polyfills, so importing the SDK here is safe: its
 * resource-management compat decides the dispose symbol at module load.
 *
 * The projection structure and the source precedence belong to the SDK; only
 * the provider/resource envelope and this host's safety policy belong here.
 * The validated projection is injected into an explicit realm object owned by
 * this bootstrap, never `globalThis`, so a second root cannot pick it up and a
 * failed start leaves nothing behind.
 */
/**
 * The one place the SDK's validation result is adapted.
 *
 * `validateConfigProjection` is typed `any` in v0.3.0 even though it returns
 * the SDK's own `ValidationResult`, so the shape is checked here rather than
 * asserted at a consumer, and the SDK keeps ownership of the schema. Upstream
 * typing gap: the public signature should return
 * `ValidationResult<FrontendOidcModeConfigProjection>`.
 */
function validateProjection(
  projection: Record<string, unknown>,
): FrontendOidcModeConfigProjection {
  const result: unknown = validateConfigProjection(projection);
  if (
    typeof result === "object" &&
    result !== null &&
    "success" in result &&
    result.success === true &&
    "value" in result
  )
    return result.value as FrontendOidcModeConfigProjection;
  throwValidationClientError({
    code: FrontendOidcModeConfigErrorCode.InvalidProjection,
    source: FrontendOidcModeConfigErrorSource,
    messagePrefix: "The published OIDC configuration is not a valid projection",
    failure:
      typeof result === "object" &&
      result !== null &&
      "issues" in result &&
      Array.isArray(result.issues)
        ? { success: false, issues: result.issues }
        : undefined,
  });
}

export function prepareAuthSetup(
  config: PublicAuthConfig,
  options: { documentOrigin: string },
): AuthSetup {
  const callbackUrl = callbackUrlForOrigin(options.documentOrigin);
  assertHostAuthPolicy(config.projection, { callbackUrl });
  const realm = {};
  injectConfigProjectionIntoRealm({
    clientKey: OIDC_CLIENT_KEY,
    projection: validateProjection(config.projection),
    realm,
  });
  return {
    clientKey: OIDC_CLIENT_KEY,
    resource: config.resource,
    realm,
  };
}
