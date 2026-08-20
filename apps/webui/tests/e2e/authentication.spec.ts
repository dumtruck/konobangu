import { expect, type Page } from "@playwright/test";
import { type } from "arktype";
import { decodeProtectedHeader } from "jose";
import { test } from "./support/deployment-fixture.ts";
import {
  nativeBasicChallenge,
  openNativeFeed,
  privateResourcePrefix,
} from "./support/native-browser.ts";

test.describe.configure({ mode: "serial" });
test.setTimeout(90_000);

const privatePrefixes = new WeakMap<Page, string>();
const privateAuthorizations = new WeakMap<Page, string>();
const observedPages = new WeakSet<Page>();

async function signIn(page: Page, origin: string, account: "A" | "B") {
  const capture = (request: import("@playwright/test").Request) => {
    const authorization = request.headers().authorization;
    if (
      new URL(request.url()).pathname === "/api/graphql" &&
      authorization?.startsWith("Bearer ")
    )
      privateAuthorizations.set(page, authorization);
  };
  if (!observedPages.has(page)) {
    observedPages.add(page);
    page.on("request", capture);
    page.once("close", () => page.off("request", capture));
  }
  await page.goto(`${origin}/subscriptions/manage`);
  // The provider owns consent and may require a second interaction.
  await expect
    .poll(
      async () => {
        if (
          new URL(page.url()).origin === origin &&
          new URL(page.url()).pathname === "/subscriptions/manage" &&
          (await page.getByText(account, { exact: true }).first().isVisible())
        )
          return "ready";
        const consent = page.getByRole("button", {
          name: `Sign in as ${account}`,
          exact: true,
        });
        if (await consent.isVisible()) await consent.click();
        return "pending";
      },
      { timeout: 30_000 },
    )
    .toBe("ready");
  await expect(
    page.getByText("Completing sign-in…", { exact: true }),
  ).toHaveCount(0);
  expect(new URL(page.url()).searchParams.has("code")).toBe(false);
  expect(new URL(page.url()).searchParams.has("state")).toBe(false);
  await page.goto(
    `${origin}/subscriptions/detail/${account === "A" ? 6000 : 6001}`,
  );
  privatePrefixes.set(page, await privateResourcePrefix(page));
}

async function privateRequest(
  page: Page,
  subscriber: number,
  headers: Record<string, string> = {},
  method = "GET",
) {
  const prefix = privatePrefixes.get(page);
  if (!prefix)
    throw new Error("Observe the real private Img before requesting a stream");
  const authorization = privateAuthorizations.get(page);
  return page.evaluate(
    async ({ subscriber, headers, method, prefix, authorization }) => {
      const response = await fetch(
        `${prefix}static/subscribers/${subscriber}/fixture-private.txt`,
        {
          headers: {
            ...headers,
            ...(authorization ? { Authorization: authorization } : {}),
          },
          method,
          credentials: "omit",
        },
      );
      return {
        status: response.status,
        body: await response.text(),
        cache: response.headers.get("cache-control"),
        etag: response.headers.get("etag"),
        contentRange: response.headers.get("content-range"),
      };
    },
    { subscriber, headers, method, prefix, authorization },
  );
}

