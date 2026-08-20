// @vitest-environment jsdom
import "core-js/actual/typed-array/to-base64";
import { fireEvent, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import {
  createKonobanguFixture,
  deferredReply,
} from "@/__test__/support/fixture";
import { renderInEnvironment } from "@/__test__/support/render";
import { SignedOutRoute } from "../signed-out";

describe("public sign-in", () => {
  it("leaves the actual PKCE redirect and callback to the SDK", async () => {
    // Normalize the host WebCrypto ArrayBuffer only at the JSDOM realm boundary.
    const digest = crypto.subtle.digest.bind(crypto.subtle);
    vi.spyOn(crypto.subtle, "digest").mockImplementation(
      async (...arguments_) =>
        new Uint8Array(await digest(...arguments_)).slice().buffer,
    );
    const app = createKonobanguFixture({
      initialPath: "/auth/signed-out",
      realSdkAuthorization: true,
    });
    const navigate = vi.spyOn(app.router, "navigate").mockResolvedValue();
    renderInEnvironment(app, <SignedOutRoute />, { strict: true });
    fireEvent.click(screen.getByRole("button", { name: "Sign in again" }));
    await waitFor(() => expect(navigate).toHaveBeenCalledTimes(1));
    const target = navigate.mock.calls[0]?.[0];
    if (!target?.href) throw new Error("An SDK external redirect is required");
    const url = new URL(target.href);
    expect(url.origin).toBe("https://idp.example");
    expect(url.pathname).toBe("/auth");
    expect(url.searchParams.get("code_challenge_method")).toBe("S256");
    expect(app.router.history.location.pathname).toBe("/auth/signed-out");
    expect(document.querySelector("dialog")).toBeNull();
  });

  it("shares an attempt and presents its error without fabricating a login", async () => {
    const app = createKonobanguFixture({ initialPath: "/auth/signed-out" });
    const reply = deferredReply<void>();
    const login = vi.spyOn(app.auth, "login").mockImplementation(reply.request);
    const navigate = vi.spyOn(app.router, "navigate");
    renderInEnvironment(app, <SignedOutRoute />, { strict: true });
    const button = screen.getByRole("button", { name: "Sign in again" });
    fireEvent.click(button);
    fireEvent.click(button);
    expect(login).toHaveBeenCalledExactlyOnceWith("/subscriptions/manage");
    reply.reject(new Error("Provider navigation failed"));
    expect(await screen.findByRole("alert")).toBeTruthy();
    expect(app.auth.isAuthenticated.get()).toBe(false);
    expect(navigate).not.toHaveBeenCalled();
  });

  it("keeps a pending SDK operation app-owned after the public view unmounts", async () => {
    const app = createKonobanguFixture({ initialPath: "/auth/signed-out" });
    const reply = deferredReply<void>();
    const login = vi.spyOn(app.auth, "login").mockImplementation(reply.request);
    const view = renderInEnvironment(app, <SignedOutRoute />, { strict: true });
    fireEvent.click(screen.getByRole("button", { name: "Sign in again" }));
    await reply.requested;
    view.unmount();
    expect(app.auth.lifetimeSignal.aborted).toBe(false);
    reply.resolve();
    expect(login).toHaveBeenCalledTimes(1);
  });
});
