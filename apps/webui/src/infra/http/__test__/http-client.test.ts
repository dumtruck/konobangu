import { createCancellationTokenSource } from "@securitydept/client";
import { type } from "arktype";
import { describe, expect, it, vi } from "vitest";
import { deferredReply } from "@/__test__/support/fixture";
import { ApiTarget } from "@/app/config/context";
import {
  type AuthorizationPolicy,
  HttpClient,
  type RequestAuthorization,
} from "@/infra/http/http-client";

const api = new ApiTarget(
  { version: 1, apiBaseUrl: "https://api.example/api/" },
  "https://ui.example",
);
function fixture(
  handler: (request: Request) => Response | Promise<Response> = () =>
    new Response('{"ok":true}'),
) {
  const root = new AbortController();
  const identity = new AbortController();
  const policy: AuthorizationPolicy = {
    identitySignal: identity.signal,
    authorize: vi.fn(async () => ({
      header: "Bearer test",
      assertCurrent: () => undefined,
    })),
    renew: vi.fn(async () => undefined),
  };
  const network = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) =>
    handler(new Request(input, init)),
  );
  const http = new HttpClient({
    api,
    network,
    rootSignal: root.signal,
    authorization: () => policy,
  });
  return { root, identity, policy, network, http };
}
describe("shared API middleware", () => {
  it("isolates public requests and rejects absent auth for protected requests", async () => {
    const { http, policy } = fixture();
    await http.json({
      endpoint: "auth/config",
      policy: "public",
      schema: type({ ok: "boolean" }),
    });
    expect(policy.authorize).not.toHaveBeenCalled();
    const unbound = new HttpClient({
      api,
      rootSignal: new AbortController().signal,
      network: async () => new Response(),
      authorization: () => null,
    });
    await expect(unbound.response({ endpoint: "graphql" })).rejects.toThrow(
      "authorization policy",
    );
  });
  it("normalizes Request method/headers and omits browser credentials", async () => {
    const seen: Request[] = [];
    const { http } = fixture((request) => {
      seen.push(request);
      return new Response(null, { status: 204 });
    });
    await http.apolloFetch(new AbortController().signal)(
      new Request(api.url("graphql"), {
        method: "POST",
        body: "{}",
        headers: { "X-Custom": "preserved" },
      }),
    );
    expect(seen[0].credentials).toBe("omit");
    expect(seen[0].headers.get("Authorization")).toBe("Bearer test");
    expect(seen[0].headers.get("X-Custom")).toBe("preserved");
    expect(seen[0].headers.get("X-Konobangu-CSRF")).toBe(null);
  });
  it("does not renew its own principal probe, nor replay commands", async () => {
    const { http, policy, network } = fixture(
      () => new Response(null, { status: 401 }),
    );
    const response = await http.response({
      endpoint: "auth/user-info",
      policy: "principal-probe",
    });
    expect(response.status).toBe(401);
    await expect(
      http.response({
        endpoint: "command",
        method: "POST",
        json: {},
        replay: "never",
      }),
    ).rejects.toThrow();
    expect(policy.renew).not.toHaveBeenCalled();
    expect(network).toHaveBeenCalledTimes(2);
  });
  it("shares a single bounded replay budget with renewal", async () => {
    const { http, policy, network } = fixture(
      () => new Response(null, { status: 401 }),
    );
    await expect(
      http.response({ endpoint: "read", replay: "safe" }),
    ).rejects.toThrow();
    expect(policy.renew).toHaveBeenCalledTimes(1);
    expect(network).toHaveBeenCalledTimes(2);
  });
  it("keeps the network signal bound to the identity after headers arrive", async () => {
    const { http, identity, network } = fixture(
      () => new Response(new ReadableStream<Uint8Array>({ pull() {} })),
    );
    await http.response({ endpoint: "static/subscribers/1/a" });
    const signal = new Request(...network.mock.calls[0]).signal;
    expect(signal.aborted).toBe(false);
    // The body is still unread: ending the identity must still reach the
    // signal the network was given, which is what ends a native body read.
    identity.abort(new Error("Identity ended"));
    expect(signal.aborted).toBe(true);
  });
  it("aborts authorization readiness immediately when the identity ends", async () => {
    const { http, policy, identity, network } = fixture();
    const authorization = deferredReply<RequestAuthorization>();
    vi.mocked(policy.authorize).mockImplementation(authorization.request);
    const pending = http
      .response({ endpoint: "read" })
      .catch((error: unknown) => error);
    await authorization.requested;
    identity.abort(new Error("Identity ended"));
    expect(await pending).toMatchObject({ message: "Identity ended" });
    expect(network).not.toHaveBeenCalled();
    authorization.resolve({
      header: "Bearer late",
      assertCurrent: () => undefined,
    });
  });
  it("checks the borrowed material before admitting its header to the network", async () => {
    const { http, policy, network } = fixture();
    vi.mocked(policy.authorize).mockResolvedValue({
      header: "Bearer superseded",
      assertCurrent: () => {
        throw new DOMException("Material superseded", "AbortError");
      },
    });
    await expect(http.response({ endpoint: "read" })).rejects.toMatchObject({
      name: "AbortError",
    });
    expect(network).not.toHaveBeenCalled();
  });
  it("preserves URL query and null bodies in protocol adapters", async () => {
    const { http } = fixture((request) => {
      expect(new URL(request.url).search).toBe("?format=one");
      return new Response(null, { status: 304 });
    });
    const response = await http.apolloFetch(new AbortController().signal)(
      api.url("graphql?format=one"),
    );
    expect(response.status).toBe(304);
    expect(response.body).toBeNull();
  });
  it("does not combine a network replay with another 401 replay", async () => {
    let calls = 0;
    const { http, policy } = fixture(() => {
      if (++calls === 1) throw new TypeError("fetch failed");
      return new Response(null, { status: 401 });
    });
    await expect(
      http.response({ endpoint: "read", replay: "safe" }),
    ).rejects.toThrow();
    expect(calls).toBe(2);
    expect(policy.renew).not.toHaveBeenCalled();
  });

  it("keeps caller cancellation connected through asynchronous schema validation", async () => {
    const { http } = fixture();
    const source = createCancellationTokenSource();
    const decoded = deferredReply<{ value: number }>();
    const pending = http
      .json({
        endpoint: "decode",
        cancellationToken: source.token,
        schema: {
          "~standard": {
            validate: decoded.request,
          },
        },
      })
      .then(
        () => "resolved",
        () => "cancelled",
      );
    await decoded.requested;
    source.cancel();
    // The validator never answers: only the cancellation can settle this.
    expect(await pending).toBe("cancelled");
  });

  it("rejects API path traversal before networking", async () => {
    const { http, network } = fixture();
    await expect(http.response({ endpoint: "../other" })).rejects.toThrow(
      "escapes",
    );
    expect(network).not.toHaveBeenCalled();
  });
});