test("public client PKCE, SDK callback and native private image work without third-party cookies", async ({
  page,
  deployment,
}) => {
  const { crossSiteUrl, crossSiteApiUrl } = deployment.descriptor;
  const requests: { url: string; headers: Record<string, string> }[] = [];
  const capabilityRequests: string[] = [];
  page.on("request", (request) => {
    capabilityRequests.push(request.url());
    if (
      request.url().startsWith(crossSiteApiUrl) ||
      request.url().includes("/token")
    )
      requests.push({ url: request.url(), headers: request.headers() });
  });
  await signIn(page, crossSiteUrl, "A");
  await openNativeFeed(page);
  const privateFile = await privateRequest(page, 1001);
  expect(privateFile).toMatchObject({
    status: 200,
    body: "fixture-private-content-1001",
    cache: "private, no-cache, no-transform",
  });
  expect(await privateRequest(page, 1002)).toMatchObject({ status: 403 });
  const range = await privateRequest(page, 1001, { Range: "bytes=0-6" });
  expect(range).toMatchObject({
    status: 206,
    body: "fixture",
    contentRange: "bytes 0-6/28",
  });
  if (!privateFile.etag)
    throw new Error("Private representation must expose its validator");
  expect(
    await privateRequest(page, 1001, { "If-None-Match": privateFile.etag }),
  ).toMatchObject({ status: 304, body: "" });
  expect(await privateRequest(page, 1001, {}, "HEAD")).toMatchObject({
    status: 200,
    body: "",
  });
  const bearer = requests.find(
    (request) =>
      request.url === `${crossSiteApiUrl}/api/graphql` &&
      request.headers.authorization?.startsWith("Bearer "),
  )?.headers.authorization;
  if (!bearer)
    throw new Error("The real SDK must authorize application requests");
  const mutation = await page.evaluate(
    async ({ api, bearer }) => {
      const response = await fetch(`${api}/api/graphql`, {
        method: "POST",
        credentials: "omit",
        headers: { Authorization: bearer, "Content-Type": "application/json" },
        body: JSON.stringify({ query: "mutation { __typename }" }),
      });
      const command = await fetch(
        `${api}/api/credential3rd/2147483647/check-available`,
        {
          method: "POST",
          credentials: "omit",
          headers: {
            Authorization: bearer,
            "Content-Type": "application/json",
          },
          body: "{}",
        },
      );
      return {
        status: response.status,
        body: await response.json(),
        commandStatus: command.status,
        commandBody: await command.json(),
      };
    },
    { api: crossSiteApiUrl, bearer },
  );
  expect(mutation).toEqual({
    status: 200,
    body: { data: { __typename: "Mutation" } },
    commandStatus: 403,
    commandBody: { success: false, message: "Permission denied" },
  });
  const instance = await page.context().newPage();
  await signIn(instance, deployment.descriptor.instanceUrl, "A");
  const otherPrincipal = await instance.evaluate(
    async ({ api, bearer }) => {
      const response = await fetch(`${api}/api/auth/user-info`, {
        credentials: "omit",
        headers: { Authorization: bearer },
      });
      return { status: response.status, body: await response.json() };
    },
    { api: deployment.descriptor.instanceApiUrl, bearer },
  );
  expect(otherPrincipal).toMatchObject({
    status: 200,
    body: {
      authType: "oidc",
      subject: "A",
      issuer: deployment.descriptor.issuer,
    },
  });
  expect(await privateRequest(instance, 1001)).toMatchObject({
    status: 200,
    body: "fixture-private-content-1001",
  });
  await instance.close();
  const poster = page.getByRole("img", { name: "Poster", exact: true }).first();
  const source = await poster.getAttribute("src");
  if (!source)
    throw new Error("The native private poster must expose its resource URL");
  const resourceUrl = new URL(source);
  expect(resourceUrl.origin).toBe(crossSiteApiUrl);
  expect(resourceUrl.searchParams.has("resource_token")).toBe(true);
  const imageRequests = requests.filter((request) =>
    new URL(request.url).searchParams.has("resource_token"),
  );
  expect(imageRequests.length).toBeGreaterThan(0);
  expect(
    imageRequests.every(
      (request) => request.headers.authorization === undefined,
    ),
  ).toBe(true);
  expect(
    await page.evaluate(async () => ({
      controller: navigator.serviceWorker.controller !== null,
      registrations: (await navigator.serviceWorker.getRegistrations()).length,
    })),
  ).toEqual({ controller: false, registrations: 0 });
  expect(
    capabilityRequests.some((url) =>
      /\/(?:sha2|hmac)-[^/]+\.js$/.test(new URL(url).pathname),
    ),
  ).toBe(false);
  expect(requests.some((request) => request.url.endsWith("/token"))).toBe(true);
  for (const request of requests) {
    expect(request.headers.cookie).toBeUndefined();
    expect(request.headers["x-konobangu-csrf"]).toBeUndefined();
    if (request.url.includes("/token")) {
      expect(request.headers.authorization).toBeUndefined();
    }
  }
});

