// @vitest-environment jsdom
import {
  RouterNavigationIntent,
  RouterNavigationMode,
  UriReferenceString,
} from "@securitydept/client";
import { takeFrontendOidcCallbackInputFromRouter } from "@securitydept/token-set-context-client/frontend-oidc-mode";
import { describe, expect, it, vi } from "vitest";
import { createKonobanguFixture } from "@/__test__/support/fixture";

describe("SDK navigation through the environment router", () => {
  it("cleans a real absolute callback URL through SDK replacement without appending its origin", async () => {
    const app = createKonobanguFixture({
      initialPath: "/auth/callback?code=code&state=state&keep=yes#anchor",
    });
    const initialHistoryLength = app.router.history.length;
    const sdkRouter = app.environment.router;
    expect(sdkRouter.currentUrl()?.toString()).toBe(
      `${window.location.origin}/auth/callback?code=code&state=state&keep=yes#anchor`,
    );
    const callback = await takeFrontendOidcCallbackInputFromRouter(sdkRouter);
    expect(callback).toEqual({
      searchParams: new URLSearchParams({ code: "code", state: "state" }),
    });
    expect(app.router.state.location.href).toBe(
      "/auth/callback?keep=yes#anchor",
    );
    expect(app.router.history.length).toBe(initialHistoryLength);
    expect(app.fetch).not.toHaveBeenCalled();
  });

  it("rejects a foreign origin for internal navigation and preserves external authorization navigation", async () => {
    const app = createKonobanguFixture();
    const sdkRouter = app.environment.router;
    await expect(
      sdkRouter.navigate({
        url: UriReferenceString.parse("https://foreign.example/private"),
        intent: RouterNavigationIntent.PostAuthRedirect,
        mode: RouterNavigationMode.Replace,
      }),
    ).rejects.toThrow("configured WebUI origin");
    const navigation = vi.spyOn(app.router, "navigate").mockResolvedValue();
    await sdkRouter.navigate({
      url: UriReferenceString.parse("https://idp.example/authorize?state=one"),
      intent: RouterNavigationIntent.AuthRedirect,
      mode: RouterNavigationMode.External,
    });
    expect(navigation).toHaveBeenCalledWith({
      href: "https://idp.example/authorize?state=one",
    });
  });
});
