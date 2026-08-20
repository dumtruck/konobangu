import {
  ClientError,
  ClientErrorKind,
  type TimeTrait,
} from "@securitydept/client";
import { NetworkError } from "ky";
import { Subject } from "rxjs";
import { describe, expect, it, vi } from "vitest";
import { ApiTarget } from "@/app/config/context";
import { createResourceCrypto } from "@/app/environment/crypto";
import {
  createPageVisibilityForTest,
  createViewportForTest,
} from "@/app/environment/test";
import { HttpResponseError } from "@/infra/http/http-client";
import fixture from "../../../../../recorder/tests/fixtures/native-resource-grants.json";
import {
  createResourceService,
  decodeBase64Url,
  encodeBase64Url,
} from "../resource-grants";

function setup() {
  let now = fixture.issuedAt * 1000;
  let scheduled: { delay: number; run(): void } | undefined;
  const identityChanges = new Subject<void>();
  let identity = new AbortController();
  let epoch = 1;
  // The service owns at most one timer, so the clock exposes exactly that one
  // with its requested delay instead of a queue to advance.
  const time: TimeTrait = {
    now: () => now,
    setTimeout(run, delay) {
      scheduled = {
        delay,
        run() {
          scheduled = undefined;
          run();
        },
      };
      return scheduled;
    },
    clearTimeout(handle) {
      if (handle === scheduled) scheduled = undefined;
    },
  };
  const visibility = createPageVisibilityForTest();
  const issue = vi.fn(async () => ({
    grant: fixture.grant,
    issuedAt: new Date(fixture.issuedAt * 1000).toISOString(),
    expiresAt: new Date((fixture.issuedAt + 3600) * 1000).toISOString(),
    childTtlSeconds: 300 as const,
  }));
  const service = createResourceService({
    api: new ApiTarget(
      { version: 1, apiBaseUrl: "https://api.test/prefix/api/" },
      "https://ui.test",
    ),
    time,
    visibility,
    viewport: createViewportForTest(),
    crypto: createResourceCrypto(globalThis.crypto),
    issue,
    auth: {
      epoch: () => epoch,
      get identitySignal() {
        return identity.signal;
      },
      lifetimeSignal: new AbortController().signal,
      identityChanges,
    },
  });
  return {
    service,
    issue,
    now: (value: number) => {
      now = value;
    },
    scheduled: () => scheduled,
    visibility: (value: boolean) => visibility.set(value),
    changeIdentity() {
      identity.abort();
      identity = new AbortController();
      epoch++;
      identityChanges.next();
    },
  };
}
const input =
  "https://api.test/prefix/api/static/subscribers/1001/image/posters/cover.png?optimize=accept";
