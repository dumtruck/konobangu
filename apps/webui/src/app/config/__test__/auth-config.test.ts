import { describe, expect, it } from "vitest";
import {
  loadPublicAuthConfig,
  PublicAuthConfigError,
  parsePublicAuthConfig,
} from "../auth-config";

const projection = {
  clientId: "konobangu-webui",
  issuerUrl: "https://idp.example",
  redirectUrl: "https://ui.konobangu.test/auth/callback",
  pkceEnabled: true,
  generatedAt: 1,
};
const oidc = {
  provider: "oidc",
  resource: {
    audience: "urn:konobangu:api",
    requiredScopes: ["konobangu:api"],
  },
  ...projection,
};

describe("the published authentication configuration", () => {
  it("splits this project's envelope from the SDK projection", () => {
    const parsed = parsePublicAuthConfig(oidc);
    expect(parsed.provider).toBe("oidc");
    if (parsed.provider !== "oidc") throw new Error("unreachable");
    expect(parsed.resource.audience).toBe("urn:konobangu:api");
    // The projection is handed to the SDK verbatim, envelope keys removed.
    expect(parsed.projection).toEqual(projection);
  });

  it("refuses the removed direct Basic API mode", () => {
    expect(() => parsePublicAuthConfig({ provider: "basic" })).toThrow(
      PublicAuthConfigError,
    );
    expect(() => parsePublicAuthConfig({ provider: "bundled" })).toThrow(
      PublicAuthConfigError,
    );
  });

  it.each([
    [{}, "provider must be"],
    [{ provider: "cookie" }, "provider must be"],
    [{ provider: "oidc" }, "resource must be an object"],
    [{ provider: "oidc", resource: {} }, "resource.requiredScopes"],
    [
      { provider: "oidc", resource: { audience: "a", requiredScopes: [] } },
      "resource.requiredScopes must be non-empty",
    ],
    [
      { provider: "oidc", resource: { audience: " ", requiredScopes: ["a"] } },
      "resource.audience must be a non-blank string",
    ],
    [[], "provider must be"],
    [null, "must be an object"],
  ])("rejects %j", (payload, message) => {
    expect(() => parsePublicAuthConfig(payload)).toThrow(message as string);
  });

  it("delegates configuration reads to its provided client", async () => {
    const signal = new AbortController().signal;
    const seen: URL[] = [];
    const result = await loadPublicAuthConfig({
      apiBaseUrl: new URL("https://api.example.net/api/"),
      signal,
      client: {
        getJson: async (url, suppliedSignal) => {
          seen.push(url);
          expect(suppliedSignal).toBe(signal);
          return oidc;
        },
      },
    });
    expect(seen).toHaveLength(1);
    expect(result).toBeTruthy();
  });
});
