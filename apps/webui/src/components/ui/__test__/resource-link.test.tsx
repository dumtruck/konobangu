// @vitest-environment jsdom
import { act, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { createSignedInFixture, response } from "@/__test__/support/fixture";
import { renderInEnvironment } from "@/__test__/support/render";
import { decodeBase64Url } from "@/app/resource/resource-grants";
import fixture from "../../../../../recorder/tests/fixtures/native-resource-grants.json";
import { ResourceLink } from "../resource-link";

afterEach(() => vi.useRealTimers());

describe("native resource links", () => {
  it("binds a feed locator, renews locally, pauses while hidden and releases its clock", async () => {
    vi.useFakeTimers();
    vi.setSystemTime(fixture.issuedAt * 1000);
    const app = await createSignedInFixture({
      handler: (request) =>
        new URL(request.url).pathname.endsWith("/auth/resource-grants")
          ? response(200, {
              grant: fixture.grant,
              issuedAt: new Date(fixture.issuedAt * 1000).toISOString(),
              expiresAt: new Date(
                (fixture.issuedAt + 3600) * 1000,
              ).toISOString(),
              childTtlSeconds: 300,
            })
          : response(404),
    });
    const visibility = app.environment.pageVisibility;
    const input = app.api.url("feeds/rss/fixture-feed-A");
    const view = renderInEnvironment(
      app,
      <ResourceLink href={input}>RSS Feed</ResourceLink>,
    );
    const link = screen.getByText("RSS Feed");
    const address = () => link.getAttribute("href");
    await vi.waitFor(async () => {
      await act(async () => {});
      expect(address()).not.toBeNull();
    });
    const first = address();
    if (!first) throw new Error("A visible feed must receive a native URL");
    const child = JSON.parse(
      new TextDecoder().decode(
        decodeBase64Url(new URL(first).searchParams.get("resource_token")!),
      ),
    );
    const resource = new TextDecoder().decode(decodeBase64Url(child.c[0].i64));
    expect(resource).toBe(
      'resource = {"subscriberId":1001,"path":"feeds/rss/fixture-feed-A"}',
    );
    expect(link.getAttribute("target")).toBe("_blank");
    expect(link.getAttribute("referrerpolicy")).toBe("no-referrer");
    await act(async () => {
      await vi.advanceTimersByTimeAsync(285_000);
    });
    await vi.waitFor(async () => {
      await act(async () => {});
      expect(address()).not.toBe(first);
      expect(address()).not.toBeNull();
    });
    expect(app.requests({ path: "auth/resource-grants" })).toHaveLength(1);
    await act(async () => visibility.set(false));
    expect(address()).toBeNull();
    await act(async () => vi.advanceTimersByTimeAsync(60_000));
    expect(address()).toBeNull();
    await act(async () => {
      visibility.set(true);
    });
    await vi.waitFor(async () => {
      await act(async () => {});
      expect(address()).not.toBeNull();
    });
    view.unmount();
    expect(vi.getTimerCount()).toBe(0);
  });
});
