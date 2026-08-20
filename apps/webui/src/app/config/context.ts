import { UriRelativeString } from "@securitydept/client";
import { InjectionToken } from "injection-js";
import type { RuntimeConfig } from "./runtime";

/** The resolved sidecar, bound before any auth client or transport is created. */
export const RUNTIME_CONFIG = new InjectionToken<RuntimeConfig>(
  "RUNTIME_CONFIG",
);

/**
 * The single validated API destination. Every API client, private resource
 * request and endpoint path goes through this, so no caller keeps its own
 * relative "/api" assumption that would break a standalone deployment.
 */
export class ApiTarget {
  /** Always absolute, so no caller re-resolves it against an ambient origin. */
  private readonly destination: string;

  get baseUrl(): URL {
    return new URL(this.destination);
  }

  constructor(config: RuntimeConfig, documentOrigin: string) {
    this.destination = new URL(config.apiBaseUrl, documentOrigin).toString();
  }

  /** Resolves an endpoint path relative to the API base, e.g. "graphql". */
  url(path: string): string {
    if (path.startsWith("/"))
      throw new Error(
        `API endpoint paths are relative to the API base: ${path}`,
      );
    const target = UriRelativeString.parse(path).toURL(this.baseUrl);
    if (!this.owns(target) || target.username || target.password || target.hash)
      throw new Error("Request escapes the configured API destination");
    return target.toString();
  }

  /** True when a URL addresses this API, used to classify its responses. */
  owns(url: string | URL): boolean {
    let target: URL;
    try {
      target = new URL(url, this.baseUrl);
    } catch {
      return false;
    }
    return (
      target.origin === this.baseUrl.origin &&
      target.pathname.startsWith(this.baseUrl.pathname)
    );
  }
}

export function provideRuntimeConfig(config: RuntimeConfig, api: ApiTarget) {
  return [
    { provide: RUNTIME_CONFIG, useValue: config },
    { provide: ApiTarget, useValue: api },
  ];
}