test("a native image failure exchanges its parent once and retries an identical still-valid URL", async ({
  page,
  deployment,
}) => {
  const { crossSiteUrl, crossSiteApiUrl } = deployment.descriptor;
  let grants = 0;
  let attempts = 0;
  let firstGrant: string | undefined;
  const imageUrls: string[] = [];
  page.on("request", (request) => {
    if (
      request.url() === `${crossSiteApiUrl}/api/auth/resource-grants` &&
      request.method() === "POST"
    )
      grants++;
  });
  await page.route(
    `${crossSiteApiUrl}/api/auth/resource-grants`,
    async (route) => {
      const response = await route.fetch();
      expect(response.status()).toBe(200);
      firstGrant ??= await response.text();
      // Replay the still-valid first grant after a real authenticated exchange
      // so the retry must work even when the recorder returns identical material.
      await route.fulfill({ response, body: firstGrant });
    },
  );
  await page.route(
    (url) =>
      url.origin === crossSiteApiUrl &&
      url.pathname.endsWith("/image/fixture/browser-poster.png") &&
      url.searchParams.has("resource_token"),
    async (route) => {
      attempts++;
      imageUrls.push(route.request().url());
      expect(route.request().headers().authorization).toBeUndefined();
      if (attempts === 1)
        await route.fulfill({
          status: 401,
          body: "Expired fixture resource capability",
        });
      else await route.continue();
    },
  );
  await signIn(page, crossSiteUrl, "A");
  expect(attempts).toBe(2);
  expect(grants).toBe(2);
  expect(imageUrls[1]).toBe(imageUrls[0]);
  await expect(page.getByRole("alert")).toHaveCount(0);
});

