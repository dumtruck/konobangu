import { type BrowserContext, expect, type Page } from "@playwright/test";

export async function nativeBasicChallenge(
  context: BrowserContext,
  page: Page,
  issuer: string,
) {
  await using setup = new AsyncDisposableStack();
  const issuerUrl = new URL(issuer);
  const loginPath = `${issuerUrl.pathname}/login`;
  const session = await context.newCDPSession(page);
  setup.defer(() => session.detach());
  const { frameTree } = await session.send("Page.getFrameTree");
  const documents = new Set<string>();
  const failures: unknown[] = [];
  const operations = new Set<Promise<void>>();
  let answered = 0;
  const capture = (operation: Promise<unknown>) => {
    const captured = operation
      .catch((error: unknown) => {
        // Navigation and Vite's reload can retire an intercepted request before
        // its queued continuation. This is a native cancellation, not a failure.
        if (
          error instanceof Error &&
          error.message.includes("Invalid InterceptionId")
        )
          return;
        failures.push(error);
      })
      .then(() => {})
      .finally(() => operations.delete(captured));
    operations.add(captured);
  };
  const onPaused: Parameters<typeof session.on<"Fetch.requestPaused">>[1] = (
    event,
  ) => {
    if (
      event.resourceType === "Document" &&
      event.frameId === frameTree.frame.id &&
      new URL(event.request.url).origin === issuerUrl.origin &&
      new URL(event.request.url).pathname === loginPath
    )
      documents.add(event.requestId);
    capture(
      session.send("Fetch.continueRequest", { requestId: event.requestId }),
    );
  };
  const onAuth: Parameters<typeof session.on<"Fetch.authRequired">>[1] = (
    event,
  ) => {
    const expected =
      documents.has(event.requestId) &&
      event.authChallenge.origin === issuerUrl.origin &&
      event.authChallenge.realm === "Konobangu Provider" &&
      answered === 0;
    if (expected) answered++;
    capture(
      session.send("Fetch.continueWithAuth", {
        requestId: event.requestId,
        authChallengeResponse: expected
          ? {
              response: "ProvideCredentials",
              username: "fixture-admin",
              password: "fixture-admin-password",
            }
          : { response: "CancelAuth" },
      }),
    );
  };
  session.on("Fetch.requestPaused", onPaused);
  session.on("Fetch.authRequired", onAuth);
  setup.defer(async () => {
    session.off("Fetch.requestPaused", onPaused);
    session.off("Fetch.authRequired", onAuth);
    await session.send("Fetch.disable");
    await Promise.all(operations);
    if (failures.length)
      throw new AggregateError(failures, "Native Basic challenge failed");
  });
  await session.send("Fetch.enable", { handleAuthRequests: true });
  const lifetime = setup.move();
  return {
    get answered() {
      return answered;
    },
    [Symbol.asyncDispose]: () => lifetime.disposeAsync(),
  };
}

export async function privateResourcePrefix(page: Page): Promise<string> {
  const poster = page.getByRole("img", { name: "Poster", exact: true }).first();
  await expect(poster).toBeVisible();
  await poster.scrollIntoViewIfNeeded();
  await expect
    .poll(() =>
      poster.evaluate((element) =>
        element instanceof HTMLImageElement
          ? element.complete && element.naturalWidth > 0
          : false,
      ),
    )
    .toBe(true);
  const source = await poster.getAttribute("src");
  if (!source)
    throw new Error("The real private Img must expose its loaded URI");
  const url = new URL(source, page.url());
  if (
    /^\/api\/static\/subscribers\/[1-9]\d*\/image\//.test(url.pathname) &&
    url.searchParams.has("resource_token")
  )
    return `${url.origin}/api/`;
  throw new Error(
    "The real private Img must use an attenuated native resource URL",
  );
}

export async function openNativeFeed(page: Page) {
  const link = page.locator('a[href*="/api/feeds/rss/"]').first();
  await expect(link).toBeVisible();
  const href = await link.getAttribute("href");
  if (!href) throw new Error("A feed link must have a bounded native URL");
  expect(new URL(href).searchParams.has("resource_token")).toBe(true);
  const response = page.context().waitForEvent("response", {
    predicate: (response) => response.url() === href,
  });
  const [popup, delivered] = await Promise.all([
    page.waitForEvent("popup"),
    response,
    link.click(),
  ]);
  expect(delivered.status()).toBe(200);
  expect(delivered.request().resourceType()).toBe("document");
  expect(delivered.headers()["content-type"]).toContain("application/xml");
  expect(delivered.headers()["referrer-policy"]).toBe("no-referrer");
  const xml = await delivered.text();
  expect(xml).toContain("<rss");
  expect(xml).not.toContain("resource_token");
  await popup.close();
}
