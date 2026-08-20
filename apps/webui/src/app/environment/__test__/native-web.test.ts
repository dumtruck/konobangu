// @vitest-environment jsdom

import { describe, expect, it, vi } from "vitest";
import { stubMatchMedia } from "@/__test__/support/match-media";
import { parseRuntimeConfig } from "@/app/config/runtime";
import { createKonobanguBootstrapEnvironmentForNativeWeb } from "../bootstrap";
import { createKonobanguEnvironmentForNativeWeb } from "../native-web";
import { createOidcAuthSetupForTest, createRouterForMemory } from "../test";

describe("native web environment creation", () => {
  it("provides a narrow pre-polyfill JSON client with credential-free policy", async () => {
    const network = vi.fn(
      async (_input: RequestInfo | URL, _init?: RequestInit) =>
        new Response('{"ok":true}'),
    );
    const environment = createKonobanguBootstrapEnvironmentForNativeWeb({
      window: { fetch: network, location: { origin: "https://ui.example" } },
      ensurePolyfills: async () => undefined,
      deadlineMs: 10,
      allowInsecureLoopback: false,
    });
    expect(environment.documentOrigin).toBe("https://ui.example");
    expect(
      await environment.configClient.getJson(
        new URL("https://ui.example/config"),
        new AbortController().signal,
      ),
    ).toEqual({ ok: true });
    expect(network.mock.calls[0]?.[1]).toMatchObject({
      credentials: "omit",
      redirect: "error",
      cache: "no-store",
      referrerPolicy: "origin",
    });
    expect("fetch" in environment).toBe(false);
  });
  it("uses one memory store and its own events when native persistence is denied", () => {
    stubMatchMedia();
    vi.spyOn(window, "localStorage", "get").mockImplementation(() => {
      throw new Error("denied");
    });
    using environment = createKonobanguEnvironmentForNativeWeb({
      window,
      config: parseRuntimeConfig(
        { version: 1, apiBaseUrl: "/api/" },
        { origin: window.location.origin, allowInsecureLoopback: true },
      ),
      auth: createOidcAuthSetupForTest(window.location.origin),
      appRouter: createRouterForMemory(),
    });
    expect(environment.documentOrigin).toBe(window.location.origin);
    expect(environment.document).toBe(document);
    const store = environment.uiPreferenceStorage;
    const values: unknown[] = [];
    const subscription = store.storageEvent.subscribe({
      next: (value: unknown) => values.push(value),
    });
    store.set("theme", "dark");
    store.remove("theme");
    expect(store.get("theme")).toBeNull();
    expect(values).toHaveLength(2);
    subscription.unsubscribe();
  });
});