test("a real LAN HTTP page completes PKCE and reads only attenuated native image URLs", async ({
  page,
  deployment,
}) => {
  const { lanUrl, lanApiUrl, issuer } = deployment.descriptor;
  const requests: { url: string; authorization?: string; cookie?: string }[] =
    [];
  await page.addInitScript(() => {
    const nativeRandom = crypto.getRandomValues;
    const nativeSubtle = crypto.subtle;
    Object.defineProperty(globalThis, "__konobanguCryptoInspection", {
      value: () => ({
        secure: isSecureContext,
        nativeRandomPreserved: crypto.getRandomValues === nativeRandom,
        originallyHadSubtle: nativeSubtle !== undefined,
        digestReady: typeof crypto.subtle?.digest === "function",
        hasServiceWorker: "serviceWorker" in navigator,
      }),
    });
  });
  page.on("request", (request) => {
    const headers = request.headers();
    requests.push({
      url: request.url(),
      authorization: headers.authorization,
      cookie: headers.cookie,
    });
  });
  const grantResponse = page.waitForResponse(
    (response) =>
      response.url() === `${lanApiUrl}/api/auth/resource-grants` &&
      response.status() === 200,
  );
  void grantResponse.catch(() => {});
  await signIn(page, lanUrl, "A");
  await openNativeFeed(page);
  const inspection = type({
    secure: "boolean",
    nativeRandomPreserved: "boolean",
    originallyHadSubtle: "boolean",
    digestReady: "boolean",
    hasServiceWorker: "boolean",
  }).assert(
    await page.evaluate(() => {
      const inspect: unknown = Reflect.get(
        globalThis,
        "__konobanguCryptoInspection",
      );
      if (typeof inspect !== "function")
        throw new Error("Crypto inspection was not installed");
      return inspect();
    }),
  );
  expect(inspection).toEqual({
    secure: false,
    nativeRandomPreserved: true,
    originallyHadSubtle: false,
    digestReady: true,
    hasServiceWorker: false,
  });
  const grant = type({
    grant: "string",
    issuedAt: "string",
    expiresAt: "string",
    childTtlSeconds: "300",
  }).assert(await (await grantResponse).json());
  const poster = page.getByRole("img", { name: "Poster", exact: true }).first();
  const source = await poster.getAttribute("src");
  if (!source)
    throw new Error(
      "The real LAN poster did not acquire a child resource token",
    );
  const childUrl = new URL(source);
  expect(childUrl.origin).toBe(lanApiUrl);
  const child = childUrl.searchParams.get("resource_token");
  if (!child) throw new Error("Native image URL lacks its attenuated token");
  expect(Buffer.from(child, "base64url").toString("utf8")).not.toBe(
    grant.grant,
  );
  const parentUrl = new URL(childUrl);
  parentUrl.searchParams.set(
    "resource_token",
    Buffer.from(grant.grant).toString("base64url"),
  );
  const wrongOwner = new URL(childUrl);
  wrongOwner.pathname = wrongOwner.pathname.replace("/1001/", "/1002/");
  const wrongResource = new URL(childUrl);
  wrongResource.pathname = wrongResource.pathname.replace(
    "browser-poster.png",
    "other-poster.png",
  );
  const checks = await page.evaluate(
    async (urls) =>
      Promise.all(
        urls.map(
          async (url) => (await fetch(url, { credentials: "omit" })).status,
        ),
      ),
    [childUrl.href, parentUrl.href, wrongOwner.href, wrongResource.href],
  );
  expect(checks[0]).toBe(200);
  for (const status of checks.slice(1)) expect([401, 403]).toContain(status);
  expect(await privateRequest(page, 1001)).toMatchObject({
    status: 200,
    body: "fixture-private-content-1001",
  });
  const nativeRequests = requests.filter((request) =>
    new URL(request.url).searchParams.has("resource_token"),
  );
  expect(nativeRequests.length).toBeGreaterThan(0);
  expect(
    nativeRequests.every(
      (request) => !request.authorization && !request.cookie,
    ),
  ).toBe(true);
  expect(
    requests.some(
      (request) =>
        request.url === `${lanApiUrl}/api/graphql` &&
        request.authorization?.startsWith("Bearer "),
    ),
  ).toBe(true);
  expect(
    requests.some(
      (request) =>
        request.url.startsWith(issuer) && request.url.endsWith("/token"),
    ),
  ).toBe(true);
  expect(
    requests.some((request) =>
      /\/sha2-[^/]+\.js$/.test(new URL(request.url).pathname),
    ),
  ).toBe(true);
  expect(
    requests.some(
      (request) => new URL(request.url).pathname === "/private-sw.js",
    ),
  ).toBe(false);
});

test("same-origin and cross-site deployments serve the identical production bundle", async ({
  page,
  context,
  deployment,
}) => {
  await signIn(page, deployment.descriptor.baseUrl, "A");
  expect(deployment.descriptor.assetPaths).not.toContain("/private-sw.js");
  expect(
    deployment.descriptor.assetPaths.some((path) => path.endsWith(".css")),
  ).toBe(true);
  const fingerprint = (page: Page) =>
    page.evaluate(
      async (paths) =>
        Promise.all(
          paths.map(async (path) => {
            const response = await fetch(path, { credentials: "omit" });
            if (!response.ok)
              throw new Error(`Published asset missing: ${path}`);
            const bytes = await response.arrayBuffer();
            const hash = Array.from(
              new Uint8Array(await crypto.subtle.digest("SHA-256", bytes)),
            )
              .map((value) => value.toString(16).padStart(2, "0"))
              .join("");
            return { path, hash };
          }),
        ),
      deployment.descriptor.assetPaths,
    );
  const same = await fingerprint(page);
  const cross = await context.newPage();
  await signIn(cross, deployment.descriptor.crossSiteUrl, "A");
  expect(await fingerprint(cross)).toEqual(same);
  await cross.close();
});

