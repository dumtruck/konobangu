import { readdir, readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";
import {
  type BrowserContext,
  type BrowserContextOptions,
  chromium,
  expect,
  firefox,
  type Page,
  type Route,
  webkit,
} from "@playwright/test";
import { type } from "arktype";
import { test } from "./support/deployment-fixture.ts";
import {
  openNativeFeed,
  privateResourcePrefix,
} from "./support/native-browser.ts";
import { chromiumFixtureArgs } from "./support/trusted-tls.mts";

const BootstrapInspection = type({
  nativeTemporal: "boolean",
  temporalPreserved: "boolean",
  temporalReady: "boolean",
  resourcesNative: "boolean",
  resourcesReady: "boolean",
  resourcesPreserved: "boolean",
});

test.setTimeout(120_000);

async function observeCapabilities(page: Page) {
  await page.addInitScript(() => {
    const temporal: unknown = Reflect.get(globalThis, "Temporal");
    const resourceKeys = [
      "DisposableStack",
      "AsyncDisposableStack",
      "SuppressedError",
    ];
    const symbols = [Symbol.dispose, Symbol.asyncDispose];
    const resources: unknown[] = resourceKeys.map((key) =>
      Reflect.get(globalThis, key),
    );
    Object.defineProperty(globalThis, "__konobanguBootstrapInspection", {
      value: () => {
        const currentTemporal: unknown = Reflect.get(globalThis, "Temporal");
        const currentResources: unknown[] = resourceKeys.map((key) =>
          Reflect.get(globalThis, key),
        );
        return {
          nativeTemporal: temporal !== undefined,
          temporalPreserved: currentTemporal === temporal,
          temporalReady: currentTemporal !== undefined,
          resourcesNative:
            resources.every((value) => value !== undefined) &&
            symbols.every((value) => value !== undefined),
          resourcesReady:
            currentResources.every((value) => value !== undefined) &&
            Symbol.dispose !== undefined &&
            Symbol.asyncDispose !== undefined,
          resourcesPreserved:
            resources.every(
              (value, index) =>
                value === undefined || value === currentResources[index],
            ) &&
            symbols.every(
              (value, index) =>
                value === undefined ||
                value === [Symbol.dispose, Symbol.asyncDispose][index],
            ),
        };
      },
    });
  });
}
async function inspectCapabilities(page: Page) {
  return BootstrapInspection.assert(
    await page.evaluate(() => {
      const inspect: unknown = Reflect.get(
        globalThis,
        "__konobanguBootstrapInspection",
      );
      if (typeof inspect !== "function")
        throw new Error("Bootstrap observation was not installed");
      return inspect();
    }),
  );
}
async function productionChunks(directory: string) {
  const files = await readdir(join(directory, "webui/assets"));
  const chunk = (pattern: RegExp) => {
    const matches = files.filter((file) => pattern.test(file));
    if (matches.length !== 1)
      throw new Error(`Expected one production capability chunk: ${pattern}`);
    return matches[0];
  };
  return {
    temporal: chunk(/^global(?:\.esm)?-[^/]+\.js$/),
    resources: chunk(/^resource-polyfill-[^/]+\.js$/),
    entry: chunk(/^entry-[^/]+\.js$/),
  };
}
async function showRealPublicPage(page: Page, origin: string) {
  await page.goto(`${origin}/auth/signed-out`);
  await expect(
    page.getByRole("heading", { name: "Local access has ended", exact: true }),
  ).toBeVisible();
}
function deferredRoute() {
  const hit = Promise.withResolvers<void>();
  const release = Promise.withResolvers<void>();
  return {
    requested: hit.promise,
    release: release.resolve,
    async hold(route: Route) {
      hit.resolve();
      await release.promise;
      await route.continue();
    },
  };
}

test("production cold bootstrap preserves native Temporal and resource primitives without loading fallback chunks", async ({
  page,
  deployment,
}) => {
  const chunks = await productionChunks(deployment.descriptor.directory);
  await observeCapabilities(page);
  const requests: string[] = [];
  page.on("request", (request) => requests.push(request.url()));
  await showRealPublicPage(page, deployment.descriptor.bundledUrl);
  const capabilities = await inspectCapabilities(page);
  expect(capabilities).toMatchObject({
    nativeTemporal: true,
    temporalPreserved: true,
    temporalReady: true,
    resourcesReady: true,
    resourcesPreserved: true,
  });
  expect(requests.some((url) => url.endsWith(`/${chunks.temporal}`))).toBe(
    false,
  );
  expect(requests.some((url) => url.endsWith(`/${chunks.resources}`))).toBe(
    !capabilities.resourcesNative,
  );
  await expect(page.locator("#app")).toHaveCount(1);
});

test("missing production capabilities and deployment read start concurrently and gate the real entry", async ({
  deployment,
}) => {
  await using ownership = new AsyncDisposableStack();
  const browser = ownership.adopt(
    await chromium.launch({
      executablePath: await deployment.baselineChromiumExecutable(),
      args: [...chromiumFixtureArgs()],
    }),
    (browser) => browser.close(),
  );
  const context = ownership.adopt(await browser.newContext(), (context) =>
    context.close(),
  );
  const page = await context.newPage();
  expect(
    await page.evaluate(() => typeof Reflect.get(globalThis, "Temporal")),
  ).toBe("undefined");
  const resourcesNative = await page.evaluate(
    () =>
      ["DisposableStack", "AsyncDisposableStack", "SuppressedError"].every(
        (key) => Reflect.get(globalThis, key) !== undefined,
      ) &&
      Symbol.dispose !== undefined &&
      Symbol.asyncDispose !== undefined,
  );
  const origin = deployment.descriptor.bundledUrl;
  const chunks = await productionChunks(deployment.descriptor.directory);
  await observeCapabilities(page);
  const temporal = deferredRoute();
  const resources = deferredRoute();
  const runtimeConfig = deferredRoute();
  using cleanup = new DisposableStack();
  cleanup.defer(temporal.release);
  cleanup.defer(resources.release);
  cleanup.defer(runtimeConfig.release);
  await page.route(`**/assets/${chunks.temporal}`, temporal.hold);
  await page.route(`**/assets/${chunks.resources}`, resources.hold);
  await page.route("**/konobangu.config.json", runtimeConfig.hold);
  const requests: string[] = [];
  page.on("request", (request) => requests.push(request.url()));
  const navigation = page.goto(`${origin}/auth/signed-out`);
  // Missing polyfills and the runtime configuration stay in flight together; serial loading hangs here.
  await Promise.all([
    temporal.requested,
    ...(resourcesNative ? [] : [resources.requested]),
    runtimeConfig.requested,
  ]);
  expect(requests.some((url) => url.endsWith(`/${chunks.entry}`))).toBe(false);
  await expect(page.getByRole("heading")).toHaveCount(0);
  const authRead = page.waitForResponse((response) =>
    response.url().endsWith("/api/auth/config"),
  );
  runtimeConfig.release();
  expect((await authRead).ok()).toBe(true);
  temporal.release();
  await expect
    .poll(async () => (await inspectCapabilities(page)).temporalReady)
    .toBe(true);
  if (!resourcesNative) {
    expect((await inspectCapabilities(page)).resourcesReady).toBe(false);
    expect(requests.some((url) => url.endsWith(`/${chunks.entry}`))).toBe(
      false,
    );
  }
  resources.release();
  await navigation;
  await expect(
    page.getByRole("heading", { name: "Local access has ended", exact: true }),
  ).toBeVisible();
  expect(await inspectCapabilities(page)).toMatchObject({
    nativeTemporal: false,
    temporalPreserved: false,
    resourcesNative,
    temporalReady: true,
    resourcesReady: true,
    resourcesPreserved: true,
  });
  expect(
    requests.filter((url) => url.endsWith(`/${chunks.entry}`)),
  ).toHaveLength(1);
});

test("a failed real production entry renders independent Retry and a fresh single application", async ({
  page,
  deployment,
}) => {
  const chunks = await productionChunks(deployment.descriptor.directory);
  let failures = 0;
  await page.route(`**/assets/${chunks.entry}`, async (route) => {
    if (failures++ === 0) await route.abort("failed");
    else await route.continue();
  });
  await page.goto(`${deployment.descriptor.bundledUrl}/auth/signed-out`);
  await expect(
    page.getByRole("button", { name: "Retry", exact: true }),
  ).toBeVisible();
  await expect(
    page.getByText("Unable to start Konobangu.", { exact: false }),
  ).toBeVisible();
  await expect(page.getByRole("heading")).toHaveCount(0);
  await page.getByRole("button", { name: "Retry", exact: true }).click();
  await expect(
    page.getByRole("heading", { name: "Local access has ended", exact: true }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Retry", exact: true }),
  ).toHaveCount(0);
  await expect(page.locator("#app")).toHaveCount(1);
  expect(failures).toBe(2);
});

async function privateText(
  page: Page,
  prefix: string,
  subscriber: number,
  authorization?: string,
) {
  return page.evaluate(
    async ({ prefix, subscriber, authorization }) => {
      const response = await fetch(
        `${prefix}static/subscribers/${subscriber}/fixture-private.txt`,
        {
          headers: authorization ? { Authorization: authorization } : {},
          credentials: "omit",
        },
      );
      return {
        status: response.status,
        body: await response.text(),
        cache: response.headers.get("cache-control"),
      };
    },
    { prefix, subscriber, authorization },
  );
}
async function signInOnEngine(page: Page, origin: string, api: string) {
  const profile = page.waitForResponse(
    (response) =>
      response.url() === `${api}/api/auth/user-info` &&
      response.status() === 200,
  );
  // A navigation failure can close the page before this response is awaited.
  void profile.catch(() => {});
  await page.goto(`${origin}/subscriptions/manage`);
  const consent = page.getByRole("button", {
    name: "Sign in as A",
    exact: true,
  });
  await expect(consent).toBeVisible({ timeout: 30_000 });
  await expect
    .poll(
      async () => {
        if (await consent.isVisible()) await consent.click();
        return (
          new URL(page.url()).origin === origin &&
          new URL(page.url()).pathname === "/subscriptions/manage"
        );
      },
      { timeout: 30_000 },
    )
    .toBe(true);
  expect((await profile).ok()).toBe(true);
  // The callback can update the URL before its destination document is ready.
  // Starting another navigation at that point aborts Firefox's pending load.
  await expect(
    page.getByRole("heading", {
      name: "Subscription Management",
      exact: true,
    }),
  ).toBeVisible();
  expect(new URL(page.url()).searchParams.has("code")).toBe(false);
  expect(new URL(page.url()).searchParams.has("state")).toBe(false);
}

/**
 * Firefox and WebKit run Playwright's bundled builds through Playwright's own
 * options: the fixture proxy maps the test names, and these engines, which
 * have no per-launch trust anchor, skip certificate errors for the fixture's
 * short-lived certificate.
 */
async function engineContext(
  browserType: typeof firefox | typeof webkit,
  blockThirdPartyCookies: boolean,
  nameProxy: string | null,
  ownership: AsyncDisposableStack,
  contextOptions: BrowserContextOptions,
) {
  if (nameProxy === null)
    throw new Error("Cross-engine tests require the fixture name proxy");
  const browser = ownership.adopt(
    await browserType.launch({
      firefoxUserPrefs: blockThirdPartyCookies
        ? { "network.cookie.cookieBehavior": 1 }
        : {},
    }),
    (browser) => browser.close(),
  );
  return ownership.adopt(
    await browser.newContext({
      ...contextOptions,
      proxy: { server: nameProxy },
      ignoreHTTPSErrors: true,
    }),
    (context) => context.close(),
  );
}

async function routeProviderLoginChallenge(
  context: BrowserContext,
  issuer: string,
  realm: string,
) {
  const login = new URL(`${issuer}/login`);
  const matches = (url: URL) =>
    url.origin === login.origin && url.pathname === login.pathname;
  const handle = async (route: Route) => {
    const request = route.request();
    if (
      !request.isNavigationRequest() ||
      request.resourceType() !== "document" ||
      request.method() !== "GET" ||
      request.frame().parentFrame() !== null ||
      request.headers().authorization !== undefined
    )
      throw new Error("Provider login must be a top-level document request");
    const challenge = await route.fetch({ maxRedirects: 0 });
    try {
      if (
        challenge.status() !== 401 ||
        challenge.headers()["www-authenticate"] !==
          `Basic realm=${JSON.stringify(realm)}`
      )
        throw new Error("Provider login did not issue the expected challenge");
    } finally {
      await challenge.dispose();
    }
    await route.continue({
      headers: {
        ...(await request.allHeaders()),
        authorization: `Basic ${Buffer.from("fixture-admin:fixture-admin-password").toString("base64")}`,
      },
    });
  };
  await context.route(matches, handle);
  return {
    async navigate(page: Page, action: () => Promise<unknown>) {
      const authorization = new URL(`${issuer}/authorize`);
      const redirect = page.waitForResponse(
        (response) =>
          response.url().split("?", 1)[0] === authorization.href &&
          response.request().isNavigationRequest() &&
          response.request().frame().parentFrame() === null &&
          response.status() === 302,
      );
      await action();
      const response = await redirect;
      const location = response.headers().location;
      if (!location) throw new Error("Provider authorize did not redirect");
      const target = new URL(location, response.url());
      if (
        !matches(target) ||
        target.hash ||
        target.searchParams.size !== 1 ||
        !target.searchParams.get("pending")
      )
        throw new Error("Provider authorize did not select its login endpoint");
      // Playwright routes only the first URL of a redirect chain. Reopen the
      // actual login target as a new document navigation so its route can answer.
      const authenticated = page.waitForResponse(
        (result) => result.url() === target.href,
      );
      await page.goto(target.href);
      if ((await authenticated).status() !== 302)
        throw new Error("Provider login did not return its code redirect");
    },
    [Symbol.asyncDispose]: () => context.unroute(matches, handle),
  };
}

for (const [engine, browserType] of [
  ["Firefox", firefox],
  ["WebKit", webkit],
] as const) {
  test(`${engine} runs cross-site public-client OIDC and a native private resource token`, async ({
    deployment,
  }) => {
    await using ownership = new AsyncDisposableStack();
    const context = await engineContext(
      browserType,
      true,
      deployment.descriptor.nameProxy,
      ownership,
      {},
    );
    const page = await context.newPage();
    const { crossSiteUrl, crossSiteApiUrl, issuer } = deployment.descriptor;
    let authorization: string | undefined;
    const publicRequests: {
      origin: string;
      auth: boolean;
      cookie: boolean;
      resource: boolean;
    }[] = [];
    page.on("request", (request) => {
      if (
        request.url().startsWith(issuer) ||
        request.url().startsWith(crossSiteApiUrl)
      ) {
        const headers = request.headers();
        if (headers.authorization?.startsWith("Bearer "))
          authorization = headers.authorization;
        publicRequests.push({
          origin: new URL(request.url()).origin,
          auth: headers.authorization !== undefined,
          cookie: headers.cookie !== undefined,
          resource: new URL(request.url()).searchParams.has("resource_token"),
        });
      }
    });
    await signInOnEngine(page, crossSiteUrl, crossSiteApiUrl);
    await page.goto(`${crossSiteUrl}/subscriptions/detail/6000`);
    const prefix = await privateResourcePrefix(page);
    await openNativeFeed(page);
    expect(
      await page.evaluate(
        async () => (await navigator.serviceWorker.getRegistrations()).length,
      ),
    ).toBe(0);
    expect(await privateText(page, prefix, 1001, authorization)).toMatchObject({
      status: 200,
      body: "fixture-private-content-1001",
      cache: "private, no-cache, no-transform",
    });
    expect(await privateText(page, prefix, 1002, authorization)).toMatchObject({
      status: 403,
    });
    expect(
      publicRequests.some(
        (request) => request.origin === crossSiteApiUrl && request.auth,
      ),
    ).toBe(true);
    expect(
      publicRequests.filter((request) => request.resource).length,
    ).toBeGreaterThan(0);
    expect(
      publicRequests
        .filter((request) => request.resource)
        .every((request) => !request.auth),
    ).toBe(true);
    // IdP's own top-level login cookie is permitted; API requests do not use it.
    expect(
      publicRequests
        .filter((request) => request.origin === crossSiteApiUrl)
        .some((request) => request.cookie),
    ).toBe(false);
  });

  test(`${engine} authenticates only at the bundled provider and keeps API/resource requests cookie-free`, async ({
    deployment,
  }) => {
    await using ownership = new AsyncDisposableStack();
    const {
      bundledUrl,
      bundledApiUrl,
      bundledIssuer,
      crossSiteApiUrl,
      issuer,
    } = deployment.descriptor;
    const context = await engineContext(
      browserType,
      true,
      deployment.descriptor.nameProxy,
      ownership,
      engine === "Firefox"
        ? {
            httpCredentials: {
              username: "fixture-admin",
              password: "fixture-admin-password",
              origin: new URL(bundledIssuer).origin,
            },
          }
        : {},
    );
    // The isolated driver repro confirms WebKit automation credentials reach
    // unrelated fetches even with credentials: "omit". Related upstream fixes:
    // https://github.com/microsoft/playwright/pull/42313.
    // Answer only the real login challenge; token/API requests stay untouched.
    const routedLogin =
      engine === "WebKit"
        ? ownership.use(
            await routeProviderLoginChallenge(
              context,
              bundledIssuer,
              deployment.descriptor.bundledRealm,
            ),
          )
        : undefined;
    const page = await context.newPage();
    let bearer: string | undefined;
    const requests: {
      origin: string;
      pathname: string;
      authorization: string | undefined;
      cookie: boolean;
    }[] = [];
    page.on("request", (request) => {
      const url = new URL(request.url());
      const headers = request.headers();
      if (
        url.origin === bundledApiUrl &&
        headers.authorization?.startsWith("Bearer ")
      )
        bearer = headers.authorization;
      requests.push({
        origin: url.origin,
        pathname: url.pathname,
        authorization: headers.authorization,
        cookie: headers.cookie !== undefined,
      });
    });
    const baseline = deployment.providerLoginChallenges().length;
    const open = () => page.goto(`${bundledUrl}/subscriptions/detail/5000`);
    if (routedLogin) await routedLogin.navigate(page, open);
    else await open();
    await expect(
      page.getByText("Media browser fixture", { exact: true }),
    ).toBeVisible({ timeout: 30_000 });
    await expect(page.locator('input[type="password"]')).toHaveCount(0);
    await expect(page.getByRole("dialog")).toHaveCount(0);
    const challenges = deployment.providerLoginChallenges().slice(baseline);
    expect(challenges.length).toBeGreaterThanOrEqual(1);
    expect(
      challenges.every(
        (event) =>
          event.url === `${bundledIssuer}/login` &&
          event.destination === "document",
      ),
    ).toBe(true);
    expect(challenges[0]?.challenge).toContain(
      `Basic realm=${JSON.stringify(deployment.descriptor.bundledRealm)}`,
    );
    const prefix = await privateResourcePrefix(page);
    await openNativeFeed(page);
    if (!bearer)
      throw new Error(
        "SDK did not attach the bundled access token to API requests",
      );
    expect(await privateText(page, prefix, 1, bearer)).toMatchObject({
      status: 200,
      body: "fixture-private-content-1",
      cache: "private, no-cache, no-transform",
    });
    expect(await privateText(page, prefix, 1001, bearer)).toMatchObject({
      status: 403,
    });
    await page.getByRole("button", { name: "User menu", exact: true }).click();
    await page.getByRole("menuitem", { name: "Sign out", exact: true }).click();
    await expect(page).toHaveURL(`${bundledUrl}/auth/signed-out`);
    await expect(
      page.getByRole("heading", {
        name: "Local access has ended",
        exact: true,
      }),
    ).toBeVisible();
    const signInAgain = () =>
      page.getByRole("button", { name: "Sign in again" }).click();
    if (routedLogin) await routedLogin.navigate(page, signInAgain);
    else await signInAgain();
    await expect(page).toHaveURL(`${bundledUrl}/subscriptions/manage`);
    await expect(
      page.getByRole("heading", {
        name: "Subscription Management",
        exact: true,
      }),
    ).toBeVisible();
    expect(
      requests
        .filter((request) => request.origin === bundledUrl)
        .some((request) => request.authorization !== undefined),
    ).toBe(false);
    expect(
      requests
        .filter((request) => request.origin === bundledApiUrl)
        .some(
          (request) =>
            request.authorization?.startsWith("Basic ") || request.cookie,
        ),
    ).toBe(false);
    expect(
      requests
        .filter((request) => request.pathname.endsWith("/token"))
        .some(
          (request) => request.authorization !== undefined || request.cookie,
        ),
    ).toBe(false);
    const foreign = await context.request.get(
      `${crossSiteApiUrl}/api/auth/user-info`,
    );
    expect(foreign.status()).toBe(401);
    const external = await context.request.get(
      `${issuer}/.well-known/openid-configuration`,
    );
    expect(external.ok()).toBe(true);
  });
}

test("Chromium 140 loads required polyfills before real media and cron pages", async ({
  deployment,
}) => {
  await using ownership = new AsyncDisposableStack();
  const executablePath = await deployment.baselineChromiumExecutable();
  const browser = ownership.adopt(
    await chromium.launch({
      executablePath,
      args: [...chromiumFixtureArgs(), "--test-third-party-cookie-phaseout"],
    }),
    (browser) => browser.close(),
  );
  expect(browser.version().split(".")[0]).toBe("140");
  const { bundledUrl, bundledIssuer } = deployment.descriptor;
  const context = ownership.adopt(
    await browser.newContext({
      httpCredentials: {
        username: "fixture-admin",
        password: "fixture-admin-password",
        origin: new URL(bundledIssuer).origin,
      },
    }),
    (context) => context.close(),
  );
  const page = await context.newPage();
  expect(
    await page.evaluate(() => typeof Reflect.get(globalThis, "Temporal")),
  ).toBe("undefined");
  await observeCapabilities(page);
  const chunks = await productionChunks(deployment.descriptor.directory);
  const requests: string[] = [];
  page.on("request", (request) => requests.push(request.url()));
  const temporal = deferredRoute();
  using release = new DisposableStack();
  release.defer(temporal.release);
  await page.route(`**/assets/${chunks.temporal}`, temporal.hold);
  const navigation = page.goto(`${bundledUrl}/auth/signed-out`);
  await temporal.requested;
  expect(requests.some((url) => url.endsWith(`/${chunks.entry}`))).toBe(false);
  temporal.release();
  await navigation;
  await expect(
    page.getByRole("heading", { name: "Local access has ended", exact: true }),
  ).toBeVisible();
  expect(await inspectCapabilities(page)).toMatchObject({
    nativeTemporal: false,
    temporalReady: true,
    resourcesReady: true,
    resourcesPreserved: true,
  });
  expect(
    requests.filter((url) => url.endsWith(`/${chunks.temporal}`)),
  ).toHaveLength(1);
  expect(
    requests.filter((url) => url.endsWith(`/${chunks.resources}`)),
  ).toHaveLength((await inspectCapabilities(page)).resourcesNative ? 0 : 1);
  expect(
    requests.some((url) => /\/(?:monaco-editor-|graphql-api\.lazy-)/.test(url)),
  ).toBe(false);
  const coldPaths = [
    ...new Set(
      requests
        .map((url) => new URL(url).pathname)
        .filter((path) => path.startsWith("/assets/") && path.endsWith(".js")),
    ),
  ];
  const scripts = [];
  for (const path of coldPaths) {
    const bytes = (
      await readFile(
        join(deployment.descriptor.directory, "webui", path.slice(1)),
      )
    ).byteLength;
    expect(bytes).toBeLessThanOrEqual(3_000_000);
    scripts.push({ path, bytes });
  }
  await page.goto(`${bundledUrl}/subscriptions/detail/5000`);
  await expect(
    page.getByText("Media browser fixture", { exact: true }),
  ).toBeVisible();
  await privateResourcePrefix(page);
  expect(
    await page
      .getByRole("img", { name: "Poster", exact: true })
      .first()
      .evaluate((image: HTMLImageElement) => image.naturalWidth),
  ).toBe(256);
  await page.goto(`${bundledUrl}/tasks/cron/detail/5020`);
  await expect(page.getByRole("heading", { name: /^Next Runs/ })).toBeVisible();
  await expect(page.getByText(/Invalid timestamp/)).toHaveCount(0);
  await writeFile(
    join(deployment.descriptor.directory, "capabilities-baseline.json"),
    JSON.stringify(
      {
        browser: browser.version(),
        capabilities: await inspectCapabilities(page),
        coldJavascriptBytes: scripts.reduce(
          (sum, entry) => sum + entry.bytes,
          0,
        ),
        scripts,
      },
      null,
      2,
    ),
  );
});
