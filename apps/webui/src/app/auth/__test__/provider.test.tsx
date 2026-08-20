// @vitest-environment jsdom
import { act, fireEvent, screen } from "@testing-library/react";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";
import {
  createKonobanguFixture,
  createSignedInFixture,
  deferredReply,
  response,
} from "@/__test__/support/fixture";
import { renderInEnvironment } from "@/__test__/support/render";
import { AuthProvider, PrivateIdentityView } from "../provider";

function PrivateEditor() {
  const [draft, setDraft] = useState("");
  return (
    <input
      aria-label="Private draft"
      value={draft}
      onChange={(event) => setDraft(event.currentTarget.value)}
    />
  );
}
function StableRoot() {
  const [value, setValue] = useState("");
  return (
    <>
      <input
        aria-label="Public root marker"
        value={value}
        onChange={(event) => setValue(event.currentTarget.value)}
      />
      <PrivateIdentityView>
        <PrivateEditor />
      </PrivateIdentityView>
    </>
  );
}
const draft = () => screen.getByLabelText<HTMLInputElement>("Private draft");
const marker = () =>
  screen.getByLabelText<HTMLInputElement>("Public root marker");

describe("private view identity ownership", () => {
  it("preserves the private draft while the same verified identity is reloading", async () => {
    const app = await createSignedInFixture({
      subject: "Administrator",
    });
    renderInEnvironment(app, <StableRoot />);
    fireEvent.change(draft(), { target: { value: "Retained private draft" } });
    const reload = deferredReply<Response>();
    app.fetch.mockImplementationOnce(reload.request);
    let refreshing: ReturnType<typeof app.auth.refresh> | undefined;
    await act(async () => {
      refreshing = app.auth.refresh();
      await reload.requested;
    });
    expect(app.auth.isAuthenticated.get()).toBe(true);
    expect(draft().value).toBe("Retained private draft");
    await act(async () => {
      reload.resolve(app.principal());
      await refreshing;
    });
    expect(draft().value).toBe("Retained private draft");
  });
  it("remounts private form state on the same route while preserving the outer tree", async () => {
    const app = await createSignedInFixture();
    renderInEnvironment(app, <StableRoot />);
    fireEvent.change(draft(), { target: { value: "A private draft" } });
    fireEvent.change(marker(), { target: { value: "Stable router" } });
    await act(() => app.identity("B"));
    expect(draft().value).toBe("");
    expect(marker().value).toBe("Stable router");
    await act(() => app.auth.logout());
    expect(screen.queryByLabelText("Private draft")).toBeNull();
    expect(marker().value).toBe("Stable router");
  });
});

describe("authentication at the React root", () => {
  it("reads the identity once and revalidates routes only when the identity changes", async () => {
    const app = await createSignedInFixture();
    const before = app.requests({ path: "auth/user-info" }).length;
    const invalidate = vi.spyOn(app.router, "invalidate").mockResolvedValue();
    renderInEnvironment(
      app,
      <AuthProvider router={app.router}>content</AuthProvider>,
    );
    await vi.waitFor(() =>
      expect(app.requests({ path: "auth/user-info" })).toHaveLength(before + 1),
    );
    // The same principal read again is not an identity change.
    await act(() => app.auth.refresh());
    expect(invalidate).not.toHaveBeenCalled();
    await act(() => app.identity("B"));
    expect(invalidate).toHaveBeenCalledTimes(1);
    await act(() => app.auth.logout());
    expect(invalidate).toHaveBeenCalledTimes(2);
  });
  it("leaves the callback route to the SDK and presents a failed identity check", async () => {
    const callback = createKonobanguFixture({ initialPath: "/auth/callback" });
    renderInEnvironment(
      callback,
      <AuthProvider router={callback.router}>content</AuthProvider>,
    );
    expect(callback.fetch).not.toHaveBeenCalled();
    const failing = createKonobanguFixture({
      handler: () => response(503, { message: "unavailable" }),
    });
    renderInEnvironment(
      failing,
      <AuthProvider router={failing.router}>content</AuthProvider>,
    );
    expect(await screen.findByRole("alert")).toBeTruthy();
  });
});