test("OIDC local logout closes old private epochs without fabricating remote token revocation", async ({
  page,
  deployment,
}) => {
  const { crossSiteUrl, crossSiteApiUrl, issuer } = deployment.descriptor;
  const response = page.waitForResponse(
    (response) =>
      response.url() === `${issuer}/token` &&
      response.request().method() === "POST",
  );
  await signIn(page, crossSiteUrl, "A");
  const tokens: unknown = await (await response).json();
  if (
    typeof tokens !== "object" ||
    tokens === null ||
    !("access_token" in tokens) ||
    typeof tokens.access_token !== "string"
  )
    throw new Error("Real provider did not return its API access token");
  const oldResource = await page
    .getByRole("img", { name: "Poster", exact: true })
    .first()
    .getAttribute("src");
  if (!oldResource)
    throw new Error(
      "The authenticated page must have loaded a resource capability",
    );
  const urls: string[] = [];
  page.on("request", (request) => urls.push(request.url()));
  await page.getByText("A", { exact: true }).first().click();
  await page.getByRole("menuitem", { name: "Sign out", exact: true }).click();
  await expect(page).toHaveURL(`${crossSiteUrl}/auth/signed-out`);
  await expect(
    page.getByRole("img", { name: "Poster", exact: true }),
  ).toHaveCount(0);
  const resourceStatus = await page.evaluate(
    async (url) => (await fetch(url, { credentials: "omit" })).status,
    oldResource,
  );
  expect(resourceStatus).toBe(200);
  const unauthenticated = await page.evaluate(
    async (api) =>
      (await fetch(`${api}/api/auth/user-info`, { credentials: "omit" }))
        .status,
    crossSiteApiUrl,
  );
  expect(unauthenticated).toBe(401);
  const remaining = await page.evaluate(
    async ({ api, token }) => {
      const response = await fetch(`${api}/api/auth/user-info`, {
        credentials: "omit",
        headers: { Authorization: `Bearer ${token}` },
      });
      return response.status;
    },
    { api: crossSiteApiUrl, token: tokens.access_token },
  );
  expect(remaining).toBe(200);
  expect(
    urls.some((url) => /\/(revocation|revoke|introspect)(?:[/?]|$)/.test(url)),
  ).toBe(false);
  await expect(
    page.getByRole("button", { name: "Sign in again" }),
  ).toBeVisible();
});

test("two OIDC pages keep their own SDK identity and native resource authorization", async ({
  context,
  deployment,
}) => {
  const first = await context.newPage();
  const second = await context.newPage();
  const { crossSiteUrl: origin, issuer } = deployment.descriptor;
  await signIn(first, origin, "A");
  // Clearing only IdP SSO cookies changes the next login, not either page's
  // SDK-held tokens. Both pages remain in one origin and one browser context.
  await context.clearCookies({ domain: new URL(issuer).hostname });
  await signIn(second, origin, "B");
  expect(await privateRequest(first, 1001)).toMatchObject({
    status: 200,
    body: "fixture-private-content-1001",
  });
  expect(await privateRequest(second, 1002)).toMatchObject({
    status: 200,
    body: "fixture-private-content-1002",
  });
  expect(await privateRequest(first, 1002)).toMatchObject({ status: 403 });
  expect(await privateRequest(second, 1001)).toMatchObject({ status: 403 });
  await first.close();
  await second.close();
});

