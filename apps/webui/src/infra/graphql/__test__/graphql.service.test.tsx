// @vitest-environment jsdom
import { gql } from "@apollo/client";
import { useQuery } from "@apollo/client/react";
import type { TypedDocumentNode } from "@graphql-typed-document-node/core";
import { act, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import {
  createKonobanguFixture,
  createSignedInFixture,
  deferredReply,
  identityKey,
  oidcClientRecord,
  principalPayload,
  response,
} from "@/__test__/support/fixture";
import { renderInEnvironment } from "@/__test__/support/render";
import { QueryPartialError } from "@/components/ui/query-error-view";
import { GraphQLService } from "../graphql.service";

const query: TypedDocumentNode<
  { privateName: string },
  Record<string, never>
> = gql`query PrivateName { privateName }`;
const mutation: TypedDocumentNode<
  { updatePrivateName: string },
  Record<string, never>
> = gql`mutation UpdatePrivateName { updatePrivateName }`;
async function fixture() {
  const app = await createSignedInFixture({
    handler: () => response(200, { data: { privateName: "private" } }),
  });
  return { ...app, graphql: app.injector.get(GraphQLService) };
}
describe("Apollo identity scopes and typed errors", () => {
  it("closes an established private scope immediately on explicit SDK material restore", async () => {
    const oldGraphql = deferredReply<Response>();
    const principalB = deferredReply<Response>();
    const scope = createKonobanguFixture({
      realSdkAuthorization: true,
      handler: (request) => {
        const material = request.headers.get("Authorization");
        if (request.url.endsWith("/graphql")) return oldGraphql.request();
        if (material === "Bearer material-B") return principalB.request();
        return response(200, principalPayload("A"));
      },
    });
    const record = await oidcClientRecord(scope);
    await record.client.restoreState({
      tokens: { accessToken: "material-A" },
      metadata: {},
    });
    await waitFor(() => expect(scope.auth.userData.get()?.subject).toBe("A"));
    const graphql = scope.injector.get(GraphQLService);
    const oldClient = graphql._apollo;
    oldClient.writeQuery({ query, data: { privateName: "A cached private" } });
    const oldSignal = scope.auth.identitySignal;
    const oldReading = graphql
      .query({ query })
      .catch((error: unknown) => error);
    await oldGraphql.requested;
    await record.client.restoreState({
      tokens: { accessToken: "material-B" },
      metadata: {},
    });
    expect(oldSignal.aborted).toBe(true);
    expect(scope.auth.identityKey.get()).toBeNull();
    expect(graphql._apollo).not.toBe(oldClient);
    await principalB.requested;
    principalB.resolve(response(200, principalPayload("B")));
    await waitFor(() => expect(scope.auth.userData.get()?.subject).toBe("B"));
    oldGraphql.resolve(
      response(200, { data: { privateName: "late A private" } }),
    );
    await oldReading;
    expect(graphql._apollo.readQuery({ query })).toBeNull();
  });
  it("confirms the latest real SDK material and never stores B data in an A identity scope", async () => {
    const principalA = deferredReply<Response>();
    const principalB = deferredReply<Response>();
    const scope = createKonobanguFixture({
      realSdkAuthorization: true,
      handler: (request) => {
        const material = request.headers.get("Authorization");
        if (request.url.endsWith("/graphql"))
          return response(200, {
            data: {
              privateName:
                material === "Bearer material-B" ? "B private" : "A private",
            },
          });
        return material === "Bearer material-A"
          ? principalA.request()
          : principalB.request();
      },
    });
    const record = await oidcClientRecord(scope);
    await record.client.restoreState({
      tokens: { accessToken: "material-A" },
      metadata: {},
    });
    await principalA.requested;
    const graphql = scope.injector.get(GraphQLService);
    await record.client.restoreState({
      tokens: { accessToken: "material-B" },
      metadata: {},
    });
    await principalB.requested;
    principalA.resolve(response(200, principalPayload("A")));
    expect(scope.auth.userData.get()).toBeNull();
    // No identity is verified yet, so authorization holds the request. It
    // belongs to the client that the confirmed identity then retires.
    const early = graphql
      .query({ query })
      .catch((error: unknown) => ({ data: undefined, error }));
    principalB.resolve(response(200, principalPayload("B")));
    await waitFor(() => expect(scope.auth.userData.get()?.subject).toBe("B"));
    expect((await early).data).toBeUndefined();
    const result = await graphql.query({ query });
    expect(result.data).toEqual({ privateName: "B private" });
    expect(graphql._apollo.readQuery({ query })).toEqual({
      privateName: "B private",
    });
    expect(scope.auth.identityKey.get()).toBe(identityKey("B"));
    expect(scope.requests({ path: "auth/user-info" })).toHaveLength(2);
  });

  it("preserves the client and cache during real SDK refresh, admitting renewed traffic after principal confirmation", async () => {
    vi.useFakeTimers({ toFake: ["Date"] });
    const renewedPrincipal = deferredReply<Response>();
    const seenGraphql: string[] = [];
    const scope = createKonobanguFixture({
      realSdkAuthorization: true,
      handler: (request) => {
        const material = request.headers.get("Authorization");
        if (request.url === "https://idp.example/token")
          return response(200, {
            access_token: "material-A-renewed",
            token_type: "Bearer",
            expires_in: 300,
          });
        if (request.url.endsWith("/graphql")) {
          seenGraphql.push(material ?? "anonymous");
          if (material === "Bearer material-A") return response(401);
          return response(200, { data: { privateName: "A renewed private" } });
        }
        if (material === "Bearer material-A-renewed")
          return renewedPrincipal.request();
        return response(200, principalPayload("A"));
      },
    });
    // SDK v0.3 OAuth grant requests use the standard Fetch terminal directly.
    vi.stubGlobal("fetch", (input: RequestInfo | URL, init?: RequestInit) =>
      scope.fetch(input, init),
    );
    const record = await oidcClientRecord(scope);
    const issued = Temporal.Now.instant();
    await record.client.restoreState({
      tokens: {
        accessToken: "material-A",
        accessTokenIssuedAt: issued.toString(),
        accessTokenExpiresAt: issued.add({ hours: 1 }).toString(),
        refreshMaterial: "refresh-A",
      },
      metadata: {},
    });
    await waitFor(() => expect(scope.auth.userData.get()?.subject).toBe("A"));
    const graphql = scope.injector.get(GraphQLService);
    const originalClient = graphql._apollo;
    originalClient.writeQuery({
      query,
      data: { privateName: "A cached private" },
    });
    vi.setSystemTime(issued.add({ hours: 2 }).epochMilliseconds);
    const reading = graphql.query({ query });
    await act(async () => {
      await renewedPrincipal.requested;
    });
    expect(graphql._apollo).toBe(originalClient);
    expect(originalClient.readQuery({ query })).toEqual({
      privateName: "A cached private",
    });
    let privateAuthorizationReady = false;
    const privateAuthorization = scope.auth
      .authorize(scope.api.baseUrl, scope.auth.identitySignal, "protected")
      .then((value) => {
        privateAuthorizationReady = true;
        return value;
      });
    // Nothing signals that authorization stayed blocked, so this one case
    // gives a wrongly admitted request time to show up.
    await new Promise((resolve) => setTimeout(resolve, 20));
    expect(seenGraphql).toEqual(["Bearer material-A"]);
    expect(privateAuthorizationReady).toBe(false);
    await act(async () => {
      renewedPrincipal.resolve(response(200, principalPayload("A")));
      await reading;
    });
    expect(await privateAuthorization).toMatchObject({
      header: "Bearer material-A-renewed",
    });
    expect(seenGraphql).toEqual([
      "Bearer material-A",
      "Bearer material-A-renewed",
    ]);
    expect(graphql._apollo).toBe(originalClient);
    expect(originalClient.readQuery({ query })).toEqual({
      privateName: "A renewed private",
    });
  });

  it("keeps the cache for a same-identity refresh and replaces it on logout or issuer/subject change", async () => {
    const scope = await fixture();
    const first = scope.graphql._apollo;
    first.writeQuery({ query, data: { privateName: "A private" } });
    const reply = deferredReply<Response>();
    scope.fetch.mockImplementationOnce(reply.request);
    const refreshing = scope.auth.refresh();
    await reply.requested;
    expect(scope.auth.isAuthenticated.get()).toBe(true);
    expect(scope.graphql._apollo).toBe(first);
    expect(first.readQuery({ query })).toEqual({ privateName: "A private" });
    reply.resolve(scope.principal());
    await refreshing;
    expect(scope.graphql._apollo).toBe(first);
    await scope.identity("B");
    const second = scope.graphql._apollo;
    expect(second).not.toBe(first);
    expect(second.readQuery({ query })).toBeNull();
    second.writeQuery({ query, data: { privateName: "B private" } });
    await scope.identity(null);
    expect(scope.graphql._apollo.readQuery({ query })).toBeNull();
  });
  it("aborts old requests and prevents a late response from filling the next identity cache", async () => {
    const scope = await fixture();
    const reply = deferredReply<Response>();
    // Only the GraphQL request stalls; the identity probe must still answer.
    scope.fetch.mockImplementation(async (input) =>
      String(input) === scope.api.url("auth/user-info")
        ? scope.principal()
        : reply.request(),
    );
    const old = scope.graphql.query({ query }).catch((error: unknown) => error);
    await reply.requested;
    await scope.identity("B");
    const [[, stalled]] = scope.requests({ path: "graphql" });
    expect(stalled?.signal?.aborted).toBe(true);
    reply.resolve(response(200, { data: { privateName: "late A" } }));
    await old;
    expect(scope.graphql._apollo.readQuery({ query })).toBeNull();
  });
  it("displays partial query data beside errors, while mutation errors reject", async () => {
    const scope = await fixture();
    scope.fetch.mockImplementation(async () =>
      response(200, {
        data: { privateName: "visible partial data" },
        errors: [{ message: "another field failed" }],
      }),
    );
    function Consumer() {
      const { data, error } = useQuery(query);
      return (
        <>
          <span>{data?.privateName}</span>
          <QueryPartialError error={error} />
        </>
      );
    }
    renderInEnvironment(scope, <Consumer />, { apollo: true });
    expect(await screen.findByText("visible partial data")).toBeTruthy();
    expect((await screen.findByRole("alert")).textContent).toContain(
      "another field failed",
    );
    await expect(scope.graphql.mutate({ mutation })).rejects.toThrow(
      "another field failed",
    );
  });
});
