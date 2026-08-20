// @vitest-environment jsdom
import { act, fireEvent, screen } from "@testing-library/react";
import { EMPTY } from "rxjs";
import { describe, expect, it, vi } from "vitest";
import { createSignedInFixture } from "@/__test__/support/fixture";
import { renderInEnvironment } from "@/__test__/support/render";
import { parseRuntimeConfig } from "@/app/config/runtime";
import { ImageBinding, ResourceBinding } from "@/app/resource/resource-binding";
import {
  type NativeResource,
  RESOURCE_SERVICE,
  type ResourceService,
} from "@/app/resource/resource-grants";
import { Img } from "../img";

const source =
  "https://api.test/api/static/subscribers/1/image/posters/cover.png";
/**
 * Grant derivation is owned by the resource service suite. This suite replaces
 * that one unit through the environment and keeps the real identity, API
 * target and viewport trait around the image.
 */
async function setup() {
  const release = vi.fn();
  const acquire = vi.fn(
    async (input: string): Promise<NativeResource> => ({
      url: `${input}&resource_token=child`,
      expiresAt: 300_000,
    }),
  );
  const app = await createSignedInFixture({
    config: parseRuntimeConfig(
      { version: 1, apiBaseUrl: "https://api.test/api/" },
      { origin: document.location.origin, allowInsecureLoopback: true },
    ),
    providers: [
      {
        provide: RESOURCE_SERVICE,
        useFactory: (): ResourceService => ({
          acquire,
          bindImage: (input) =>
            new ImageBinding({
              input,
              private: input.startsWith(app.api.url("static/")),
              acquire,
              activate: () => ({ [Symbol.dispose]: release }),
              identityChanges: app.auth.identityChanges,
            }),
          bind: (input) =>
            new ResourceBinding({
              input,
              private: false,
              observe: () => EMPTY,
            }),
          isPrivate: (input) => input.startsWith(app.api.url("static/")),
          activate: () => ({ [Symbol.dispose]: release }),
          nearViewport: (element, run) =>
            app.environment.viewport.nearViewport(element, run),
        }),
        deps: [],
      },
    ],
  });
  const element = (src: string, loading: "eager" | "lazy") => (
    <Img src={src} loading={loading} alt="cover" />
  );
  return {
    ...app,
    acquire,
    release,
    viewport: app.environment.viewport,
    render: (src = source, loading: "eager" | "lazy" = "eager") => {
      const view = renderInEnvironment(app, element(src, loading));
      return {
        ...view,
        show: (next = source, nextLoading = loading) =>
          view.rerender(element(next, nextLoading)),
      };
    },
  };
}
describe("native private images", () => {
  it("signs lazy images only near the viewport and owns observer cleanup", async () => {
    const state = await setup();
    const view = state.render(source, "lazy");
    const image = screen.getByAltText<HTMLImageElement>("cover");
    expect(image.getAttribute("src")).toBeNull();
    expect(state.acquire).not.toHaveBeenCalled();
    await act(async () => state.viewport.approach());
    expect(image.src).toContain("resource_token=child");
    expect(image.crossOrigin).toBe("anonymous");
    expect(state.acquire).toHaveBeenCalledTimes(1);
    expect(state.viewport.observedCount).toBe(0);
    fireEvent.load(image);
    view.show();
    expect(state.acquire).toHaveBeenCalledTimes(1);
    view.unmount();
    expect(state.release).toHaveBeenCalledTimes(1);
  });
  it("recovers once through a forced grant renewal, then resets on a different source", async () => {
    const state = await setup();
    const view = state.render();
    await act(async () => {});
    const image = screen.getByAltText<HTMLImageElement>("cover");
    await act(async () => fireEvent.error(image));
    expect(state.acquire).toHaveBeenLastCalledWith(
      `${source}?optimize=accept`,
      "renew",
    );
    const recovered = screen.getByAltText<HTMLImageElement>("cover");
    expect(recovered).not.toBe(image);
    await act(async () => fireEvent.error(recovered));
    expect(state.acquire).toHaveBeenCalledTimes(2);
    expect(screen.getByRole("alert").textContent).toContain(
      "Image unavailable",
    );
    view.show(source.replace("cover.png", "other.png"));
    await act(async () => {});
    await act(async () => fireEvent.error(screen.getByAltText("cover")));
    expect(state.acquire).toHaveBeenCalledTimes(4);
  });
  it("removes the old identity URL and derives against the new identity", async () => {
    const state = await setup();
    state.render();
    await act(async () => {});
    expect(screen.getByAltText("cover").getAttribute("src")).toContain(
      "resource_token",
    );
    const next = Promise.withResolvers<NativeResource>();
    state.acquire.mockImplementationOnce(() => next.promise);
    await act(() => state.identity("B"));
    expect(screen.getByAltText("cover").getAttribute("src")).toBeNull();
    await act(async () =>
      next.resolve({ url: `${source}?resource_token=new`, expiresAt: 300_000 }),
    );
    expect(screen.getByAltText("cover").getAttribute("src")).toContain(
      "resource_token=new",
    );
  });
  it("shows final acquisition failures and supports an explicit per-image retry", async () => {
    const state = await setup();
    state.acquire.mockRejectedValueOnce(new Error("offline"));
    state.render();
    await act(async () => {});
    expect(screen.getByRole("alert").textContent).toContain(
      "Image unavailable",
    );
    expect(screen.getByAltText("cover").getAttribute("src")).toBeNull();
    await act(async () =>
      fireEvent.click(
        screen.getByRole("button", { name: "Retry image cover" }),
      ),
    );
    expect(screen.queryByRole("alert")).toBeNull();
    expect(screen.getByAltText("cover").getAttribute("src")).toContain(
      "resource_token=child",
    );
    expect(state.acquire).toHaveBeenCalledTimes(2);
  });
  it("keeps a new automatic retry element empty until its newly derived URL resolves", async () => {
    const state = await setup();
    state.render();
    await act(async () => {});
    const previous = screen.getByAltText<HTMLImageElement>("cover");
    const previousUrl = previous.src;
    const next = Promise.withResolvers<NativeResource>();
    state.acquire.mockImplementationOnce(() => next.promise);
    await act(async () => fireEvent.error(previous));
    const current = screen.getByAltText<HTMLImageElement>("cover");
    expect(current).not.toBe(previous);
    expect(current.getAttribute("src")).toBeNull();
    expect(state.acquire).toHaveBeenCalledTimes(2);
    await act(async () =>
      next.resolve({ url: previousUrl, expiresAt: 300_000 }),
    );
    expect(current.src).toBe(previousUrl);
  });
  it("clears an already loaded resource before a manual retry changes the native element", async () => {
    const state = await setup();
    state.render();
    await act(async () => {});
    const previous = screen.getByAltText<HTMLImageElement>("cover");
    const previousUrl = previous.src;
    // The binding publishes its state through a signal, one tick later.
    await act(async () => {
      fireEvent.load(previous);
      fireEvent.error(previous);
    });
    const next = Promise.withResolvers<NativeResource>();
    state.acquire.mockImplementationOnce(() => next.promise);
    await act(async () =>
      fireEvent.click(
        screen.getByRole("button", { name: "Retry image cover" }),
      ),
    );
    const current = screen.getByAltText<HTMLImageElement>("cover");
    expect(current).not.toBe(previous);
    expect(current.getAttribute("src")).toBeNull();
    expect(screen.queryByRole("alert")).toBeNull();
    await act(async () =>
      next.resolve({ url: previousUrl, expiresAt: 300_000 }),
    );
    expect(current.src).toBe(previousUrl);
  });
});