test("bundled public-client OIDC confines Basic to provider login and exchanges native resource grants", async ({
  page,
  context,
  browserName,
  deployment,
}) => {
  if (browserName !== "chromium")
    throw new Error(
      "This protocol test requires Chromium CDP; focused Firefox/WebKit tests use issuer-scoped native challenge responders separately",
    );
  const { bundledUrl, bundledApiUrl, bundledIssuer } = deployment.descriptor;
  page.on("request", (request) => {
    const authorization = request.headers().authorization;
    if (
      new URL(request.url()).pathname === "/api/graphql" &&
      authorization?.startsWith("Bearer ")
    )
      privateAuthorizations.set(page, authorization);
  });
  {
    await using challenge = await nativeBasicChallenge(
      context,
      page,
      bundledIssuer,
    );
    await page.goto(`${bundledUrl}/subscriptions/detail/5000`);
    await expect
      .poll(() => page.url(), { timeout: 30_000 })
      .toBe(`${bundledUrl}/subscriptions/detail/5000`);
    await expect(
      page.getByText("Media browser fixture", { exact: true }),
    ).toBeVisible();
    expect(challenge.answered).toBe(1);
  }
  const bearer = privateAuthorizations.get(page);
  if (!bearer)
    throw new Error("Bundled SDK login did not authorize application requests");
  const probe = await page.evaluate(
    async ({ api, bearer }) => {
      const response = await fetch(`${api}/api/auth/user-info`, {
        credentials: "omit",
        headers: { Authorization: bearer },
      });
      return {
        status: response.status,
        authType: (await response.json()).authType,
      };
    },
    { api: bundledApiUrl, bearer },
  );
  expect(probe).toEqual({ status: 200, authType: "oidc" });
  const renewed = page.waitForResponse(
    (response) =>
      response.url() === `${bundledIssuer}/token` &&
      response.request().postData()?.includes("grant_type=refresh_token") ===
        true,
    { timeout: 60_000 },
  );
  void renewed.catch(() => {});
  for (const [endpoint, method] of [
    ["auth/user-info", "GET"],
    ["graphql", "POST"],
    ["graphql/introspection", "POST"],
    ["auth/resource-grants", "POST"],
  ] as const) {
    const response = await context.request.fetch(
      `${bundledApiUrl}/api/${endpoint}`,
      {
        method,
        headers: {
          Authorization: `Basic ${Buffer.from("fixture-admin:fixture-admin-password").toString("base64")}`,
          Origin: bundledUrl,
          "Content-Type": "application/json",
        },
        ...(method === "POST" ? { data: {} } : {}),
      },
    );
    expect(response.status()).toBe(401);
    expect(
      response.headers()["www-authenticate"]?.startsWith("Basic ") ?? false,
    ).toBe(false);
  }
  const removedLogin = await context.request.get(`${bundledApiUrl}/api/login`, {
    headers: { Authorization: bearer },
  });
  expect(removedLogin.status()).toBe(404);

  privatePrefixes.set(page, await privateResourcePrefix(page));
  await openNativeFeed(page);
  expect(await privateRequest(page, 1)).toMatchObject({
    status: 200,
    body: "fixture-private-content-1",
    cache: "private, no-cache, no-transform",
  });
  await expect(page.locator('input[type="password"]')).toHaveCount(0);
  const sourceValidated = await page.evaluate(
    async ({ api, bearer }) => {
      const response = await fetch(`${api}/api/graphql`, {
        credentials: "omit",
        method: "POST",
        headers: { "Content-Type": "application/json", Authorization: bearer },
        body: JSON.stringify({ query: "mutation { __typename }" }),
      });
      const command = await fetch(
        `${api}/api/credential3rd/2147483647/check-available`,
        {
          credentials: "omit",
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            Authorization: bearer,
          },
          body: "{}",
        },
      );
      return { mutation: response.status, command: command.status };
    },
    { api: bundledApiUrl, bearer },
  );
  expect(sourceValidated).toEqual({ mutation: 200, command: 403 });
  await page.goto(`${bundledUrl}/tasks/cron/edit/5020`);
  await page.getByLabel("Timezone *", { exact: true }).fill("Europe/Paris");
  await page
    .getByPlaceholder("0 0 * * * *", { exact: true })
    .fill("0 */10 * * * *");
  const updated = page.waitForResponse(
    (response) =>
      response.url() === `${bundledApiUrl}/api/graphql` &&
      response.request().postData()?.includes("UpdateCrons") === true,
  );
  await page.getByRole("button", { name: "Save", exact: true }).click();
  expect((await updated).status()).toBe(200);
  await page.goto(`${bundledUrl}/tasks/cron/detail/5020`);
  await expect(
    page.getByText("Europe/Paris", { exact: false }).first(),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Retry", exact: true }),
  ).toHaveCount(0);
  await page.locator('a[href="/tasks/detail/fixture-cron-run"]').click();
  await page.getByRole("button", { name: "Retry", exact: true }).click();
  await expect(
    page.getByText("Generation", { exact: true }).locator("..").locator("span"),
  ).toHaveText("2");
  await page.waitForTimeout(31_000);
  const tasks = page.locator('a[href="/tasks/manage"]').first();
  if (!(await tasks.isVisible()))
    await page
      .getByRole("button", { name: "Tasks (expandable)", exact: true })
      .click();
  await tasks.click();
  expect((await renewed).status()).toBe(200);
  const refreshedBearer = privateAuthorizations.get(page);
  if (!refreshedBearer)
    throw new Error("Renewed SDK access token was not used");
  await page.getByRole("button", { name: "User menu", exact: true }).click();
  await page.getByRole("menuitem", { name: "Sign out", exact: true }).click();
  await expect(page).toHaveURL(`${bundledUrl}/auth/signed-out`);
  const previousToken = await context.request.get(
    `${bundledApiUrl}/api/auth/user-info`,
    { headers: { Authorization: refreshedBearer } },
  );
  expect(previousToken.status()).toBe(200);
  await page.getByRole("button", { name: "Sign in again" }).click();
  await expect(page).toHaveURL(`${bundledUrl}/subscriptions/manage`);
  await expect(
    page.getByRole("heading", { name: "Subscription Management", exact: true }),
  ).toBeVisible();
});

