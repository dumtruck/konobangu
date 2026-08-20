import { Subject } from "rxjs";
import { describe, expect, it, vi } from "vitest";
import { ImageBinding, ResourceBinding } from "../resource-binding";

function setup(options: { private?: boolean } = {}) {
  const sources: Subject<{ url: string } | undefined>[] = [];
  const binding = new ResourceBinding({
    input: "https://api.test/api/feeds/rss/a",
    private: options.private ?? true,
    observe: () => {
      const source = new Subject<{ url: string } | undefined>();
      sources.push(source);
      return source;
    },
  });
  return {
    binding,
    sources,
    /** Sources that still have the binding subscribed. */
    live: () => sources.filter((source) => source.observed).length,
  };
}

describe("a connectable resource address", () => {
  it("derives nothing until connected and follows its source while connected", () => {
    const { binding, sources, live } = setup();
    expect(binding.state.get()).toEqual({ status: "pending" });
    expect(sources).toHaveLength(0);
    binding.connect();
    expect(live()).toBe(1);
    sources[0].next({ url: "https://api.test/a?resource_token=1" });
    expect(binding.state.get()).toEqual({
      status: "ready",
      url: "https://api.test/a?resource_token=1",
    });
    // The source withdraws the address while it renews.
    sources[0].next(undefined);
    expect(binding.state.get()).toEqual({ status: "pending" });
  });

  it("counts connections and stays usable after its last disconnect", () => {
    const { binding, sources, live } = setup();
    // A renderer may mount, unmount and mount the same view again.
    binding.connect();
    binding.connect();
    expect(sources).toHaveLength(1);
    binding.disconnect();
    expect(live()).toBe(1);
    sources[0].next({ url: "https://api.test/a?resource_token=1" });
    binding.disconnect();
    expect(live()).toBe(0);
    // The bounded address does not outlive its last user.
    expect(binding.state.get()).toEqual({ status: "pending" });
    binding.disconnect();
    binding.connect();
    expect(sources).toHaveLength(2);
    expect(live()).toBe(1);
    sources[1].next({ url: "https://api.test/a?resource_token=2" });
    expect(binding.state.get()).toMatchObject({ status: "ready" });
  });

  it("reports a failure and derives again only on an explicit retry", () => {
    const { binding, sources, live } = setup();
    binding.retry();
    expect(sources).toHaveLength(0);
    binding.connect();
    const failure = new Error("offline");
    sources[0].error(failure);
    expect(binding.state.get()).toEqual({ status: "failed", error: failure });
    expect(sources).toHaveLength(1);
    binding.retry();
    expect(binding.state.get()).toEqual({ status: "pending" });
    expect(live()).toBe(1);
    sources[1].next({ url: "https://api.test/a?resource_token=3" });
    expect(binding.state.get()).toMatchObject({ status: "ready" });
  });

  it("uses a public locator as its own address without deriving anything", () => {
    const { binding, sources } = setup({ private: false });
    expect(binding.state.get()).toEqual({
      status: "ready",
      url: "https://api.test/api/feeds/rss/a",
    });
    binding.connect();
    binding.retry();
    binding.disconnect();
    expect(sources).toHaveLength(0);
    expect(binding.state.get()).toMatchObject({ status: "ready" });
  });
});

function image(options: { private?: boolean } = {}) {
  const identityChanges = new Subject<void>();
  const release = vi.fn();
  const acquire = vi.fn(async (input: string, renewal?: "renew") => ({
    url: `${input}?resource_token=${renewal ?? "first"}`,
  }));
  const binding = new ImageBinding({
    input: "https://api.test/api/static/subscribers/1/image/a.png",
    private: options.private ?? true,
    acquire,
    activate: () => ({ [Symbol.dispose]: release }),
    identityChanges,
  });
  return { binding, acquire, release, identityChanges };
}
const settle = () => vi.waitFor(() => undefined);

describe("a connectable image address", () => {
  it("derives once while connected and keeps renewal alive until disconnected", async () => {
    const { binding, acquire, release } = image();
    expect(binding.state.get()).toEqual({ status: "pending", generation: 0 });
    expect(acquire).not.toHaveBeenCalled();
    binding.connect();
    binding.connect();
    await vi.waitFor(() =>
      expect(binding.state.get()).toMatchObject({ status: "ready" }),
    );
    expect(acquire).toHaveBeenCalledTimes(1);
    binding.disconnect();
    expect(release).not.toHaveBeenCalled();
    binding.disconnect();
    expect(release).toHaveBeenCalledTimes(1);
    expect(binding.state.get()).toEqual({ status: "pending", generation: 0 });
    // Still usable: a remounted view connects the same binding again.
    binding.connect();
    await vi.waitFor(() =>
      expect(binding.state.get()).toMatchObject({ status: "ready" }),
    );
  });

  it("renews once after a failed first load, then reports failure", async () => {
    const { binding, acquire } = image();
    binding.connect();
    await vi.waitFor(() =>
      expect(binding.state.get()).toMatchObject({ status: "ready" }),
    );
    expect(binding.loadFailed()).toBe(true);
    expect(binding.state.get()).toEqual({ status: "pending", generation: 1 });
    await vi.waitFor(() =>
      expect(binding.state.get()).toMatchObject({ status: "ready" }),
    );
    expect(acquire).toHaveBeenLastCalledWith(binding.input, "renew");
    expect(binding.loadFailed()).toBe(false);
    expect(binding.state.get()).toEqual({ status: "failed", generation: 1 });
    binding.retry();
    expect(binding.state.get()).toEqual({ status: "pending", generation: 2 });
  });

  it("does not replace an address its element already loaded", async () => {
    const { binding, acquire } = image();
    binding.connect();
    await vi.waitFor(() =>
      expect(binding.state.get()).toMatchObject({ status: "ready" }),
    );
    binding.loaded();
    expect(binding.loadFailed()).toBe(false);
    expect(acquire).toHaveBeenCalledTimes(1);
  });

  it("starts over for a new identity and drops a late address of the old one", async () => {
    const { binding, acquire, identityChanges } = image();
    const late = Promise.withResolvers<{ url: string }>();
    acquire.mockImplementationOnce(() => late.promise);
    binding.connect();
    identityChanges.next();
    expect(binding.state.get()).toEqual({ status: "pending", generation: 1 });
    late.resolve({ url: "https://api.test/old" });
    await vi.waitFor(() =>
      expect(binding.state.get()).toMatchObject({ status: "ready" }),
    );
    expect(binding.state.get()).toMatchObject({
      generation: 1,
      url: expect.stringContaining("resource_token=first"),
    });
  });

  it("serves a public image as its own address, with a fresh element per retry", async () => {
    const { binding, acquire, release } = image({ private: false });
    binding.connect();
    expect(binding.state.get()).toEqual({
      status: "ready",
      url: binding.input,
      generation: 0,
    });
    expect(binding.loadFailed()).toBe(false);
    expect(binding.state.get()).toMatchObject({ status: "failed" });
    binding.retry();
    expect(binding.state.get()).toEqual({
      status: "ready",
      url: binding.input,
      generation: 1,
    });
    binding.disconnect();
    await settle();
    expect(acquire).not.toHaveBeenCalled();
    expect(release).not.toHaveBeenCalled();
  });
});
