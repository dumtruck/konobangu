/**
 * The public authentication configuration the browser reads before any auth
 * client exists. It is fetched during bootstrap, so this module must stay free
 * of SDK imports: the SDK's resource-management compat decides its dispose
 * symbol at module load, which must not happen before the polyfills. The
 * schema library the SDK uses has no such constraint.
 *
 * For OIDC the server flattens the SecurityDept config projection at the
 * response root and adds only this project's `provider` and `resource`
 * envelope, so the projection is handed to the SDK verbatim.
 */
import { type } from "arktype";

export const PUBLIC_AUTH_CONFIG_PATH = "auth/config";

// A value of whitespace alone names nothing.
const Name = type(/\S/).describe("a non-blank string");
const Resource = type({
  audience: Name,
  requiredScopes: Name.array().atLeastLength(1),
});
/**
 * Only this project's envelope is described. Every other key is the SDK
 * projection, whose structure is the SDK's
 * contract and is validated there.
 */
const Envelope = type({
  provider: '"oidc"',
  resource: Resource,
});

export type PublicResourceConfig = {
  readonly audience: string;
  readonly requiredScopes: readonly string[];
};

export type PublicAuthConfig = {
  readonly provider: "oidc";
  readonly resource: PublicResourceConfig;
  /** The SDK projection, unmodified; its structure is the SDK's contract. */
  readonly projection: Record<string, unknown>;
};

export class PublicAuthConfigError extends Error {
  constructor(reason: string) {
    super(`Invalid ${PUBLIC_AUTH_CONFIG_PATH} response: ${reason}`);
    this.name = "PublicAuthConfigError";
  }
}

/** Splits this project's envelope from the SDK projection. */
export function parsePublicAuthConfig(value: unknown): PublicAuthConfig {
  const envelope = Envelope(value);
  if (envelope instanceof type.errors)
    throw new PublicAuthConfigError(envelope.summary);
  const { provider, resource, ...projection } = envelope;
  return Object.freeze({
    provider,
    resource: Object.freeze({
      audience: resource.audience,
      requiredScopes: Object.freeze([...resource.requiredScopes]),
    }),
    projection: Object.freeze(projection),
  });
}

export async function loadPublicAuthConfig(options: {
  apiBaseUrl: URL;
  client: import("../environment/bootstrap").BootstrapConfigClient;
  signal: AbortSignal;
}): Promise<PublicAuthConfig> {
  return parsePublicAuthConfig(
    await options.client.getJson(
      new URL(PUBLIC_AUTH_CONFIG_PATH, options.apiBaseUrl),
      options.signal,
    ),
  );
}