test("LAN HTTP WebUI uses the bundled HTTPS issuer and native resource tokens", async ({
  page,
  context,
  deployment,
}) => {
  const { lanBundledUrl, bundledIssuer } = deployment.descriptor;
  const grants: number[] = [];
  page.on("response", (response) => {
    if (response.url() === `${lanBundledUrl}/api/auth/resource-grants`)
      grants.push(response.status());
  });
  {
    await using challenge = await nativeBasicChallenge(
      context,
      page,
      bundledIssuer,
    );
    await page.goto(`${lanBundledUrl}/subscriptions/detail/5000`);
    await privateResourcePrefix(page);
    await openNativeFeed(page);
    expect(challenge.answered).toBe(1);
  }
  expect(grants).toEqual([200]);
  expect(
    await page.evaluate(() => ({
      secure: isSecureContext,
      worker: "serviceWorker" in navigator,
    })),
  ).toEqual({ secure: false, worker: false });
  const source = await page
    .getByRole("img", { name: "Poster", exact: true })
    .first()
    .getAttribute("src");
  if (!source) throw new Error("Native image must have a scoped URL");
  const loaded = await page.evaluate(async (url) => {
    const response = await fetch(url, { credentials: "omit" });
    return {
      status: response.status,
      cache: response.headers.get("cache-control"),
    };
  }, source);
  expect(loaded).toEqual({
    status: 200,
    cache: "private, no-cache, no-transform",
  });
  await expect(page.locator('input[type="password"]')).toHaveCount(0);
});

test("a failed or absent SDK callback never performs a fabricated login/navigation", async ({
  page,
  deployment,
}) => {
  const origin = deployment.descriptor.crossSiteUrl;
  await page.goto(`${origin}/auth/callback`);
  await expect(
    page.getByRole("alert").filter({
      hasText: "does not match a pending frontend-mode login",
    }),
  ).toBeVisible();
  await page.goto(`${origin}/auth/callback?code=invalid&state=unbound`);
  await expect(
    page.getByRole("alert").filter({
      hasText: "does not match a pending frontend-mode login",
    }),
  ).toBeVisible();
  expect(new URL(page.url()).pathname).toBe("/auth/callback");
  await page.goto(`${deployment.descriptor.bundledUrl}/auth/callback`);
  await expect(
    page.getByRole("alert").filter({
      hasText: "does not match a pending frontend-mode login",
    }),
  ).toBeVisible();
  expect(new URL(page.url()).pathname).toBe("/auth/callback");
});