describe("native resource grants", () => {
  it("matches the Rust verifier's standard macaroon vector, omitting parent signature", async () => {
    const { service, issue } = setup();
    using _owned = service;
    const [first, second] = await Promise.all([
      service.acquire(input),
      service.acquire(input),
    ]);
    expect(issue).toHaveBeenCalledTimes(1);
    expect(first).toEqual(second);
    const url = new URL(first.url);
    expect(url.searchParams.get("resource_token")).toBe(fixture.resourceToken);
    expect(
      new TextDecoder().decode(
        decodeBase64Url(url.searchParams.get("resource_token")!),
      ),
    ).toBe(fixture.child);
    expect(first.url).not.toContain(JSON.parse(fixture.grant).s64);
    expect(url.searchParams.get("optimize")).toBe("accept");
  });
  it("rejects cross-owner and malformed resource paths before signing", async () => {
    const { service } = setup();
    using _owned = service;
    await expect(
      service.acquire(input.replace("/1001/", "/1002/")),
    ).rejects.toThrow("another subscriber");
    for (const value of [
      input.replace("cover.png", "a%2fb"),
      input.replace("cover.png", "a%5cb"),
      input.replace("cover.png", "a%25b"),
      input.replace("cover.png", "a%3ab"),
      input.replace("posters/cover.png", "posters/../cover.png"),
      `${input}&extra=value`,
      input.replace("/image/", "/torrent/"),
    ])
      await expect(service.acquire(value)).rejects.toThrow();
    expect(service.isPrivate("https://other.test/image.png")).toBe(false);
  });
  it("owns one renewal timer only while resources are active and visible", async () => {
    const state = setup();
    using _owned = state.service;
    await state.service.acquire(input);
    expect(state.scheduled()).toBeUndefined();
    const lease = state.service.activate();
    expect(state.scheduled()?.delay).toBe(3_300_000);
    state.visibility(false);
    expect(state.scheduled()).toBeUndefined();
    state.now((fixture.issuedAt + 4000) * 1000);
    state.visibility(true);
    expect(state.scheduled()?.delay).toBe(0);
    lease[Symbol.dispose]();
    expect(state.scheduled()).toBeUndefined();
  });
  it("rejects late issuance after identity changes", async () => {
    const state = setup();
    using _owned = state.service;
    const reply =
      Promise.withResolvers<Awaited<ReturnType<typeof state.issue>>>();
    state.issue.mockImplementationOnce(() => reply.promise);
    const pending = state.service.acquire(input);
    state.changeIdentity();
    reply.resolve({
      grant: fixture.grant,
      issuedAt: new Date(fixture.issuedAt * 1000).toISOString(),
      expiresAt: new Date((fixture.issuedAt + 3600) * 1000).toISOString(),
      childTtlSeconds: 300,
    });
    await expect(pending).rejects.toThrow();
  });
  it("keeps an unexpired grant after transient renewal failure and bounds retries", async () => {
    const state = setup();
    using _owned = state.service;
    const lease = state.service.activate();
    await state.service.acquire(input);
    state.issue.mockRejectedValue(
      new NetworkError(
        new Request("https://api.test/prefix/api/auth/resource-grants"),
        { cause: new TypeError("Failed to fetch") },
      ),
    );
    state.now((fixture.issuedAt + 3300) * 1000);
    for (let n = 0; n < 3; n++) {
      state.scheduled()!.run();
      // Every failed renewal but the last schedules the next attempt.
      if (n < 2)
        await vi.waitFor(() => expect(state.scheduled()).toBeDefined());
    }
    await vi.waitFor(() => expect(state.issue).toHaveBeenCalledTimes(4));
    await expect(state.service.acquire(input)).resolves.toBeDefined();
    expect(state.scheduled()).toBeUndefined();
    lease[Symbol.dispose]();
  });
  it("recovers initial transient acquisition with one shared cancellable retry", async () => {
    const state = setup();
    using _owned = state.service;
    using _lease = state.service.activate();
    state.issue.mockRejectedValueOnce(
      new NetworkError(
        new Request("https://api.test/prefix/api/auth/resource-grants"),
        { cause: new TypeError("Failed to fetch") },
      ),
    );
    const pending = Promise.all([
      state.service.acquire(input),
      state.service.acquire(input),
    ]);
    await vi.waitFor(() => expect(state.scheduled()?.delay).toBe(5000));
    expect(state.issue).toHaveBeenCalledTimes(1);
    state.scheduled()!.run();
    const [first, second] = await pending;
    expect(first).toEqual(second);
    expect(state.issue).toHaveBeenCalledTimes(2);
  });
  it("limits initial transient failure to three retries and rejects callers explicitly", async () => {
    const state = setup();
    using _owned = state.service;
    using _lease = state.service.activate();
    state.issue.mockRejectedValue(
      new HttpResponseError(503, {
        code: "custom.unavailable",
        message: "Unavailable",
      }),
    );
    const rejected = expect(state.service.acquire(input)).rejects.toMatchObject(
      { status: 503 },
    );
    for (const delay of [5000, 10000, 20000]) {
      await vi.waitFor(() => expect(state.scheduled()?.delay).toBe(delay));
      state.scheduled()!.run();
    }
    await rejected;
    expect(state.issue).toHaveBeenCalledTimes(4);
    expect(state.scheduled()).toBeUndefined();
  });
  it.each([401, 403])(
    "never schedules a retry for HTTP %s, even with a misleading body kind",
    async (status) => {
      const state = setup();
      using _owned = state.service;
      using _lease = state.service.activate();
      state.issue.mockRejectedValue(
        new HttpResponseError(status, { kind: "unavailable" }),
      );
      await expect(state.service.acquire(input)).rejects.toMatchObject({
        status,
      });
      expect(state.issue).toHaveBeenCalledTimes(1);
      expect(state.scheduled()).toBeUndefined();
    },
  );
  it("does not retry invalid response schemas", async () => {
    const state = setup();
    using _owned = state.service;
    using _lease = state.service.activate();
    state.issue.mockRejectedValue(
      new ClientError({
        kind: ClientErrorKind.Configuration,
        message: "Invalid response",
      }),
    );
    await expect(state.service.acquire(input)).rejects.toThrow(
      "Invalid response",
    );
    expect(state.scheduled()).toBeUndefined();
  });
  it.each(["inactive", "hidden", "identity", "owner"])(
    "cancels initial acquisition retry at the %s boundary",
    async (boundary) => {
      const state = setup();
      const lease = state.service.activate();
      state.issue.mockRejectedValue(
        new NetworkError(
          new Request("https://api.test/prefix/api/auth/resource-grants"),
          { cause: new TypeError("Failed to fetch") },
        ),
      );
      const rejected = expect(
        state.service.acquire(input),
      ).rejects.toBeDefined();
      await vi.waitFor(() => expect(state.scheduled()?.delay).toBe(5000));
      if (boundary === "inactive") lease[Symbol.dispose]();
      if (boundary === "hidden") state.visibility(false);
      if (boundary === "identity") state.changeIdentity();
      if (boundary === "owner") state.service[Symbol.dispose]();
      await rejected;
      expect(state.scheduled()).toBeUndefined();
      expect(state.issue).toHaveBeenCalledTimes(1);
      if (boundary !== "inactive") lease[Symbol.dispose]();
      if (boundary !== "owner") state.service[Symbol.dispose]();
    },
  );
  it("rejects malformed UTF-8 grant identifiers without retrying or falling back to a cached parent", async () => {
    const state = setup();
    using _owned = state.service;
    await state.service.acquire(input);
    const malformed = {
      ...JSON.parse(fixture.grant),
      i64: encodeBase64Url(new Uint8Array([0xff])),
    };
    const reply = {
      grant: JSON.stringify(malformed),
      issuedAt: new Date(fixture.issuedAt * 1000).toISOString(),
      expiresAt: new Date((fixture.issuedAt + 3600) * 1000).toISOString(),
      childTtlSeconds: 300 as const,
    };
    state.issue.mockResolvedValue(reply);
    await expect(state.service.acquire(input, "renew")).rejects.toBeInstanceOf(
      TypeError,
    );
    expect(state.issue).toHaveBeenCalledTimes(2);
    expect(state.scheduled()).toBeUndefined();
    const initial = setup();
    using _initialOwned = initial.service;
    using _lease = initial.service.activate();
    initial.issue.mockResolvedValue(reply);
    await expect(initial.service.acquire(input)).rejects.toBeInstanceOf(
      TypeError,
    );
    expect(initial.issue).toHaveBeenCalledTimes(1);
    expect(initial.scheduled()).toBeUndefined();
  });
});
