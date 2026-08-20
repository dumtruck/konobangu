/**
 * The environment that exists before the SDK and its polyfills load. It follows
 * the SDK naming (`createXEnvironmentFor{Host}`) but must not import the SDK:
 * its resource-management compat fixes the dispose symbol at module load.
 */
export interface BootstrapConfigClient {
  getJson(url: URL, signal: AbortSignal): Promise<unknown>;
}
export interface KonobanguBootstrapEnvironment {
  readonly documentOrigin: string;
  readonly configClient: BootstrapConfigClient;
  readonly ensurePolyfills: (signal: AbortSignal) => Promise<void>;
  readonly deadlineMs: number;
  readonly allowInsecureLoopback: boolean;
}
export interface CreateKonobanguBootstrapEnvironmentOptions {
  documentOrigin: string;
  fetch: typeof fetch;
  ensurePolyfills: (signal: AbortSignal) => Promise<void>;
  deadlineMs: number;
  allowInsecureLoopback: boolean;
}
export interface CreateKonobanguBootstrapEnvironmentForNativeWebOptions
  extends Omit<
    CreateKonobanguBootstrapEnvironmentOptions,
    "documentOrigin" | "fetch"
  > {
  window: {
    readonly fetch?: typeof fetch;
    readonly location: { readonly origin: string };
  };
}

export class MissingBootstrapTraitError extends Error {
  constructor(trait: string) {
    super(`This browser does not provide the required ${trait}.`);
    this.name = "MissingBootstrapTraitError";
  }
}

export function createKonobanguBootstrapEnvironment(
  options: CreateKonobanguBootstrapEnvironmentOptions,
): KonobanguBootstrapEnvironment {
  const { fetch: request, ...environment } = options;
  return {
    ...environment,
    configClient: {
      async getJson(url, signal) {
        const response = await request(url, {
          credentials: "omit",
          cache: "no-store",
          redirect: "error",
          referrerPolicy: "origin",
          signal,
        });
        if (!response.ok)
          throw new Error(`Configuration request failed (${response.status})`);
        return response.json();
      },
    },
  };
}

export function createKonobanguBootstrapEnvironmentForNativeWeb(
  options: CreateKonobanguBootstrapEnvironmentForNativeWebOptions,
): KonobanguBootstrapEnvironment {
  const { window: view, ...baseOptions } = options;
  if (typeof view.fetch !== "function")
    throw new MissingBootstrapTraitError("fetch");
  return createKonobanguBootstrapEnvironment({
    ...baseOptions,
    documentOrigin: view.location.origin,
    fetch: view.fetch.bind(view),
  });
}