test("actual expiry renews through SDK and retained-key rotation authorizes a new login", async ({
  page,
  deployment,
}) => {
  test.setTimeout(120_000);
  const { crossSiteUrl, issuer } = deployment.descriptor;
  await signIn(page, crossSiteUrl, "A");
  const renewed = page.waitForResponse(
    (response) =>
      response.url() === `${issuer}/token` &&
      response.request().postData()?.includes("grant_type=refresh_token") ===
        true,
    { timeout: 60_000 },
  );
  await page.waitForTimeout(31_000);
  const tasks = page.locator('a[href="/tasks/manage"]').first();
  if (!(await tasks.isVisible()))
    await page.getByText("Tasks", { exact: true }).first().click();
  await tasks.click();
  expect((await renewed).status()).toBe(200);
  await expect(page.getByText("No tasks found", { exact: true })).toBeVisible();
  expect(await privateRequest(page, 1001)).toMatchObject({ status: 200 });
  await page.getByText("A", { exact: true }).first().click();
  await page.getByRole("menuitem", { name: "Sign out", exact: true }).click();
  await expect(page).toHaveURL(`${crossSiteUrl}/auth/signed-out`);
  await deployment.rotateSigningKey();
  const next = page.waitForResponse(
    (response) =>
      response.url() === `${issuer}/token` &&
      response
        .request()
        .postData()
        ?.includes("grant_type=authorization_code") === true,
  );
  await page.getByRole("button", { name: "Sign in again" }).click();
  await expect
    .poll(
      async () => {
        if (
          new URL(page.url()).origin === crossSiteUrl &&
          new URL(page.url()).pathname === "/subscriptions/manage" &&
          (await page.getByText("A", { exact: true }).first().isVisible())
        )
          return true;
        const interaction = page.getByRole("button", {
          name: "Sign in as A",
          exact: true,
        });
        if (await interaction.isVisible()) await interaction.click();
        return false;
      },
      { timeout: 30_000 },
    )
    .toBe(true);
  const data: unknown = await (await next).json();
  if (
    typeof data !== "object" ||
    data === null ||
    !("access_token" in data) ||
    typeof data.access_token !== "string"
  )
    throw new Error("Rotated provider did not issue an access token");
  expect(decodeProtectedHeader(data.access_token).kid).toMatch(
    /^test-rsa-\d+$/,
  );
  await page.goto(`${crossSiteUrl}/subscriptions/detail/6000`);
  privatePrefixes.set(page, await privateResourcePrefix(page));
  expect(await privateRequest(page, 1001)).toMatchObject({ status: 200 });
});

test("two allowed WebUI entries select distinct callbacks against the same API", async ({
  page,
  deployment,
}) => {
  const { crossSiteUrl, alternateUrl, crossSiteApiUrl } = deployment.descriptor;
  for (const origin of [crossSiteUrl, alternateUrl]) {
    const configuration = page.waitForResponse(
      (response) => response.url() === `${crossSiteApiUrl}/api/auth/config`,
    );
    await signIn(page, origin, "A");
    const published = await configuration;
    expect(published.status()).toBe(200);
    expect(await published.json()).toMatchObject({
      provider: "oidc",
      redirectUrl: `${origin}/auth/callback`,
    });
    expect(new URL(page.url()).origin).toBe(origin);
    expect(await privateRequest(page, 1001)).toMatchObject({
      status: 200,
      body: "fixture-private-content-1001",
    });
  }
});
