// @vitest-environment jsdom
import { describe, expect, it, vi } from "vitest";
import {
  createKonobanguFixture,
  deferredReply,
  identityKey,
  type KonobanguFixtureOptions,
  principalPayload,
  response,
  type TestRequestHandler,
} from "@/__test__/support/fixture";
import { OIDC_CLIENT_KEY } from "@/app/auth/setup";
import {
  OIDC_CLIENT_ENTRY,
  tokenSetRegistryFromInjector,
} from "@/app/auth/tokens";
import { HTTP_CLIENT } from "@/infra/http/http-client";

const scope = (
  handler: TestRequestHandler,
  options?: Omit<KonobanguFixtureOptions, "handler">,
) => createKonobanguFixture({ ...options, handler });
const administrator = () => response(200, principalPayload("Administrator"));

describe("the server-projected application identity", () => {
  it("drops a late projection once the root is disposed", async () => {
    const reply = deferredReply<Response>();
    const { environment, auth } = scope(reply.request);
    const refreshing = auth.refresh().catch((error: unknown) => error);
    await reply.requested;
    environment[Symbol.dispose]();
    reply.resolve(response(200, principalPayload("late")));
    await refreshing;
    expect(auth.isAuthenticated.get()).toBe(false);
  });

  it("does not register a client after root disposal while logout clears SDK state", async () => {
    const { environment, auth } = scope(() =>
      response(200, principalPayload()),
    );
    const registry = tokenSetRegistryFromInjector(environment.injector);
    const record = await registry.clientRecordFor(OIDC_CLIENT_KEY, {
      initialize: true,
    });
    const clearing = deferredReply<void>();
    vi.spyOn(record.client, "clearState").mockImplementation(clearing.request);
    const register = vi.spyOn(registry, "register");
    const logout = auth.logout();
    await clearing.requested;
    environment[Symbol.dispose]();
    clearing.resolve();
    await logout;
    expect(register).not.toHaveBeenCalled();
  });

  it("shares one request between concurrent callers", async () => {
    const { auth, fetch } = scope(() => response(200, principalPayload()));
    await Promise.all([auth.refresh(), auth.refresh()]);
    expect(fetch).toHaveBeenCalledTimes(1);
    expect(auth.isAuthenticated.get()).toBe(true);
    expect(auth.userData.get()?.subject).toBe("A");
    expect(auth.identityKey.get()).toBe(identityKey());
  });

  it("sends explicit Bearer only to protected requests and omits browser credentials", async () => {
    const seen: Array<[string, string | null, RequestCredentials | undefined]> =
      [];
    const { environment, auth } = scope((request) => {
      seen.push([
        request.url,
        request.headers.get("Authorization"),
        request.credentials,
      ]);
      return administrator();
    });
    await auth.refresh();
    const http = environment.injector.get(HTTP_CLIENT);
    await http.response({ endpoint: "auth/config", policy: "public" });
    const config = seen.find(([url]) => url.endsWith("/auth/config"));
    const userInfo = seen.find(([url]) => url.endsWith("/auth/user-info"));
    expect(userInfo?.[1]).toBe("Bearer fixture");
    expect(userInfo?.[2]).toBe("omit");
    expect(config?.[2]).toBe("omit");
    expect(config?.[1]).toBeNull();
  });

  it.each([403, 500, 503])(
    "preserves HTTP %i as a failure instead of a signed-out state",
    async (status) => {
      const { auth } = scope(() => response(status, { message: "denied" }));
      await expect(auth.refresh()).rejects.toThrow();
      expect(auth.check.get().status).toBe("error");
      expect(auth.isAuthenticated.get()).toBe(false);
    },
  );

  it("treats 401 as unauthenticated and malformed success as a failure", async () => {
    const anonymous = scope(() => response(401));
    expect(await anonymous.auth.refresh()).toBeNull();
    expect(anonymous.auth.check.get()).toEqual({
      status: "success",
      isAuthenticated: false,
    });
    const malformed = scope(() => new Response("not json", { status: 200 }));
    await expect(malformed.auth.refresh()).rejects.toThrow();
    expect(malformed.auth.check.get().status).toBe("error");
  });

  it("closes new acquisitions before broadcasting local logout", async () => {
    const { auth, fetch } = scope(administrator);
    await auth.refresh();
    const epoch = auth.epoch();
    const identity = auth.identitySignal;
    const calls = fetch.mock.calls.length;
    const logout = auth.logout();
    await Promise.all([
      expect(auth.refresh()).rejects.toMatchObject({ name: "AbortError" }),
      expect(auth.renew(new AbortController().signal)).rejects.toMatchObject({
        name: "AbortError",
      }),
      expect(auth.login()).rejects.toMatchObject({ name: "AbortError" }),
    ]);
    await logout;
    expect(identity.aborted).toBe(true);
    expect(auth.epoch()).toBeGreaterThan(epoch);
    expect(auth.isAuthenticated.get()).toBe(false);
    expect(auth.userData.get()).toBeNull();
    expect(fetch).toHaveBeenCalledTimes(calls);
  });

  it("ends the owned identity after a later principal 401", async () => {
    let authorized = true;
    const { auth } = scope(() =>
      authorized ? administrator() : response(401),
    );
    await auth.refresh();
    const epoch = auth.epoch();
    const identity = auth.identitySignal;
    authorized = false;
    expect(await auth.refresh()).toBeNull();
    expect(identity.aborted).toBe(true);
    expect(auth.epoch()).toBeGreaterThan(epoch);
    expect(auth.userData.get()).toBeNull();
    expect(auth.identityKey.get()).toBeNull();
    expect(auth.check.get()).toEqual({
      status: "success",
      isAuthenticated: false,
    });
    expect(await auth.refresh()).toBeNull();
  });

  it("confirms principal from shared SDK authenticated events, including public destinations", async () => {
    const reply = deferredReply<Response>();
    const { environment, auth, fetch } = scope(reply.request);
    const record = await tokenSetRegistryFromInjector(
      environment.injector,
    ).clientRecordFor(OIDC_CLIENT_KEY, { initialize: true });
    const renew = vi.spyOn(record.client, "refreshState");
    await record.client.restoreState({
      tokens: { accessToken: "fixture" },
      metadata: {},
    });
    await reply.requested;
    // MaterialRestored and Authenticated belong to the same SDK commit.
    expect(fetch).toHaveBeenCalledTimes(1);
    reply.resolve(response(200, principalPayload("PublicTarget")));
    await vi.waitFor(() =>
      expect(auth.userData.get()?.subject).toBe("PublicTarget"),
    );
    expect(renew).not.toHaveBeenCalled();
  });
  it("confirms a factory-authenticated ready snapshot even when its auth event occurred before registry forwarding", async () => {
    const { environment, auth } = scope(() =>
      response(200, principalPayload("ReadyUser")),
    );
    const registry = tokenSetRegistryFromInjector(environment.injector);
    const entry = environment.injector.get(OIDC_CLIENT_ENTRY)();
    registry.unregister(OIDC_CLIENT_KEY);
    registry.register({
      ...entry,
      clientFactory: async (options) => {
        const client = await entry.clientFactory(options);
        await client.restoreState({
          tokens: { accessToken: "ready-fixture" },
          metadata: {},
        });
        return client;
      },
    });
    await registry.clientRecordFor(OIDC_CLIENT_KEY, { initialize: true });
    await vi.waitFor(() =>
      expect(auth.userData.get()?.subject).toBe("ReadyUser"),
    );
  });

  it("ends the OIDC identity locally without calling a revocation endpoint", async () => {
    const urls: string[] = [];
    const { auth } = scope((request) => {
      urls.push(request.url);
      return response(200, principalPayload());
    });
    await auth.refresh();
    expect(auth.isAuthenticated.get()).toBe(true);
    await auth.logout();
    expect(auth.isAuthenticated.get()).toBe(false);
    expect(auth.identityKey.get()).toBeNull();
    expect(urls.every((url) => url.includes("/api/auth/user-info"))).toBe(true);
  });
});
