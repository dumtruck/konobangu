import { describe, expect, it } from "vitest";
import { ApiTarget } from "../context";
import {
  loadRuntimeConfig,
  parseRuntimeConfig,
  RuntimeConfigError,
} from "../runtime";

const origin = "https://ui.konobangu.test";
const strict = { origin, allowInsecureLoopback: false };
const dev = { origin, allowInsecureLoopback: true };

describe("runtime configuration sidecar", () => {
  it("accepts a same-origin base and a reverse proxy prefix", () => {
    expect(
      parseRuntimeConfig({ version: 1, apiBaseUrl: "/api/" }, strict)
        .apiBaseUrl,
    ).toBe("/api/");
    // A missing trailing slash would silently drop the last path segment.
    expect(
      parseRuntimeConfig({ version: 1, apiBaseUrl: "/konobangu/api" }, strict)
        .apiBaseUrl,
    ).toBe("/konobangu/api/");
  });

  it("accepts an exact absolute HTTPS API on an unrelated site", () => {
    expect(
      parseRuntimeConfig(
        { version: 1, apiBaseUrl: "https://api.recorder.test/api" },
        strict,
      ).apiBaseUrl,
    ).toBe("https://api.recorder.test/api/");
  });

  it("allows loopback HTTP only for development", () => {
    expect(
      parseRuntimeConfig(
        { version: 1, apiBaseUrl: "http://127.0.0.1:5001/api/" },
        dev,
      ).apiBaseUrl,
    ).toBe("http://127.0.0.1:5001/api/");
    expect(() =>
      parseRuntimeConfig(
        { version: 1, apiBaseUrl: "http://127.0.0.1:5001/api/" },
        strict,
      ),
    ).toThrow(RuntimeConfigError);
  });

  it("accepts explicitly selected LAN HTTP only from an HTTP WebUI", () => {
    const value = {
      version: 1,
      apiBaseUrl: "http://192.168.1.50:5001/prefix/api/",
    };
    expect(
      parseRuntimeConfig(value, {
        origin: "http://192.168.1.50:5000",
        allowInsecureLoopback: false,
      }).apiBaseUrl,
    ).toBe(value.apiBaseUrl);
    expect(() => parseRuntimeConfig(value, strict)).toThrow("mixed content");
  });

  it.each([
    ["remote HTTP in development", "http://api.recorder.test/api/", dev],
    ["protocol relative", "//api.recorder.test/api/", strict],
    ["non HTTP scheme", "ftp://api.recorder.test/api/", strict],
    ["user information", "https://user:pw@api.recorder.test/api/", strict],
    ["a query", "https://api.recorder.test/api/?tenant=a", strict],
    ["a fragment", "https://api.recorder.test/api/#x", strict],
    ["a bare host", "api.recorder.test/api", strict],
    ["an empty value", "", strict],
  ])("refuses to guess an API from %s", (_label, apiBaseUrl, options) => {
    expect(() =>
      parseRuntimeConfig({ version: 1, apiBaseUrl }, options),
    ).toThrow(RuntimeConfigError);
  });

  it.each([
    ["an unsupported version", { version: 2, apiBaseUrl: "/api/" }],
    ["a missing version", { apiBaseUrl: "/api/" }],
    ["a missing base", { version: 1 }],
    ["an extra field", { version: 1, apiBaseUrl: "/api/", authMode: "basic" }],
    ["a non-object", [{ version: 1, apiBaseUrl: "/api/" }]],
    ["a non-string base", { version: 1, apiBaseUrl: 5 }],
  ])("rejects %s", (_label, payload) => {
    expect(() => parseRuntimeConfig(payload, strict)).toThrow(
      RuntimeConfigError,
    );
  });

  it("does not allow borrowed URLs to change the configured API", () => {
    const api = new ApiTarget({ version: 1, apiBaseUrl: "/api/" }, origin);
    const borrowed = api.baseUrl;
    borrowed.hostname = "attacker.example";
    borrowed.pathname = "/other/";
    expect(api.url("graphql")).toBe(`${origin}/api/graphql`);
    expect(api.owns(borrowed)).toBe(false);
  });
  it("delegates configuration reads to its provided client", async () => {
    const signal = new AbortController().signal;
    const seen: URL[] = [];
    const result = await loadRuntimeConfig({
      ...strict,
      signal,
      client: {
        getJson: async (url, suppliedSignal) => {
          seen.push(url);
          expect(suppliedSignal).toBe(signal);
          return { version: 1, apiBaseUrl: "/api/" };
        },
      },
    });
    expect(seen).toHaveLength(1);
    expect(result).toBeTruthy();
  });
});
