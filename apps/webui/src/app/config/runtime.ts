/**
 * The published runtime configuration sidecar. Deployments mount or edit this
 * file to point the same prebuilt bundle at their API, so it is fetched before
 * the application modules load and never contains an auth mode, token or
 * secret. An unreadable or invalid file is a hard failure: guessing an API
 * origin would send credentials somewhere the operator did not choose.
 */
export const RUNTIME_CONFIG_PATH = "/konobangu.config.json";
export const RUNTIME_CONFIG_VERSION = 1;

export type RuntimeConfig = {
  readonly version: typeof RUNTIME_CONFIG_VERSION;
  /** Always ends with "/" so endpoint paths append without rejoining rules. */
  readonly apiBaseUrl: string;
};

const ALLOWED_KEYS = new Set(["version", "apiBaseUrl"]);

export class RuntimeConfigError extends Error {
  constructor(reason: string) {
    super(`Invalid ${RUNTIME_CONFIG_PATH}: ${reason}`);
    this.name = "RuntimeConfigError";
  }
}

function assertNoCredentialsOrExtras(url: URL, source: string) {
  if (url.username || url.password)
    throw new RuntimeConfigError(
      `apiBaseUrl must not carry user information (${source})`,
    );
  if (url.search)
    throw new RuntimeConfigError(
      `apiBaseUrl must not carry a query (${source})`,
    );
  if (url.hash)
    throw new RuntimeConfigError(
      `apiBaseUrl must not carry a fragment (${source})`,
    );
}

function isLoopbackHost(hostname: string) {
  return (
    hostname === "localhost" ||
    hostname === "127.0.0.1" ||
    hostname === "[::1]" ||
    hostname === "::1" ||
    hostname.endsWith(".localhost")
  );
}

/**
 * Accepts a same-origin absolute path, including a reverse proxy prefix, or an
 * exact absolute HTTPS URL. An HTTP WebUI may explicitly select an HTTP API; HTTPS pages reject mixed content.
 */
export function resolveApiBaseUrl(
  value: unknown,
  options: { origin: string; allowInsecureLoopback: boolean },
): string {
  if (typeof value !== "string" || value.length === 0)
    throw new RuntimeConfigError("apiBaseUrl must be a non-empty string");
  if (value.startsWith("//"))
    throw new RuntimeConfigError("apiBaseUrl must not be protocol relative");

  const absolute = /^[a-zA-Z][a-zA-Z\d+\-.]*:/.test(value);
  if (!absolute) {
    if (!value.startsWith("/"))
      throw new RuntimeConfigError('a relative apiBaseUrl must start with "/"');
    const url = new URL(value, options.origin);
    assertNoCredentialsOrExtras(url, "relative path");
    return url.pathname.endsWith("/") ? url.pathname : `${url.pathname}/`;
  }

  let url: URL;
  try {
    url = new URL(value);
  } catch {
    throw new RuntimeConfigError("apiBaseUrl is not a valid URL");
  }
  if (url.protocol !== "https:" && url.protocol !== "http:")
    throw new RuntimeConfigError(
      `apiBaseUrl must use http(s), got ${url.protocol}`,
    );
  assertNoCredentialsOrExtras(url, "absolute URL");
  if (
    url.protocol === "http:" &&
    new URL(options.origin).protocol !== "http:" &&
    !(options.allowInsecureLoopback && isLoopbackHost(url.hostname))
  )
    throw new RuntimeConfigError(
      "An HTTPS WebUI cannot use an HTTP API (mixed content)",
    );
  const base = url.origin + url.pathname;
  return base.endsWith("/") ? base : `${base}/`;
}

export function parseRuntimeConfig(
  value: unknown,
  options: { origin: string; allowInsecureLoopback: boolean },
): RuntimeConfig {
  if (typeof value !== "object" || value === null || Array.isArray(value))
    throw new RuntimeConfigError("expected a JSON object");
  const unknown = Object.keys(value).filter((key) => !ALLOWED_KEYS.has(key));
  if (unknown.length)
    throw new RuntimeConfigError(
      `unknown field(s): ${unknown.sort().join(", ")}`,
    );
  const record = value as Record<string, unknown>;
  if (record.version !== RUNTIME_CONFIG_VERSION)
    throw new RuntimeConfigError(
      `unsupported version, expected ${RUNTIME_CONFIG_VERSION}`,
    );
  return Object.freeze({
    version: RUNTIME_CONFIG_VERSION,
    apiBaseUrl: resolveApiBaseUrl(record.apiBaseUrl, options),
  });
}

export async function loadRuntimeConfig(options: {
  origin: string;
  allowInsecureLoopback: boolean;
  client: import("../environment/bootstrap").BootstrapConfigClient;
  signal: AbortSignal;
}): Promise<RuntimeConfig> {
  return parseRuntimeConfig(
    await options.client.getJson(
      new URL(RUNTIME_CONFIG_PATH, options.origin),
      options.signal,
    ),
    options,
  );
}
