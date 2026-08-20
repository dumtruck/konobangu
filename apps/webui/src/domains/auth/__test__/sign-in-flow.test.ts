import { describe, expect, it, vi } from "vitest";
import { SignInFlow } from "../sign-in-flow";

function setup() {
  const login = Promise.withResolvers<void>();
  const lifetime = new AbortController();
  const start = vi.fn(() => login.promise);
  const flow = new SignInFlow({
    lifetimeSignal: lifetime.signal,
    login: start,
  });
  return { flow, login, lifetime, start };
}
const idle = { pending: false, error: null };

describe("a deliberate sign-in from a public page", () => {
  it("starts one SDK redirect at a time while a view is connected", async () => {
    const { flow, login, start } = setup();
    flow.start("/subscriptions/manage");
    expect(start).not.toHaveBeenCalled();
    flow.connect();
    flow.start("/subscriptions/manage");
    flow.start("/subscriptions/manage");
    expect(start).toHaveBeenCalledExactlyOnceWith("/subscriptions/manage");
    expect(flow.state.get()).toEqual({ pending: true, error: null });
    login.resolve();
    await vi.waitFor(() => expect(flow.state.get()).toEqual(idle));
  });

  it("presents a failed redirect and clears its result on a later visit", async () => {
    const { flow, login } = setup();
    flow.connect();
    flow.start("/");
    login.reject(new Error("Provider navigation failed"));
    await vi.waitFor(() => expect(flow.state.get().pending).toBe(false));
    expect(flow.state.get().error).toBeTruthy();
    flow.disconnect();
    flow.connect();
    expect(flow.state.get()).toEqual(idle);
  });

  it("keeps the operation app-owned and releases it when no view remains", async () => {
    const { flow, login, start } = setup();
    flow.connect();
    flow.connect();
    flow.start("/");
    flow.disconnect();
    expect(flow.state.get().pending).toBe(true);
    flow.disconnect();
    login.resolve();
    await vi.waitFor(() => {
      flow.connect();
      try {
        expect(flow.state.get()).toEqual(idle);
      } finally {
        flow.disconnect();
      }
    });
    expect(start).toHaveBeenCalledTimes(1);
  });

  it("does not acquire a redirect after the application ends", () => {
    const { flow, lifetime, start } = setup();
    flow.connect();
    lifetime.abort();
    flow.start("/");
    expect(start).not.toHaveBeenCalled();
  });
});
